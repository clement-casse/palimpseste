//! What the service keeps: users, identities, sessions, sign-ins in
//! progress and characters. The repository keeps what it is given and
//! never reads the rules; it decides only what needs a transaction.

pub mod sqlite;

use std::time::Duration;

use async_trait::async_trait;
use palimpseste_document::Document;

pub use self::sqlite::SqliteRepo;
use crate::{
    Character, CharacterId, CharacterName, Error, Identity, Linking, Nickname, PendingSignIn,
    Profile, SignInId, TokenHash, User, UserId, Version,
};

/// The persistence the service needs. Every character is reached through
/// its owner.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PalimpsesteRepository: Send + Sync + 'static {
    /// The user of the identity `subject` at `issuer`, created on its first
    /// sign-in with the first free of the nickname candidates of
    /// `wanted_nickname`. The identity's label becomes `label`. Also tells
    /// whether the user was created.
    async fn user_for_identity(
        &self,
        issuer: &str,
        subject: &str,
        wanted_nickname: &str,
        label: &str,
    ) -> Result<(User, bool), Error>;

    /// Changes the given fields of `user`; [`Error::NicknameTaken`] when
    /// another user has the nickname.
    async fn update_user(
        &self,
        user: UserId,
        nickname: Option<Nickname>,
        profile: Option<Profile>,
    ) -> Result<User, Error>;

    /// The identities of `user`, in the order they were linked.
    async fn identities(&self, user: UserId) -> Result<Vec<Identity>, Error>;

    /// Links the identity `subject` at `issuer` to `user`, labelled
    /// `label`; [`Error::IdentityTaken`] when it is another user's.
    async fn link_identity(
        &self,
        user: UserId,
        issuer: &str,
        subject: &str,
        label: &str,
    ) -> Result<Linking, Error>;

    /// Opens a session of `user` for `ttl`, known by its hash.
    async fn create_session(
        &self,
        user: UserId,
        token: &TokenHash,
        ttl: Duration,
    ) -> Result<(), Error>;

    /// The user of a session, unless it is unknown or expired.
    async fn session_user(&self, token: &TokenHash) -> Result<Option<User>, Error>;

    /// Ends a session, if any.
    async fn delete_session(&self, token: &TokenHash) -> Result<(), Error>;

    /// Keeps a sign-in in progress under `id` for `ttl`.
    async fn create_sign_in(
        &self,
        id: &SignInId,
        sign_in: &PendingSignIn,
        ttl: Duration,
    ) -> Result<(), Error>;

    /// Removes the sign-in in progress `id` and gives it back, unless it is
    /// unknown or expired: a sign-in is completed once at most.
    async fn take_sign_in(&self, id: &SignInId) -> Result<Option<PendingSignIn>, Error>;

    /// Removes the expired sessions and sign-ins.
    async fn purge_expired(&self) -> Result<(), Error>;

    /// Creates a character of `owner` at [`Version::FIRST`].
    async fn create_character(
        &self,
        owner: UserId,
        name: &CharacterName,
        document: &Document,
    ) -> Result<Character, Error>;

    /// The characters of `owner`.
    async fn characters(&self, owner: UserId) -> Result<Vec<Character>, Error>;

    /// Character `id`, unless it is unknown or not `owner`'s.
    async fn character(&self, owner: UserId, id: CharacterId) -> Result<Option<Character>, Error>;

    /// Renames `owner`'s character `id`; `None` when it is unknown or not
    /// theirs.
    async fn rename_character(
        &self,
        owner: UserId,
        id: CharacterId,
        name: &CharacterName,
    ) -> Result<Option<Character>, Error>;

    /// Deletes `owner`'s character `id`; `false` when it is unknown or not
    /// theirs.
    async fn delete_character(&self, owner: UserId, id: CharacterId) -> Result<bool, Error>;

    /// Replaces the document of `owner`'s character `id` if it is at
    /// version `expected`, and gives the next version;
    /// [`Error::Stale`] or [`Error::CharacterNotFound`] otherwise.
    async fn replace_document(
        &self,
        owner: UserId,
        id: CharacterId,
        expected: Version,
        document: &Document,
    ) -> Result<Version, Error>;
}
