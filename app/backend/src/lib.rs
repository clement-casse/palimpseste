//! Palimpseste backend: signs users in through OIDC providers
//! (ADR BE-OIDC), creates their account at their first sign-in
//! (ADR BE-FIRST-SIGN-IN), keeps their nickname, profile and identities
//! (ADR BE-USERS), and checks and stores their characters with their choices
//! documents (ADR BE-CHECKS). It never applies a rule: the rules run in the
//! browser.
//!
//! The crate is one bounded context laid out as ADR BE-LAYOUT says: the model
//! ([`User`], [`Character`], ...), the use cases ([`PalimpsesteService`])
//! and their [`Error`] here; [`PalimpsesteServiceImpl`] implements them on
//! a [`PalimpsesteRepository`] ([`SqliteRepo`]) and [`IdentityProviders`]
//! ([`OidcProviders`]); [`HttpApi`] serves them. `main.rs` composes them.

pub mod api;
mod character;
mod provider;
mod registration;
mod repo;
mod service;
mod session;
mod sign_in;
mod user;

use async_trait::async_trait;
use palimpseste_document::{Document, Engine};
use thiserror::Error as ThisError;

pub use crate::{
    api::http::HttpApi,
    character::{Character, CharacterId, CharacterName, Version},
    provider::{IdentityProviders, OidcProviders, ProviderConfig},
    registration::{PendingRegistration, REGISTRATION_TTL, Registration, RegistrationToken},
    repo::{PalimpsesteRepository, SqliteRepo},
    service::PalimpsesteServiceImpl,
    session::{SESSION_TTL, SessionToken, TokenHash},
    sign_in::{
        Authorization, LinkedIdentity, Linking, PendingSignIn, ProviderAnswer, ProviderId,
        ProviderSummary, SIGN_IN_TTL, SignInEnd, SignInId, StartedSignIn, VerifiedIdentity,
    },
    user::{Identity, NICKNAME_MAX, Nickname, PROFILE_MAX, Profile, User, UserId},
};

/// Who a use case acts for: the signed-in user. Every character and
/// profile use case reaches that user's data only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionContext {
    /// The signed-in user.
    pub user: UserId,
}

/// The use cases of the backend.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PalimpsesteService: Send + Sync + 'static {
    /// The configured providers, in their configured order.
    fn providers(&self) -> Vec<ProviderSummary>;

    /// Starts signing in with `provider`, or linking an identity at it to
    /// the user of `link` (ADR BE-IDENTITIES).
    async fn start_sign_in(
        &self,
        provider: &ProviderId,
        link: Option<ActionContext>,
    ) -> Result<StartedSignIn, Error>;

    /// Ends the sign-in in progress `sign_in` with the provider's `answer`:
    /// opens a session for the user of the identity, starts its registration
    /// when no user has it, or links the identity. The sign-in is used up
    /// either way.
    async fn finish_sign_in(
        &self,
        provider: &ProviderId,
        sign_in: Option<SignInId>,
        answer: ProviderAnswer,
    ) -> Result<SignInEnd, Error>;

    /// The registration in progress `token`, with the provider of its
    /// identity.
    async fn registration(&self, token: &RegistrationToken) -> Result<PendingRegistration, Error>;

    /// Completes the registration in progress `token`: creates its user with
    /// `nickname` and `profile`, and opens a session for them.
    async fn register(
        &self,
        token: &RegistrationToken,
        nickname: Nickname,
        profile: Profile,
    ) -> Result<(User, SessionToken), Error>;

    /// Cancels the registration in progress `token`, if it exists: no user
    /// is created and the identity stays free.
    async fn cancel_registration(&self, token: &RegistrationToken) -> Result<(), Error>;

    /// The user of a live session.
    async fn session_user(&self, token: &SessionToken) -> Result<User, Error>;

    /// Ends a session, if it exists.
    async fn sign_out(&self, token: &SessionToken) -> Result<(), Error>;

    /// Changes the given fields of the user's nickname and profile.
    async fn update_profile(
        &self,
        ctx: &ActionContext,
        nickname: Option<Nickname>,
        profile: Option<Profile>,
    ) -> Result<User, Error>;

    /// The user's identities, in the order they were linked.
    async fn identities(&self, ctx: &ActionContext) -> Result<Vec<LinkedIdentity>, Error>;

    /// The user's characters.
    async fn characters(&self, ctx: &ActionContext) -> Result<Vec<Character>, Error>;

    /// Creates a character of the user with the empty document of `engine`.
    async fn create_character(
        &self,
        ctx: &ActionContext,
        name: CharacterName,
        engine: Engine,
    ) -> Result<Character, Error>;

    /// A character of the user.
    async fn character(&self, ctx: &ActionContext, id: CharacterId) -> Result<Character, Error>;

    /// Renames a character of the user.
    async fn rename_character(
        &self,
        ctx: &ActionContext,
        id: CharacterId,
        name: CharacterName,
    ) -> Result<Character, Error>;

    /// Deletes a character of the user.
    async fn delete_character(&self, ctx: &ActionContext, id: CharacterId) -> Result<(), Error>;

    /// Replaces the document of a character of the user, if it is readable
    /// (ADR APP-READABLE), of the character's engine, and the character is at
    /// version `expected`. Gives the new version. Its rules are not checked.
    async fn replace_document(
        &self,
        ctx: &ActionContext,
        id: CharacterId,
        expected: Version,
        document: Document,
    ) -> Result<Version, Error>;
}

/// Why a use case failed. Each variant but [`Error::Underlying`] is an
/// outcome its user can react to; `Underlying` is a technical failure, for
/// whoever runs the server.
#[derive(Debug, ThisError)]
pub enum Error {
    /// No live session.
    #[error("no live session")]
    NoSession,
    /// No provider has this id.
    #[error("unknown provider")]
    UnknownProvider,
    /// Signing in, or linking an identity, failed: try again.
    #[error("signing in failed")]
    SignInFailed {
        /// Whether it was linking an identity rather than signing in.
        linking: bool,
        /// Why, for the logs only.
        #[source]
        cause: anyhow::Error,
    },
    /// The identity belongs to another user: never moved, never merged.
    #[error("identity of another user")]
    IdentityTaken,
    /// The user already has an identity at this provider: one per provider.
    #[error("an identity at this provider is already linked")]
    ProviderAlreadyLinked,
    /// No registration in progress in this browser, or it expired.
    #[error("no registration in progress")]
    NoRegistration,
    /// A blank nickname.
    #[error("empty name")]
    BlankNickname,
    /// A nickname longer than [`NICKNAME_MAX`] characters.
    #[error("nickname longer than {NICKNAME_MAX} characters")]
    NicknameTooLong,
    /// A profile longer than [`PROFILE_MAX`] characters.
    #[error("profile longer than {PROFILE_MAX} characters")]
    ProfileTooLong,
    /// Another user has the nickname, regardless of ASCII case.
    #[error("nickname taken")]
    NicknameTaken,
    /// Unknown, or not the user's.
    #[error("character not found")]
    CharacterNotFound,
    /// A blank character name.
    #[error("empty name")]
    BlankName,
    /// A document that cannot be stored (ADR BE-CHECKS), with why.
    #[error("{0}")]
    UnreadableDocument(String),
    /// A document of another engine than its character's.
    #[error("the document is of another engine than the character")]
    OtherEngine,
    /// The character is not at the version expected: it is at this one.
    #[error("document at version {0}")]
    Stale(Version),
    /// A technical failure: the database, a provider unreachable, ...
    #[error(transparent)]
    Underlying(#[from] anyhow::Error),
}
