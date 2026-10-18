//! The use cases, on a repository and identity providers. Each change is
//! logged with ids, never documents, tokens or claims (ADR BE-LOGS).

use anyhow::anyhow;
use async_trait::async_trait;
use palimpseste_document::{Document, Engine};

use crate::{
    ActionContext, Character, CharacterId, CharacterName, Error, IdentityProviders, LinkedIdentity,
    Linking, Nickname, PalimpsesteRepository, PalimpsesteService, PendingRegistration,
    PendingSignIn, Profile, ProviderAnswer, ProviderId, ProviderSummary, REGISTRATION_TTL,
    RegistrationToken, SESSION_TTL, SIGN_IN_TTL, SessionToken, SignInEnd, SignInId, StartedSignIn,
    User, UserId, VerifiedIdentity, Version,
};

/// The use cases, keeping their data in `R` and signing in through `P`.
pub struct PalimpsesteServiceImpl<R: PalimpsesteRepository, P: IdentityProviders> {
    repo: R,
    providers: P,
}

impl<R: PalimpsesteRepository, P: IdentityProviders> PalimpsesteServiceImpl<R, P> {
    /// The use cases on `repo` and `providers`.
    pub const fn new(repo: R, providers: P) -> Self {
        Self { repo, providers }
    }

    /// Takes the sign-in in progress `sign_in` out of the repository,
    /// checking it was started with `provider`.
    async fn take_sign_in(
        &self,
        provider: &ProviderId,
        sign_in: Option<SignInId>,
    ) -> Result<PendingSignIn, Error> {
        let sign_in = sign_in.ok_or_else(|| anyhow!("no sign-in in progress in this browser"))?;
        let pending = self
            .repo
            .take_sign_in(&sign_in)
            .await?
            .ok_or_else(|| anyhow!("unknown or expired sign-in"))?;
        if pending.provider == *provider {
            Ok(pending)
        } else {
            Err(anyhow!("sign-in started with another provider").into())
        }
    }

    /// Checks the provider's answer against the sign-in in progress and
    /// has the provider verify the identity.
    async fn verify(
        &self,
        provider: &ProviderId,
        pending: &PendingSignIn,
        answer: ProviderAnswer,
    ) -> Result<VerifiedIdentity, Error> {
        if answer.state.as_deref() != Some(pending.state.as_str()) {
            return Err(anyhow!("state does not match").into());
        }
        let code = match (answer.code, answer.error) {
            (_, Some(error)) => return Err(anyhow!("the provider answered {error}").into()),
            (None, None) => return Err(anyhow!("no code").into()),
            (Some(code), None) => code,
        };
        self.providers.verify(provider, &code, pending).await
    }

    /// Opens a session for the user of `identity`, or starts its
    /// registration when no user has it (ADR BE-FIRST-SIGN-IN).
    async fn sign_in(
        &self,
        provider: &ProviderId,
        identity: &VerifiedIdentity,
    ) -> Result<SignInEnd, Error> {
        let known = self
            .repo
            .identity_user(&identity.issuer, &identity.subject, identity.label())
            .await?;
        if let Some(user) = known {
            let token = self.open_session(user.id).await?;
            tracing::info!(user_id = %user.id, %provider, "signed in");
            return Ok(SignInEnd::SignedIn(token));
        }
        let token = RegistrationToken::random();
        self.repo
            .create_registration(
                &token.hash(),
                &identity.issuer,
                &identity.subject,
                identity.wanted_nickname(),
                identity.label(),
                REGISTRATION_TTL,
            )
            .await?;
        tracing::info!(%provider, "registration started");
        Ok(SignInEnd::Registering(token))
    }

    /// Opens a session of `user`.
    async fn open_session(&self, user: UserId) -> Result<SessionToken, Error> {
        let token = SessionToken::random();
        self.repo
            .create_session(user, &token.hash(), SESSION_TTL)
            .await?;
        Ok(token)
    }

    /// Links `identity` to `user`, unless it belongs to another user.
    async fn link_identity(
        &self,
        provider: &ProviderId,
        user: UserId,
        identity: &VerifiedIdentity,
    ) -> Result<Linking, Error> {
        let linked = self
            .repo
            .link_identity(user, &identity.issuer, &identity.subject, identity.label())
            .await;
        match &linked {
            Ok(Linking::Linked) => tracing::info!(user_id = %user, %provider, "identity linked"),
            Ok(Linking::AlreadyYours) => {
                tracing::info!(user_id = %user, %provider, "identity already linked");
            }
            Err(Error::IdentityTaken) => {
                tracing::warn!(user_id = %user, %provider, "identity of another user not linked");
            }
            Err(Error::ProviderAlreadyLinked) => {
                tracing::info!(user_id = %user, %provider, "second identity at a provider not linked");
            }
            Err(_) => {}
        }
        linked
    }
}

/// `error` as a failed sign-in, unless it tells the user something more
/// precise.
fn sign_in_failed(linking: bool, error: Error) -> Error {
    match error {
        Error::Underlying(cause) => Error::SignInFailed { linking, cause },
        Error::SignInFailed { .. } | Error::IdentityTaken | Error::ProviderAlreadyLinked => error,
        other => Error::SignInFailed {
            linking,
            cause: anyhow::Error::new(other),
        },
    }
}

#[async_trait]
impl<R: PalimpsesteRepository, P: IdentityProviders> PalimpsesteService
    for PalimpsesteServiceImpl<R, P>
{
    async fn health(&self) -> Result<(), Error> {
        self.repo.ping().await
    }

    fn providers(&self) -> Vec<ProviderSummary> {
        self.providers.list()
    }

    async fn start_sign_in(
        &self,
        provider: &ProviderId,
        link: Option<ActionContext>,
    ) -> Result<StartedSignIn, Error> {
        let authorization = match self.providers.authorize(provider).await {
            Ok(authorization) => authorization,
            Err(Error::Underlying(cause)) => {
                return Err(Error::SignInFailed {
                    linking: link.is_some(),
                    cause,
                });
            }
            Err(error) => return Err(error),
        };
        let pending = PendingSignIn {
            provider: provider.clone(),
            state: authorization.state,
            nonce: authorization.nonce,
            pkce_verifier: authorization.pkce_verifier,
            link: link.map(|ctx| ctx.user),
        };
        let id = SignInId::random();
        self.repo.purge_expired().await?;
        self.repo.create_sign_in(&id, &pending, SIGN_IN_TTL).await?;
        Ok(StartedSignIn {
            id,
            url: authorization.url,
        })
    }

    async fn finish_sign_in(
        &self,
        provider: &ProviderId,
        sign_in: Option<SignInId>,
        answer: ProviderAnswer,
    ) -> Result<SignInEnd, Error> {
        let pending = self
            .take_sign_in(provider, sign_in)
            .await
            .map_err(|error| sign_in_failed(false, error))?;
        let link = pending.link;
        let failed = |error| sign_in_failed(link.is_some(), error);
        let identity = self
            .verify(provider, &pending, answer)
            .await
            .map_err(failed)?;
        match link {
            None => self.sign_in(provider, &identity).await.map_err(failed),
            Some(user) => self
                .link_identity(provider, user, &identity)
                .await
                .map(SignInEnd::Linked)
                .map_err(failed),
        }
    }

    async fn registration(&self, token: &RegistrationToken) -> Result<PendingRegistration, Error> {
        let registration = self
            .repo
            .registration(&token.hash())
            .await?
            .ok_or(Error::NoRegistration)?;
        Ok(PendingRegistration {
            provider: self.providers.provider_of(&registration.issuer),
            registration,
        })
    }

    async fn register(
        &self,
        token: &RegistrationToken,
        nickname: Nickname,
        profile: Profile,
    ) -> Result<(User, SessionToken), Error> {
        let user = self
            .repo
            .register(&token.hash(), nickname, profile)
            .await?
            .ok_or(Error::NoRegistration)?;
        tracing::info!(user_id = %user.id, "user created");
        let session = self.open_session(user.id).await?;
        tracing::info!(user_id = %user.id, "signed in");
        Ok((user, session))
    }

    async fn cancel_registration(&self, token: &RegistrationToken) -> Result<(), Error> {
        self.repo.delete_registration(&token.hash()).await?;
        tracing::info!("registration cancelled");
        Ok(())
    }

    async fn session_user(&self, token: &SessionToken) -> Result<User, Error> {
        self.repo
            .session_user(&token.hash())
            .await?
            .ok_or(Error::NoSession)
    }

    async fn sign_out(&self, token: &SessionToken) -> Result<(), Error> {
        self.repo.delete_session(&token.hash()).await?;
        tracing::info!("signed out");
        Ok(())
    }

    async fn update_profile(
        &self,
        ctx: &ActionContext,
        nickname: Option<Nickname>,
        profile: Option<Profile>,
    ) -> Result<User, Error> {
        let user = self.repo.update_user(ctx.user, nickname, profile).await?;
        tracing::info!(user_id = %user.id, "profile updated");
        Ok(user)
    }

    async fn identities(&self, ctx: &ActionContext) -> Result<Vec<LinkedIdentity>, Error> {
        let identities = self.repo.identities(ctx.user).await?;
        Ok(identities
            .into_iter()
            .map(|identity| LinkedIdentity {
                provider: self.providers.provider_of(&identity.issuer),
                identity,
            })
            .collect())
    }

    async fn characters(&self, ctx: &ActionContext) -> Result<Vec<Character>, Error> {
        self.repo.characters(ctx.user).await
    }

    async fn create_character(
        &self,
        ctx: &ActionContext,
        name: CharacterName,
        engine: Engine,
    ) -> Result<Character, Error> {
        let character = self
            .repo
            .create_character(ctx.user, &name, &Document::empty(engine))
            .await?;
        tracing::info!(character_id = %character.id, user_id = %ctx.user, engine = ?character.engine, "character created");
        Ok(character)
    }

    async fn character(&self, ctx: &ActionContext, id: CharacterId) -> Result<Character, Error> {
        self.repo
            .character(ctx.user, id)
            .await?
            .ok_or(Error::CharacterNotFound)
    }

    async fn rename_character(
        &self,
        ctx: &ActionContext,
        id: CharacterId,
        name: CharacterName,
    ) -> Result<Character, Error> {
        let character = self
            .repo
            .rename_character(ctx.user, id, &name)
            .await?
            .ok_or(Error::CharacterNotFound)?;
        tracing::info!(character_id = %id, "character renamed");
        Ok(character)
    }

    async fn delete_character(&self, ctx: &ActionContext, id: CharacterId) -> Result<(), Error> {
        if !self.repo.delete_character(ctx.user, id).await? {
            return Err(Error::CharacterNotFound);
        }
        tracing::info!(character_id = %id, "character deleted");
        Ok(())
    }

    async fn replace_document(
        &self,
        ctx: &ActionContext,
        id: CharacterId,
        expected: Version,
        document: Document,
    ) -> Result<Version, Error> {
        document
            .check()
            .map_err(|error| Error::UnreadableDocument(error.to_string()))?;
        // A character never changes engine, so reading it first is safe.
        let character = self.character(ctx, id).await?;
        if character.engine != document.engine() {
            return Err(Error::OtherEngine);
        }
        let version = self
            .repo
            .replace_document(ctx.user, id, expected, &document)
            .await?;
        tracing::info!(character_id = %id, %version, "document stored");
        Ok(version)
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use super::*;
    use crate::{
        Authorization, Identity, Registration, provider::MockIdentityProviders,
        repo::MockPalimpsesteRepository,
    };

    const ANA: ActionContext = ActionContext { user: UserId(1) };

    fn service(
        repo: MockPalimpsesteRepository,
        providers: MockIdentityProviders,
    ) -> PalimpsesteServiceImpl<MockPalimpsesteRepository, MockIdentityProviders> {
        PalimpsesteServiceImpl::new(repo, providers)
    }

    fn dex() -> ProviderId {
        "dex".into()
    }

    fn pending(link: Option<UserId>) -> PendingSignIn {
        PendingSignIn {
            provider: dex(),
            state: "state".into(),
            nonce: "nonce".into(),
            pkce_verifier: "verifier".into(),
            link,
        }
    }

    fn answer(state: &str) -> ProviderAnswer {
        ProviderAnswer {
            code: Some("code".into()),
            state: Some(state.into()),
            error: None,
        }
    }

    fn verified() -> VerifiedIdentity {
        VerifiedIdentity {
            issuer: "https://issuer.test".into(),
            subject: "ana".into(),
            preferred_username: None,
            name: Some("Ana".into()),
            email: Some("ana@mail.test".into()),
        }
    }

    fn user() -> User {
        User {
            id: ANA.user,
            nickname: Nickname::parse("Ana").unwrap(),
            profile: Profile::default(),
        }
    }

    #[tokio::test]
    async fn healthy_while_the_database_answers() {
        let mut repo = MockPalimpsesteRepository::new();
        repo.expect_ping().times(1).returning(|| Ok(()));
        service(repo, MockIdentityProviders::new())
            .health()
            .await
            .unwrap();

        let mut repo = MockPalimpsesteRepository::new();
        repo.expect_ping()
            .returning(|| Err(anyhow!("disk I/O error").into()));
        let result = service(repo, MockIdentityProviders::new()).health().await;
        assert!(matches!(result, Err(Error::Underlying(_))));
    }

    #[tokio::test]
    async fn starting_a_sign_in_keeps_it_for_its_callback() {
        let mut providers = MockIdentityProviders::new();
        providers.expect_authorize().returning(|_| {
            Ok(Authorization {
                url: "https://issuer.test/authorize".into(),
                state: "state".into(),
                nonce: "nonce".into(),
                pkce_verifier: "verifier".into(),
            })
        });
        let mut repo = MockPalimpsesteRepository::new();
        repo.expect_purge_expired().times(1).returning(|| Ok(()));
        repo.expect_create_sign_in()
            .withf(|_, sign_in, ttl| *sign_in == pending(Some(ANA.user)) && *ttl == SIGN_IN_TTL)
            .times(1)
            .returning(|_, _, _| Ok(()));
        let started = service(repo, providers)
            .start_sign_in(&dex(), Some(ANA))
            .await
            .unwrap();
        assert_eq!(started.url, "https://issuer.test/authorize");
    }

    #[tokio::test]
    async fn an_unreachable_provider_fails_the_sign_in() {
        let mut providers = MockIdentityProviders::new();
        providers
            .expect_authorize()
            .returning(|_| Err(anyhow!("discovery failed").into()));
        let result = service(MockPalimpsesteRepository::new(), providers)
            .start_sign_in(&dex(), Some(ANA))
            .await;
        assert!(matches!(
            result,
            Err(Error::SignInFailed { linking: true, .. })
        ));
    }

    #[tokio::test]
    async fn a_callback_not_matching_its_sign_in_fails() {
        let mut repo = MockPalimpsesteRepository::new();
        repo.expect_take_sign_in()
            .returning(|_| Ok(Some(pending(Some(ANA.user)))));
        let result = service(repo, MockIdentityProviders::new())
            .finish_sign_in(&dex(), Some(SignInId::from_cookie("id")), answer("forged"))
            .await;
        assert!(matches!(
            result,
            Err(Error::SignInFailed { linking: true, .. })
        ));

        let result = service(
            MockPalimpsesteRepository::new(),
            MockIdentityProviders::new(),
        )
        .finish_sign_in(&dex(), None, answer("state"))
        .await;
        assert!(matches!(
            result,
            Err(Error::SignInFailed { linking: false, .. })
        ));
    }

    /// A callback answered with the identity `verified()`.
    fn signing_in(
        mut repo: MockPalimpsesteRepository,
    ) -> PalimpsesteServiceImpl<MockPalimpsesteRepository, MockIdentityProviders> {
        let mut providers = MockIdentityProviders::new();
        providers
            .expect_verify()
            .withf(|_, code, _| code == "code")
            .returning(|_, _, _| Ok(verified()));
        repo.expect_take_sign_in()
            .returning(|_| Ok(Some(pending(None))));
        service(repo, providers)
    }

    #[tokio::test]
    async fn a_callback_opens_a_session_for_a_known_identity() {
        let mut repo = MockPalimpsesteRepository::new();
        repo.expect_identity_user()
            .with(eq("https://issuer.test"), eq("ana"), eq("ana@mail.test"))
            .returning(|_, _, _| Ok(Some(user())));
        repo.expect_create_registration().never();
        repo.expect_create_session()
            .withf(|user, _, ttl| *user == ANA.user && *ttl == SESSION_TTL)
            .times(1)
            .returning(|_, _, _| Ok(()));
        let ended = signing_in(repo)
            .finish_sign_in(&dex(), Some(SignInId::from_cookie("id")), answer("state"))
            .await
            .unwrap();
        assert!(matches!(ended, SignInEnd::SignedIn(_)));
    }

    #[tokio::test]
    async fn a_callback_starts_registering_an_unknown_identity() {
        let mut repo = MockPalimpsesteRepository::new();
        repo.expect_identity_user().returning(|_, _, _| Ok(None));
        repo.expect_create_registration()
            .withf(|_, issuer, subject, wanted, label, ttl| {
                (issuer, subject, wanted, label)
                    == ("https://issuer.test", "ana", "Ana", "ana@mail.test")
                    && *ttl == REGISTRATION_TTL
            })
            .times(1)
            .returning(|_, _, _, _, _, _| Ok(()));
        repo.expect_create_session().never();
        let ended = signing_in(repo)
            .finish_sign_in(&dex(), Some(SignInId::from_cookie("id")), answer("state"))
            .await
            .unwrap();
        assert!(matches!(ended, SignInEnd::Registering(_)));
    }

    #[tokio::test]
    async fn registering_creates_the_user_and_opens_a_session() {
        let token = RegistrationToken::from_cookie("token");
        let mut repo = MockPalimpsesteRepository::new();
        repo.expect_register()
            .with(
                eq(token.hash()),
                eq(Nickname::parse("Ana").unwrap()),
                eq(Profile::default()),
            )
            .returning(|_, _, _| Ok(Some(user())));
        repo.expect_create_session()
            .withf(|user, _, _| *user == ANA.user)
            .times(1)
            .returning(|_, _, _| Ok(()));
        let (created, _) = service(repo, MockIdentityProviders::new())
            .register(&token, Nickname::parse("Ana").unwrap(), Profile::default())
            .await
            .unwrap();
        assert_eq!(created, user());

        let mut repo = MockPalimpsesteRepository::new();
        repo.expect_register().returning(|_, _, _| Ok(None));
        repo.expect_create_session().never();
        let result = service(repo, MockIdentityProviders::new())
            .register(&token, Nickname::parse("Ana").unwrap(), Profile::default())
            .await;
        assert!(matches!(result, Err(Error::NoRegistration)));
    }

    #[tokio::test]
    async fn a_registration_is_shown_with_its_provider() {
        let mut repo = MockPalimpsesteRepository::new();
        repo.expect_registration().returning(|_| {
            Ok(Some(Registration {
                issuer: "https://issuer.test".into(),
                subject: "ana".into(),
                label: "ana@mail.test".into(),
                nickname: Nickname::parse("Ana").unwrap(),
            }))
        });
        let mut providers = MockIdentityProviders::new();
        providers.expect_provider_of().returning(|_| {
            Some(ProviderSummary {
                id: dex(),
                name: "Dex".into(),
            })
        });
        let pending = service(repo, providers)
            .registration(&RegistrationToken::from_cookie("token"))
            .await
            .unwrap();
        assert_eq!(pending.registration.nickname.as_str(), "Ana");
        assert_eq!(pending.provider.unwrap().name, "Dex");

        let mut repo = MockPalimpsesteRepository::new();
        repo.expect_registration().returning(|_| Ok(None));
        let result = service(repo, MockIdentityProviders::new())
            .registration(&RegistrationToken::from_cookie("token"))
            .await;
        assert!(matches!(result, Err(Error::NoRegistration)));
    }

    #[tokio::test]
    async fn linking_the_identity_of_another_user_is_refused() {
        let mut providers = MockIdentityProviders::new();
        providers
            .expect_verify()
            .returning(|_, _, _| Ok(verified()));
        let mut repo = MockPalimpsesteRepository::new();
        repo.expect_take_sign_in()
            .returning(|_| Ok(Some(pending(Some(ANA.user)))));
        repo.expect_link_identity()
            .returning(|_, _, _, _| Err(Error::IdentityTaken));
        let result = service(repo, providers)
            .finish_sign_in(&dex(), Some(SignInId::from_cookie("id")), answer("state"))
            .await;
        assert!(matches!(result, Err(Error::IdentityTaken)));
    }

    #[tokio::test]
    async fn identities_are_named_by_their_provider() {
        let mut repo = MockPalimpsesteRepository::new();
        repo.expect_identities().with(eq(ANA.user)).returning(|_| {
            Ok(vec![
                Identity {
                    issuer: "https://issuer.test".into(),
                    label: "ana@mail.test".into(),
                },
                Identity {
                    issuer: "https://gone.test".into(),
                    label: String::new(),
                },
            ])
        });
        let mut providers = MockIdentityProviders::new();
        providers.expect_provider_of().returning(|issuer| {
            (issuer == "https://issuer.test").then(|| ProviderSummary {
                id: dex(),
                name: "Dex".into(),
            })
        });
        let identities = service(repo, providers).identities(&ANA).await.unwrap();
        assert_eq!(identities[0].provider.as_ref().unwrap().name, "Dex");
        assert_eq!(identities[1].provider, None);
    }

    fn brune(engine: Engine) -> Character {
        Character {
            id: CharacterId::from(1),
            name: CharacterName::parse("Brune").unwrap(),
            engine,
            document: Document::empty(engine),
            version: Version::FIRST,
        }
    }

    #[tokio::test]
    async fn a_document_of_another_engine_is_refused_before_storing() {
        let mut repo = MockPalimpsesteRepository::new();
        repo.expect_character()
            .returning(|_, _| Ok(Some(brune(Engine::Fantasy))));
        repo.expect_replace_document().never();
        let result = service(repo, MockIdentityProviders::new())
            .replace_document(
                &ANA,
                CharacterId::from(1),
                Version::FIRST,
                Document::empty(Engine::Contemporain),
            )
            .await;
        assert!(matches!(result, Err(Error::OtherEngine)));
    }

    #[tokio::test]
    async fn a_document_of_an_unknown_character_is_not_found() {
        let mut repo = MockPalimpsesteRepository::new();
        repo.expect_character().returning(|_, _| Ok(None));
        let result = service(repo, MockIdentityProviders::new())
            .replace_document(
                &ANA,
                CharacterId::from(1),
                Version::FIRST,
                Document::empty(Engine::Fantasy),
            )
            .await;
        assert!(matches!(result, Err(Error::CharacterNotFound)));
    }

    #[tokio::test]
    async fn a_document_is_stored_at_the_expected_version() {
        let mut repo = MockPalimpsesteRepository::new();
        repo.expect_character()
            .returning(|_, _| Ok(Some(brune(Engine::Fantasy))));
        repo.expect_replace_document()
            .withf(|owner, _, expected, _| *owner == ANA.user && *expected == Version::FIRST)
            .returning(|_, _, _, _| Ok(Version::from(2)));
        let version = service(repo, MockIdentityProviders::new())
            .replace_document(
                &ANA,
                CharacterId::from(1),
                Version::FIRST,
                Document::empty(Engine::Fantasy),
            )
            .await
            .unwrap();
        assert_eq!(version, Version::from(2));
    }

    #[tokio::test]
    async fn deleting_an_unknown_character_is_not_found() {
        let mut repo = MockPalimpsesteRepository::new();
        repo.expect_delete_character().returning(|_, _| Ok(false));
        let result = service(repo, MockIdentityProviders::new())
            .delete_character(&ANA, CharacterId::from(1))
            .await;
        assert!(matches!(result, Err(Error::CharacterNotFound)));
    }
}
