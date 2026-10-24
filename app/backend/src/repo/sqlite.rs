//! The repository in one SQLite database (ADR BE-STORE), through
//! `rusqlite`. Queries block, so each runs off the async runtime, on
//! `spawn_blocking`, behind one connection. The schema is versioned by
//! `PRAGMA user_version` and migrated when the database opens.

use std::{
    path::Path,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    time::Duration,
};

use anyhow::{Context as _, anyhow};
use async_trait::async_trait;
use palimpseste_document::{Document, Engine};
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

use crate::{
    Character, CharacterId, CharacterName, Error, Friend, FriendshipId, Identity, Invitation,
    InvitationId, Linking, Nickname, PendingSignIn, Profile, ProviderId, Registration, SignInId,
    TokenHash, UnixTime, User, UserId, Version, friend::pair, repo::PalimpsesteRepository,
    user::nickname_candidates,
};

/// The tables of schema version 1, created where missing.
const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY,
        nickname TEXT NOT NULL,
        profile TEXT NOT NULL DEFAULT ''
    );
    CREATE TABLE IF NOT EXISTS characters (
        id INTEGER PRIMARY KEY,
        user_id INTEGER NOT NULL REFERENCES users (id),
        name TEXT NOT NULL,
        engine TEXT NOT NULL,
        document TEXT NOT NULL,
        version INTEGER NOT NULL
    );
    CREATE TABLE IF NOT EXISTS identities (
        issuer TEXT NOT NULL,
        subject TEXT NOT NULL,
        user_id INTEGER NOT NULL REFERENCES users (id),
        label TEXT NOT NULL DEFAULT '',
        PRIMARY KEY (issuer, subject)
    );
    CREATE TABLE IF NOT EXISTS sessions (
        token_hash TEXT PRIMARY KEY,
        user_id INTEGER NOT NULL REFERENCES users (id),
        expires_at INTEGER NOT NULL
    );
    CREATE TABLE IF NOT EXISTS logins (
        id TEXT PRIMARY KEY,
        provider TEXT NOT NULL,
        state TEXT NOT NULL,
        nonce TEXT NOT NULL,
        pkce_verifier TEXT NOT NULL,
        expires_at INTEGER NOT NULL,
        link_user INTEGER REFERENCES users (id)
    );
    CREATE TABLE IF NOT EXISTS registrations (
        token_hash TEXT PRIMARY KEY,
        issuer TEXT NOT NULL,
        subject TEXT NOT NULL,
        label TEXT NOT NULL,
        nickname TEXT NOT NULL,
        expires_at INTEGER NOT NULL
    );
";

/// Brings a database older than the versioned schema to version 1, each
/// migration when its column is missing, before indexing it. Before
/// nicknames and linked identities (ADR BE-USERS and ADR BE-IDENTITIES), a
/// user had a `name` (not unique), an identity no label, and a sign-in
/// nothing to link.
const UNVERSIONED_MIGRATIONS: [(&str, &str, &str); 3] = [
    (
        "users",
        "profile",
        "ALTER TABLE users RENAME COLUMN name TO nickname;
         ALTER TABLE users ADD COLUMN profile TEXT NOT NULL DEFAULT '';
         UPDATE users SET nickname = nickname || ' ' || id
         WHERE id NOT IN (SELECT MIN(id) FROM users GROUP BY nickname COLLATE NOCASE);",
    ),
    (
        "identities",
        "label",
        "ALTER TABLE identities ADD COLUMN label TEXT NOT NULL DEFAULT '';",
    ),
    (
        "logins",
        "link_user",
        "ALTER TABLE logins ADD COLUMN link_user INTEGER REFERENCES users (id);",
    ),
];

/// The indexes of schema version 1, created where missing.
const INDEXES: &str = "
    CREATE UNIQUE INDEX IF NOT EXISTS users_nickname ON users (nickname COLLATE NOCASE);
    CREATE UNIQUE INDEX IF NOT EXISTS identities_user_issuer ON identities (user_id, issuer);
";

/// The schema changes after version 1, in order: `MIGRATIONS[i]` brings a
/// database at version `i + 1` to version `i + 2`. A change to the schema
/// appends one; a migration once released is never edited. They run in the
/// opening transaction with foreign keys enforced, which SQLite does not let
/// a migration turn off: rebuilding a table others reference (`users`)
/// first makes `SqliteRepo::with` turn them off around `migrate`, and check
/// them with `PRAGMA foreign_key_check` before the commit.
const MIGRATIONS: [&str; 2] = [
    // 2: invitations and friendships (ADR BE-INVITATIONS, ADR BE-FRIENDS).
    // One friendship per pair of users, the lower id first.
    "CREATE TABLE invitations (
         id INTEGER PRIMARY KEY,
         token_hash TEXT NOT NULL UNIQUE,
         user_id INTEGER NOT NULL REFERENCES users (id),
         expires_at INTEGER NOT NULL
     );
     CREATE INDEX invitations_user ON invitations (user_id);
     CREATE TABLE friendships (
         id INTEGER PRIMARY KEY,
         user_low INTEGER NOT NULL REFERENCES users (id),
         user_high INTEGER NOT NULL REFERENCES users (id),
         CHECK (user_low < user_high),
         UNIQUE (user_low, user_high)
     );
     CREATE INDEX friendships_high ON friendships (user_high);",
    // 3: the choices document moves to version 2, which reads no version 1
    // (ADR APP-DOCUMENT); the characters stored before are deleted, the
    // accounts, identities and friendships kept (ADR BE-STORE).
    "DELETE FROM characters;",
];

const USER_COLUMNS: &str = "users.id, users.nickname, users.profile";

const CHARACTER_COLUMNS: &str = "id, name, engine, document, version";

/// The friendships of user `?1`, each with its `id` and the other user as
/// `friend_id`.
const FRIENDSHIPS_OF: &str = "SELECT id,
           CASE WHEN user_low = ?1 THEN user_high ELSE user_low END AS friend_id
    FROM friendships WHERE user_low = ?1 OR user_high = ?1";

/// Users, their identities, sessions, sign-ins, registrations, characters,
/// invitations and friendships, in one SQLite database. Clones share the
/// connection.
#[derive(Debug, Clone)]
pub struct SqliteRepo {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteRepo {
    /// Opens the database at `path`, creating it and its tables if needed
    /// and migrating an earlier schema.
    ///
    /// # Errors
    ///
    /// [`Error::Underlying`] when the database cannot be opened or migrated,
    /// or its schema is newer than this server's.
    pub async fn open(path: &Path) -> Result<Self, Error> {
        let path = path.to_owned();
        let repo = tokio::task::spawn_blocking(move || Self::with(Connection::open(path)?))
            .await
            .context("opening the database off the async runtime")?
            .context("opening and migrating the database")?;
        Ok(repo)
    }

    /// A database in memory, lost when the last clone is dropped.
    #[cfg(test)]
    pub(crate) fn in_memory() -> Self {
        Self::with(Connection::open_in_memory().unwrap()).unwrap()
    }

    /// The repository on `connection`, its foreign keys enforced and its
    /// schema migrated.
    fn with(mut connection: Connection) -> anyhow::Result<Self> {
        // Outside any transaction, where SQLite would ignore it.
        connection.pragma_update(None, "foreign_keys", true)?;
        migrate(&mut connection, &MIGRATIONS)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    /// Runs `query` on the connection, off the async runtime; `what` says
    /// what it does, for the logs.
    async fn call<T: Send + 'static>(
        &self,
        what: &'static str,
        query: impl FnOnce(&mut Connection) -> rusqlite::Result<T> + Send + 'static,
    ) -> Result<T, Error> {
        let connection = Arc::clone(&self.connection);
        let result = tokio::task::spawn_blocking(move || query(&mut lock(&connection)))
            .await
            .context(what)?;
        Ok(result.context(what)?)
    }
}

/// Brings the database to the last version `migrations` lead to, in one
/// transaction: from no version (a new database, or one older than the
/// versioned schema) to version 1, then through each migration after the
/// version it is at. A database at a later version than `migrations` know is
/// refused, rather than read by a server that does not know its schema.
fn migrate(connection: &mut Connection, migrations: &[&str]) -> anyhow::Result<()> {
    let transaction = connection.transaction()?;
    let stored: i64 = transaction.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let latest = migrations.len() + 1;
    let current = usize::try_from(stored)
        .ok()
        .filter(|current| *current <= latest)
        .ok_or_else(|| {
            anyhow!("database at schema version {stored}, this server knows up to {latest}")
        })?;
    if current == 0 {
        unversioned_to_first(&transaction).context("migrating the schema to version 1")?;
    }
    for (index, migration) in migrations
        .iter()
        .enumerate()
        .skip(current.saturating_sub(1))
    {
        transaction
            .execute_batch(migration)
            .with_context(|| format!("migrating the schema to version {}", index + 2))?;
    }
    transaction.pragma_update(None, "user_version", i64::try_from(latest)?)?;
    transaction.commit()?;
    Ok(())
}

/// Brings a database of no version to version 1: creates the tables and
/// indexes missing, after migrating each earlier schema whose column is
/// missing. Applied to a database already at the shape of version 1, it
/// changes nothing.
fn unversioned_to_first(transaction: &Transaction<'_>) -> rusqlite::Result<()> {
    transaction.execute_batch(SCHEMA)?;
    for (table, column, migration) in UNVERSIONED_MIGRATIONS {
        let current: bool = transaction.query_row(
            "SELECT EXISTS (SELECT 1 FROM pragma_table_info(?1) WHERE name = ?2)",
            params![table, column],
            |row| row.get(0),
        )?;
        if !current {
            transaction.execute_batch(migration)?;
        }
    }
    transaction.execute_batch(INDEXES)
}

fn lock(connection: &Mutex<Connection>) -> MutexGuard<'_, Connection> {
    // A panic never leaves a transaction half applied: SQLite rolls it
    // back, so a poisoned lock is safe to reuse.
    connection.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What replacing a document did, inside its transaction.
enum Replaced {
    Stored(Version),
    NotFound,
    Stale(Version),
}

#[async_trait]
impl PalimpsesteRepository for SqliteRepo {
    async fn identity_user(
        &self,
        issuer: &str,
        subject: &str,
        label: &str,
    ) -> Result<Option<User>, Error> {
        let (issuer, subject, label) = (issuer.to_owned(), subject.to_owned(), label.to_owned());
        self.call("finding the user of an identity", move |connection| {
            let transaction = connection.transaction()?;
            let known = transaction
                .query_row(
                    &format!(
                        "SELECT {USER_COLUMNS} FROM identities
                         JOIN users ON users.id = identities.user_id
                         WHERE identities.issuer = ?1 AND identities.subject = ?2"
                    ),
                    params![issuer, subject],
                    user,
                )
                .optional()?;
            if known.is_some() {
                transaction.execute(
                    "UPDATE identities SET label = ?1 WHERE issuer = ?2 AND subject = ?3",
                    params![label, issuer, subject],
                )?;
            }
            transaction.commit()?;
            Ok(known)
        })
        .await
    }

    async fn create_registration(
        &self,
        token: &TokenHash,
        issuer: &str,
        subject: &str,
        wanted_nickname: &str,
        label: &str,
        ttl: Duration,
    ) -> Result<(), Error> {
        let (token, issuer, subject, wanted, label) = (
            token.0.clone(),
            issuer.to_owned(),
            subject.to_owned(),
            wanted_nickname.to_owned(),
            label.to_owned(),
        );
        self.call("keeping a registration in progress", move |connection| {
            let transaction = connection.transaction()?;
            let nickname = free_nickname(&transaction, &wanted)?;
            transaction.execute(
                "INSERT INTO registrations (token_hash, issuer, subject, label, nickname, expires_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, unixepoch() + ?6)",
                params![token, issuer, subject, label, nickname.0, seconds(ttl)],
            )?;
            transaction.commit()
        })
        .await
    }

    async fn registration(&self, token: &TokenHash) -> Result<Option<Registration>, Error> {
        let token = token.0.clone();
        self.call("reading a registration in progress", move |connection| {
            live_registration(connection, &token)
        })
        .await
    }

    async fn register(
        &self,
        token: &TokenHash,
        nickname: Nickname,
        profile: Profile,
    ) -> Result<Option<User>, Error> {
        let token = token.0.clone();
        self.call("completing a registration", move |connection| {
            let transaction = connection.transaction()?;
            let Some(registration) = live_registration(&transaction, &token)? else {
                return Ok(Ok(None));
            };
            if nickname_taken(&transaction, &nickname.0, None)? {
                return Ok(Err(Error::NicknameTaken));
            }
            let owned: bool = transaction.query_row(
                "SELECT EXISTS (SELECT 1 FROM identities WHERE issuer = ?1 AND subject = ?2)",
                params![registration.issuer, registration.subject],
                |row| row.get(0),
            )?;
            transaction.execute(
                "DELETE FROM registrations WHERE token_hash = ?1",
                params![token],
            )?;
            if owned {
                transaction.commit()?;
                return Ok(Err(Error::IdentityTaken));
            }
            transaction.execute(
                "INSERT INTO users (nickname, profile) VALUES (?1, ?2)",
                params![nickname.0, profile.0],
            )?;
            let id = UserId(transaction.last_insert_rowid());
            transaction.execute(
                "INSERT INTO identities (issuer, subject, user_id, label) VALUES (?1, ?2, ?3, ?4)",
                params![
                    registration.issuer,
                    registration.subject,
                    id.0,
                    registration.label
                ],
            )?;
            transaction.commit()?;
            Ok(Ok(Some(User {
                id,
                nickname,
                profile,
            })))
        })
        .await?
    }

    async fn delete_registration(&self, token: &TokenHash) -> Result<(), Error> {
        let token = token.0.clone();
        self.call("cancelling a registration", move |connection| {
            connection.execute(
                "DELETE FROM registrations WHERE token_hash = ?1",
                params![token],
            )?;
            Ok(())
        })
        .await
    }

    async fn update_user(
        &self,
        id: UserId,
        nickname: Option<Nickname>,
        profile: Option<Profile>,
    ) -> Result<User, Error> {
        self.call("updating a user", move |connection| {
            let transaction = connection.transaction()?;
            if let Some(nickname) = nickname {
                if nickname_taken(&transaction, &nickname.0, Some(id))? {
                    return Ok(None);
                }
                transaction.execute(
                    "UPDATE users SET nickname = ?1 WHERE id = ?2",
                    params![nickname.0, id.0],
                )?;
            }
            if let Some(profile) = profile {
                transaction.execute(
                    "UPDATE users SET profile = ?1 WHERE id = ?2",
                    params![profile.0, id.0],
                )?;
            }
            let user = transaction.query_row(
                &format!("SELECT {USER_COLUMNS} FROM users WHERE id = ?1"),
                params![id.0],
                user,
            )?;
            transaction.commit()?;
            Ok(Some(user))
        })
        .await?
        .ok_or(Error::NicknameTaken)
    }

    async fn identities(&self, user: UserId) -> Result<Vec<Identity>, Error> {
        self.call("listing the identities of a user", move |connection| {
            let mut statement = connection.prepare(
                "SELECT issuer, label FROM identities WHERE user_id = ?1 ORDER BY rowid",
            )?;
            statement
                .query_map(params![user.0], |row| {
                    Ok(Identity {
                        issuer: row.get(0)?,
                        label: row.get(1)?,
                    })
                })?
                .collect()
        })
        .await
    }

    async fn link_identity(
        &self,
        user: UserId,
        issuer: &str,
        subject: &str,
        label: &str,
    ) -> Result<Linking, Error> {
        let (issuer, subject, label) = (issuer.to_owned(), subject.to_owned(), label.to_owned());
        self.call("linking an identity", move |connection| {
            let transaction = connection.transaction()?;
            let owner: Option<i64> = transaction
                .query_row(
                    "SELECT user_id FROM identities WHERE issuer = ?1 AND subject = ?2",
                    params![issuer, subject],
                    |row| row.get(0),
                )
                .optional()?;
            let linked = match owner {
                Some(owner) if owner != user.0 => return Ok(Err(Error::IdentityTaken)),
                Some(_) => {
                    transaction.execute(
                        "UPDATE identities SET label = ?1 WHERE issuer = ?2 AND subject = ?3",
                        params![label, issuer, subject],
                    )?;
                    Linking::AlreadyYours
                }
                None => {
                    let at_issuer: bool = transaction.query_row(
                        "SELECT EXISTS (SELECT 1 FROM identities WHERE user_id = ?1 AND issuer = ?2)",
                        params![user.0, issuer],
                        |row| row.get(0),
                    )?;
                    if at_issuer {
                        return Ok(Err(Error::ProviderAlreadyLinked));
                    }
                    transaction.execute(
                        "INSERT INTO identities (issuer, subject, user_id, label)
                         VALUES (?1, ?2, ?3, ?4)",
                        params![issuer, subject, user.0, label],
                    )?;
                    Linking::Linked
                }
            };
            transaction.commit()?;
            Ok(Ok(linked))
        })
        .await?
    }

    async fn create_session(
        &self,
        user: UserId,
        token: &TokenHash,
        ttl: Duration,
    ) -> Result<(), Error> {
        let token = token.0.clone();
        self.call("opening a session", move |connection| {
            connection.execute(
                "INSERT INTO sessions (token_hash, user_id, expires_at)
                 VALUES (?1, ?2, unixepoch() + ?3)",
                params![token, user.0, seconds(ttl)],
            )?;
            Ok(())
        })
        .await
    }

    async fn session_user(&self, token: &TokenHash) -> Result<Option<User>, Error> {
        let token = token.0.clone();
        self.call("finding the user of a session", move |connection| {
            connection
                .query_row(
                    &format!(
                        "SELECT {USER_COLUMNS} FROM sessions
                         JOIN users ON users.id = sessions.user_id
                         WHERE sessions.token_hash = ?1 AND sessions.expires_at > unixepoch()"
                    ),
                    params![token],
                    user,
                )
                .optional()
        })
        .await
    }

    async fn delete_session(&self, token: &TokenHash) -> Result<(), Error> {
        let token = token.0.clone();
        self.call("ending a session", move |connection| {
            connection.execute("DELETE FROM sessions WHERE token_hash = ?1", params![token])?;
            Ok(())
        })
        .await
    }

    async fn create_sign_in(
        &self,
        id: &SignInId,
        sign_in: &PendingSignIn,
        ttl: Duration,
    ) -> Result<(), Error> {
        let (id, sign_in) = (id.as_str().to_owned(), sign_in.clone());
        self.call("keeping a sign-in in progress", move |connection| {
            connection.execute(
                "INSERT INTO logins (id, provider, state, nonce, pkce_verifier, link_user, expires_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, unixepoch() + ?7)",
                params![
                    id,
                    sign_in.provider.as_str(),
                    sign_in.state,
                    sign_in.nonce,
                    sign_in.pkce_verifier,
                    sign_in.link.map(|user| user.0),
                    seconds(ttl)
                ],
            )?;
            Ok(())
        })
        .await
    }

    async fn take_sign_in(&self, id: &SignInId) -> Result<Option<PendingSignIn>, Error> {
        let id = id.as_str().to_owned();
        let taken = self
            .call("taking a sign-in in progress", move |connection| {
                connection
                    .query_row(
                        "DELETE FROM logins WHERE id = ?1
                         RETURNING provider, state, nonce, pkce_verifier, link_user,
                                   expires_at > unixepoch()",
                        params![id],
                        |row| {
                            let sign_in = PendingSignIn {
                                provider: ProviderId::from(row.get::<_, String>(0)?),
                                state: row.get(1)?,
                                nonce: row.get(2)?,
                                pkce_verifier: row.get(3)?,
                                link: row.get::<_, Option<i64>>(4)?.map(UserId),
                            };
                            Ok((sign_in, row.get::<_, bool>(5)?))
                        },
                    )
                    .optional()
            })
            .await?;
        Ok(taken.and_then(|(sign_in, live)| live.then_some(sign_in)))
    }

    async fn purge_expired(&self) -> Result<(), Error> {
        self.call(
            "purging expired sessions, sign-ins, registrations and invitations",
            |connection| {
                connection.execute_batch(
                    "DELETE FROM sessions WHERE expires_at <= unixepoch();
                 DELETE FROM logins WHERE expires_at <= unixepoch();
                 DELETE FROM registrations WHERE expires_at <= unixepoch();
                 DELETE FROM invitations WHERE expires_at <= unixepoch();",
                )
            },
        )
        .await
    }

    async fn ping(&self) -> Result<(), Error> {
        // Reads the schema page of the file, through the shared connection.
        self.call("checking the database", |connection| {
            connection.query_row("SELECT count(*) FROM sqlite_schema", [], |_| Ok(()))
        })
        .await
    }

    async fn create_character(
        &self,
        owner: UserId,
        name: &CharacterName,
        document: &Document,
    ) -> Result<Character, Error> {
        let (name, document) = (name.clone(), document.clone());
        self.call("creating a character", move |connection| {
            connection.execute(
                "INSERT INTO characters (user_id, name, engine, document, version)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    owner.0,
                    name.0,
                    engine_name(document.engine()),
                    to_json(&document)?,
                    Version::FIRST.get()
                ],
            )?;
            Ok(Character {
                id: CharacterId::from(connection.last_insert_rowid()),
                name,
                engine: document.engine(),
                document,
                version: Version::FIRST,
            })
        })
        .await
    }

    async fn characters(&self, owner: UserId) -> Result<Vec<Character>, Error> {
        self.call("listing the characters of a user", move |connection| {
            owned_characters(connection, owner)
        })
        .await
    }

    async fn character(&self, owner: UserId, id: CharacterId) -> Result<Option<Character>, Error> {
        self.call("reading a character", move |connection| {
            find_character(connection, owner, id)
        })
        .await
    }

    async fn rename_character(
        &self,
        owner: UserId,
        id: CharacterId,
        name: &CharacterName,
    ) -> Result<Option<Character>, Error> {
        let name = name.clone();
        self.call("renaming a character", move |connection| {
            let changed = connection.execute(
                "UPDATE characters SET name = ?1 WHERE id = ?2 AND user_id = ?3",
                params![name.0, id.get(), owner.0],
            )?;
            if changed == 0 {
                return Ok(None);
            }
            find_character(connection, owner, id)
        })
        .await
    }

    async fn delete_character(&self, owner: UserId, id: CharacterId) -> Result<bool, Error> {
        self.call("deleting a character", move |connection| {
            let deleted = connection.execute(
                "DELETE FROM characters WHERE id = ?1 AND user_id = ?2",
                params![id.get(), owner.0],
            )?;
            Ok(deleted > 0)
        })
        .await
    }

    async fn replace_document(
        &self,
        owner: UserId,
        id: CharacterId,
        expected: Version,
        document: &Document,
    ) -> Result<Version, Error> {
        let document = document.clone();
        let replaced = self
            .call("replacing a document", move |connection| {
                let transaction = connection.transaction()?;
                let current = transaction
                    .query_row(
                        "SELECT version FROM characters WHERE id = ?1 AND user_id = ?2",
                        params![id.get(), owner.0],
                        |row| row.get::<_, i64>(0),
                    )
                    .optional()?
                    .map(Version::from);
                let Some(version) = current else {
                    return Ok(Replaced::NotFound);
                };
                if version != expected {
                    return Ok(Replaced::Stale(version));
                }
                let next = version.next();
                transaction.execute(
                    "UPDATE characters SET document = ?1, version = ?2 WHERE id = ?3",
                    params![to_json(&document)?, next.get(), id.get()],
                )?;
                transaction.commit()?;
                Ok(Replaced::Stored(next))
            })
            .await?;
        match replaced {
            Replaced::Stored(version) => Ok(version),
            Replaced::NotFound => Err(Error::CharacterNotFound),
            Replaced::Stale(version) => Err(Error::Stale(version)),
        }
    }

    async fn create_invitation(
        &self,
        inviter: UserId,
        token: &TokenHash,
        ttl: Duration,
    ) -> Result<Invitation, Error> {
        let token = token.0.clone();
        self.call("creating an invitation", move |connection| {
            connection.query_row(
                "INSERT INTO invitations (token_hash, user_id, expires_at)
                 VALUES (?1, ?2, unixepoch() + ?3)
                 RETURNING id, expires_at",
                params![token, inviter.0, seconds(ttl)],
                invitation,
            )
        })
        .await
    }

    async fn invitations(&self, inviter: UserId) -> Result<Vec<Invitation>, Error> {
        self.call("listing the invitations of a user", move |connection| {
            let mut statement = connection.prepare(
                "SELECT id, expires_at FROM invitations
                 WHERE user_id = ?1 AND expires_at > unixepoch()
                 ORDER BY expires_at, id",
            )?;
            statement
                .query_map(params![inviter.0], invitation)?
                .collect()
        })
        .await
    }

    async fn delete_invitation(&self, inviter: UserId, id: InvitationId) -> Result<bool, Error> {
        self.call("revoking an invitation", move |connection| {
            let deleted = connection.execute(
                "DELETE FROM invitations WHERE id = ?1 AND user_id = ?2",
                params![id.get(), inviter.0],
            )?;
            Ok(deleted > 0)
        })
        .await
    }

    async fn invitation_inviter(&self, token: &TokenHash) -> Result<Option<User>, Error> {
        let token = token.0.clone();
        self.call("finding the sender of an invitation", move |connection| {
            connection
                .query_row(
                    &format!(
                        "SELECT {USER_COLUMNS} FROM invitations
                         JOIN users ON users.id = invitations.user_id
                         WHERE invitations.token_hash = ?1
                           AND invitations.expires_at > unixepoch()"
                    ),
                    params![token],
                    user,
                )
                .optional()
        })
        .await
    }

    async fn friendship_between(
        &self,
        one: UserId,
        other: UserId,
    ) -> Result<Option<FriendshipId>, Error> {
        self.call("finding a friendship", move |connection| {
            find_friendship(connection, one, other)
        })
        .await
    }

    async fn accept_invitation(&self, user: UserId, token: &TokenHash) -> Result<Friend, Error> {
        let token = token.0.clone();
        self.call("accepting an invitation", move |connection| {
            let transaction = connection.transaction()?;
            let inviter = transaction
                .query_row(
                    "SELECT user_id FROM invitations
                     WHERE token_hash = ?1 AND expires_at > unixepoch()",
                    params![token],
                    |row| row.get(0).map(UserId),
                )
                .optional()?;
            let Some(inviter) = inviter else {
                return Ok(Err(Error::InvitationNotFound));
            };
            if inviter == user {
                return Ok(Err(Error::OwnInvitation));
            }
            if find_friendship(&transaction, user, inviter)?.is_some() {
                return Ok(Err(Error::AlreadyFriends));
            }
            transaction.execute(
                "DELETE FROM invitations WHERE token_hash = ?1",
                params![token],
            )?;
            let (low, high) = pair(user, inviter);
            transaction.execute(
                "INSERT INTO friendships (user_low, user_high) VALUES (?1, ?2)",
                params![low.0, high.0],
            )?;
            let friendship = FriendshipId::from(transaction.last_insert_rowid());
            let friend = transaction.query_row(
                "SELECT ?1, nickname, profile FROM users WHERE id = ?2",
                params![friendship.get(), inviter.0],
                friend,
            )?;
            transaction.commit()?;
            Ok(Ok(friend))
        })
        .await?
    }

    async fn friends(&self, user: UserId) -> Result<Vec<Friend>, Error> {
        self.call("listing the friends of a user", move |connection| {
            let mut statement = connection.prepare(&format!(
                "SELECT mine.id, users.nickname, users.profile
                 FROM ({FRIENDSHIPS_OF}) AS mine JOIN users ON users.id = mine.friend_id
                 ORDER BY users.nickname COLLATE NOCASE, mine.id"
            ))?;
            statement.query_map(params![user.0], friend)?.collect()
        })
        .await
    }

    async fn friend(&self, user: UserId, id: FriendshipId) -> Result<Option<Friend>, Error> {
        self.call("reading a friend", move |connection| {
            connection
                .query_row(
                    &format!(
                        "SELECT mine.id, users.nickname, users.profile
                         FROM ({FRIENDSHIPS_OF}) AS mine JOIN users ON users.id = mine.friend_id
                         WHERE mine.id = ?2"
                    ),
                    params![user.0, id.get()],
                    friend,
                )
                .optional()
        })
        .await
    }

    async fn delete_friendship(&self, user: UserId, id: FriendshipId) -> Result<bool, Error> {
        self.call("ending a friendship", move |connection| {
            let deleted = connection.execute(
                "DELETE FROM friendships WHERE id = ?2 AND (user_low = ?1 OR user_high = ?1)",
                params![user.0, id.get()],
            )?;
            Ok(deleted > 0)
        })
        .await
    }

    async fn friend_characters(
        &self,
        user: UserId,
        id: FriendshipId,
    ) -> Result<Option<Vec<Character>>, Error> {
        self.call("listing the characters of a friend", move |connection| {
            let friend = connection
                .query_row(
                    &format!("SELECT friend_id FROM ({FRIENDSHIPS_OF}) WHERE id = ?2"),
                    params![user.0, id.get()],
                    |row| row.get(0).map(UserId),
                )
                .optional()?;
            friend
                .map(|friend| owned_characters(connection, friend))
                .transpose()
        })
        .await
    }

    async fn friend_character(
        &self,
        user: UserId,
        id: FriendshipId,
        character: CharacterId,
    ) -> Result<Option<Character>, Error> {
        self.call("reading a character of a friend", move |connection| {
            connection
                .query_row(
                    &format!(
                        "SELECT {CHARACTER_COLUMNS} FROM characters
                         WHERE id = ?3
                           AND user_id = (SELECT friend_id FROM ({FRIENDSHIPS_OF}) WHERE id = ?2)"
                    ),
                    params![user.0, id.get(), character.get()],
                    self::character,
                )
                .optional()
        })
        .await
    }
}

/// `ttl` in whole seconds, for SQL.
fn seconds(ttl: Duration) -> i64 {
    i64::try_from(ttl.as_secs()).unwrap_or(i64::MAX)
}

const fn engine_name(engine: Engine) -> &'static str {
    match engine {
        Engine::Contemporain => "contemporain",
        Engine::Fantasy => "fantasy",
    }
}

/// Whether a user other than `except` has `nickname`, regardless of ASCII
/// case.
fn nickname_taken(
    transaction: &Transaction<'_>,
    nickname: &str,
    except: Option<UserId>,
) -> rusqlite::Result<bool> {
    transaction.query_row(
        "SELECT EXISTS (SELECT 1 FROM users WHERE nickname = ?1 COLLATE NOCASE AND id IS NOT ?2)",
        params![nickname, except.map(|user| user.0)],
        |row| row.get(0),
    )
}

/// The first nickname candidate of `wanted` no user has.
fn free_nickname(transaction: &Transaction<'_>, wanted: &str) -> rusqlite::Result<Nickname> {
    for candidate in nickname_candidates(wanted) {
        if !nickname_taken(transaction, &candidate.0, None)? {
            return Ok(candidate);
        }
    }
    unreachable!("nickname candidates never end")
}

/// The registration `token`, unless it is unknown or expired.
fn live_registration(
    connection: &Connection,
    token: &str,
) -> rusqlite::Result<Option<Registration>> {
    connection
        .query_row(
            "SELECT issuer, subject, label, nickname FROM registrations
             WHERE token_hash = ?1 AND expires_at > unixepoch()",
            params![token],
            |row| {
                Ok(Registration {
                    issuer: row.get(0)?,
                    subject: row.get(1)?,
                    label: row.get(2)?,
                    nickname: Nickname(row.get(3)?),
                })
            },
        )
        .optional()
}

fn find_character(
    connection: &Connection,
    owner: UserId,
    id: CharacterId,
) -> rusqlite::Result<Option<Character>> {
    connection
        .query_row(
            &format!("SELECT {CHARACTER_COLUMNS} FROM characters WHERE id = ?1 AND user_id = ?2"),
            params![id.get(), owner.0],
            character,
        )
        .optional()
}

/// The characters of `owner`, in the order they were created.
fn owned_characters(connection: &Connection, owner: UserId) -> rusqlite::Result<Vec<Character>> {
    let mut statement = connection.prepare(&format!(
        "SELECT {CHARACTER_COLUMNS} FROM characters WHERE user_id = ?1 ORDER BY id"
    ))?;
    statement.query_map(params![owner.0], character)?.collect()
}

/// The friendship of `one` and `other`, if they are friends.
fn find_friendship(
    connection: &Connection,
    one: UserId,
    other: UserId,
) -> rusqlite::Result<Option<FriendshipId>> {
    let (low, high) = pair(one, other);
    connection
        .query_row(
            "SELECT id FROM friendships WHERE user_low = ?1 AND user_high = ?2",
            params![low.0, high.0],
            |row| row.get::<_, i64>(0).map(FriendshipId::from),
        )
        .optional()
}

fn invitation(row: &Row<'_>) -> rusqlite::Result<Invitation> {
    Ok(Invitation {
        id: InvitationId::from(row.get::<_, i64>(0)?),
        expires_at: UnixTime(row.get(1)?),
    })
}

fn friend(row: &Row<'_>) -> rusqlite::Result<Friend> {
    Ok(Friend {
        friendship: FriendshipId::from(row.get::<_, i64>(0)?),
        nickname: Nickname(row.get(1)?),
        profile: Profile(row.get(2)?),
    })
}

fn user(row: &Row<'_>) -> rusqlite::Result<User> {
    Ok(User {
        id: UserId(row.get(0)?),
        nickname: Nickname(row.get(1)?),
        profile: Profile(row.get(2)?),
    })
}

fn character(row: &Row<'_>) -> rusqlite::Result<Character> {
    let document: Document = from_json(3, &row.get::<_, String>(3)?)?;
    Ok(Character {
        id: CharacterId::from(row.get::<_, i64>(0)?),
        name: CharacterName(row.get(1)?),
        engine: document.engine(),
        document,
        version: Version::from(row.get::<_, i64>(4)?),
    })
}

fn to_json(document: &Document) -> rusqlite::Result<String> {
    serde_json::to_string(document)
        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(error.into()))
}

fn from_json(column: usize, json: &str) -> rusqlite::Result<Document> {
    serde_json::from_str(json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(column, rusqlite::types::Type::Text, error.into())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SESSION_TTL, SessionToken};

    const ISSUER: &str = "https://issuer.test";
    const HOUR: Duration = Duration::from_hours(1);

    /// Registers the identity `subject` at the issuer, with the nickname
    /// suggested for `wanted`.
    async fn sign_up(repo: &SqliteRepo, subject: &str, wanted: &str) -> User {
        let token = token(&format!("registration {subject}"));
        repo.create_registration(&token, ISSUER, subject, wanted, "", HOUR)
            .await
            .unwrap();
        let suggested = repo.registration(&token).await.unwrap().unwrap().nickname;
        repo.register(&token, suggested, Profile::default())
            .await
            .unwrap()
            .unwrap()
    }

    fn pending(provider: &str, link: Option<UserId>) -> PendingSignIn {
        PendingSignIn {
            provider: provider.into(),
            state: "state".into(),
            nonce: "nonce".into(),
            pkce_verifier: "verifier".into(),
            link,
        }
    }

    fn identity(issuer: &str, label: &str) -> Identity {
        Identity {
            issuer: issuer.into(),
            label: label.into(),
        }
    }

    fn token(name: &str) -> TokenHash {
        SessionToken::from_cookie(name).hash()
    }

    fn nickname(value: &str) -> Nickname {
        Nickname::parse(value).unwrap()
    }

    fn schema_version(connection: &Connection) -> i64 {
        connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap()
    }

    /// The version `SqliteRepo` migrates a database to.
    fn last_version() -> i64 {
        i64::try_from(MIGRATIONS.len()).unwrap() + 1
    }

    #[test]
    fn a_new_database_is_at_the_last_version() {
        let repo = SqliteRepo::in_memory();
        assert_eq!(schema_version(&lock(&repo.connection)), last_version());
    }

    #[test]
    fn migrations_run_once_in_order() {
        let first = ["CREATE TABLE notes (id INTEGER PRIMARY KEY)"];
        let second = [first[0], "ALTER TABLE notes ADD COLUMN text TEXT"];
        // From version 1, where `open` leaves a database today: one
        // migration at a time, then both in one opening.
        let one_by_one: [&[&str]; 3] = [&[], &first, &second];
        let at_once: [&[&str]; 2] = [&[], &second];
        for steps in [&one_by_one[..], &at_once[..]] {
            let mut connection = Connection::open_in_memory().unwrap();
            for migrations in steps {
                // Run again, a migration would fail: its table or column exists.
                migrate(&mut connection, migrations).unwrap();
                migrate(&mut connection, migrations).unwrap();
                assert_eq!(
                    schema_version(&connection),
                    i64::try_from(migrations.len()).unwrap() + 1
                );
            }
            connection
                .execute("INSERT INTO notes (text) VALUES ('kept')", [])
                .unwrap();
        }
    }

    #[test]
    fn a_failed_migration_changes_nothing() {
        let mut connection = Connection::open_in_memory().unwrap();
        let failing = ["CREATE TABLE notes (id INTEGER PRIMARY KEY)", "NOT SQL"];
        let error = migrate(&mut connection, &failing).unwrap_err();
        assert!(error.to_string().contains("version 3"), "{error}");
        assert_eq!(schema_version(&connection), 0);
        let tables: i64 = connection
            .query_row("SELECT COUNT(*) FROM sqlite_schema", [], |row| row.get(0))
            .unwrap();
        assert_eq!(tables, 0);
    }

    #[test]
    fn a_database_newer_than_the_server_is_refused() {
        let mut connection = Connection::open_in_memory().unwrap();
        migrate(
            &mut connection,
            &["CREATE TABLE notes (id INTEGER PRIMARY KEY)"],
        )
        .unwrap();
        let error = migrate(&mut connection, &[]).unwrap_err();
        assert!(error.to_string().contains("schema version 2"), "{error}");
        assert_eq!(schema_version(&connection), 2);
    }

    #[test]
    fn foreign_keys_are_enforced() {
        let connection = Connection::open_in_memory().unwrap();
        // Whatever the default of the SQLite build.
        connection
            .pragma_update(None, "foreign_keys", false)
            .unwrap();
        let repo = SqliteRepo::with(connection).unwrap();
        let orphan = lock(&repo.connection)
            .execute(
                "INSERT INTO sessions (token_hash, user_id, expires_at) VALUES ('t', 9, 0)",
                [],
            )
            .unwrap_err();
        assert!(orphan.to_string().contains("FOREIGN KEY"), "{orphan}");
    }

    #[tokio::test]
    async fn an_open_database_answers() {
        SqliteRepo::in_memory().ping().await.unwrap();
    }

    #[tokio::test]
    async fn an_identity_is_registered_once() {
        let repo = SqliteRepo::in_memory();
        let ana_token = token("ana");
        assert_eq!(repo.identity_user(ISSUER, "a", "").await.unwrap(), None);
        repo.create_registration(&ana_token, ISSUER, "a", "Ana", "ana@mail.test", HOUR)
            .await
            .unwrap();
        assert_eq!(
            repo.registration(&ana_token).await.unwrap(),
            Some(Registration {
                issuer: ISSUER.into(),
                subject: "a".into(),
                label: "ana@mail.test".into(),
                nickname: nickname("Ana"),
            })
        );
        // A registration creates no user until completed.
        assert_eq!(repo.identity_user(ISSUER, "a", "").await.unwrap(), None);
        let ana = repo
            .register(&ana_token, nickname("Anaïs"), Profile("Joue.".into()))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(ana.nickname.as_str(), "Anaïs");
        assert_eq!(ana.profile.as_str(), "Joue.");
        assert_eq!(repo.registration(&ana_token).await.unwrap(), None);
        assert_eq!(
            repo.register(&ana_token, nickname("Again"), Profile::default())
                .await
                .unwrap(),
            None
        );
        assert_eq!(
            repo.identity_user(ISSUER, "a", "ana@new.test")
                .await
                .unwrap(),
            Some(ana.clone())
        );
        assert_eq!(
            repo.identities(ana.id).await.unwrap(),
            vec![identity(ISSUER, "ana@new.test")]
        );
        assert_eq!(
            repo.identity_user("https://other.test", "a", "")
                .await
                .unwrap(),
            None
        );
    }

    #[tokio::test]
    async fn a_registration_waits_for_a_free_nickname() {
        let repo = SqliteRepo::in_memory();
        let ana = sign_up(&repo, "a", "Ana").await;
        let ben_token = token("ben");
        repo.create_registration(&ben_token, ISSUER, "b", "Ben", "", HOUR)
            .await
            .unwrap();
        assert!(matches!(
            repo.register(&ben_token, nickname("ANA"), Profile::default())
                .await,
            Err(Error::NicknameTaken)
        ));
        let ben = repo
            .register(&ben_token, nickname("Ben"), Profile::default())
            .await
            .unwrap()
            .unwrap();
        assert_ne!(ben.id, ana.id);
    }

    #[tokio::test]
    async fn registrations_end_when_cancelled_expired_or_raced() {
        let repo = SqliteRepo::in_memory();
        let (cancelled, expired) = (token("cancelled"), token("expired"));
        repo.create_registration(&cancelled, ISSUER, "a", "Ana", "", HOUR)
            .await
            .unwrap();
        repo.create_registration(&expired, ISSUER, "b", "Ben", "", Duration::ZERO)
            .await
            .unwrap();
        repo.delete_registration(&cancelled).await.unwrap();
        repo.delete_registration(&cancelled).await.unwrap();
        for token in [&cancelled, &expired] {
            assert_eq!(repo.registration(token).await.unwrap(), None);
            assert_eq!(
                repo.register(token, nickname("X"), Profile::default())
                    .await
                    .unwrap(),
                None
            );
        }
        repo.purge_expired().await.unwrap();
        assert_eq!(repo.identity_user(ISSUER, "a", "").await.unwrap(), None);

        // Two browsers registering one identity: the second is refused.
        let (first, second) = (token("first"), token("second"));
        for token in [&first, &second] {
            repo.create_registration(token, ISSUER, "c", "Cy", "", HOUR)
                .await
                .unwrap();
        }
        repo.register(&first, nickname("Cy"), Profile::default())
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(
            repo.register(&second, nickname("Cy 2"), Profile::default())
                .await,
            Err(Error::IdentityTaken)
        ));
        assert_eq!(repo.registration(&second).await.unwrap(), None);
    }

    #[tokio::test]
    async fn new_users_get_a_free_nickname() {
        let repo = SqliteRepo::in_memory();
        let mut nicknames = Vec::new();
        for (subject, wanted) in ["Ana", "ana", "ANA", "Ben"].iter().enumerate() {
            nicknames.push(
                sign_up(&repo, &subject.to_string(), wanted)
                    .await
                    .nickname
                    .0,
            );
        }
        assert_eq!(nicknames, ["Ana", "ana 2", "ANA 3", "Ben"]);
        let long = sign_up(&repo, "long", &"x".repeat(40)).await.nickname;
        assert_eq!(long.0, "x".repeat(28));
    }

    #[tokio::test]
    async fn profiles_are_updated_with_a_free_nickname() {
        let repo = SqliteRepo::in_memory();
        let ana = sign_up(&repo, "a", "Ana").await;
        let ben = sign_up(&repo, "b", "Ben").await;
        assert!(matches!(
            repo.update_user(ben.id, Some(nickname("aNa")), Some(Profile("x".into())))
                .await,
            Err(Error::NicknameTaken)
        ));
        let renamed = repo
            .update_user(ana.id, Some(nickname("ANA")), None)
            .await
            .unwrap();
        assert_eq!(renamed.nickname.as_str(), "ANA");
        let described = repo
            .update_user(ana.id, None, Some(Profile("Joue une barbare.".into())))
            .await
            .unwrap();
        assert_eq!(described.nickname.as_str(), "ANA");
        assert_eq!(described.profile.as_str(), "Joue une barbare.");
        assert_eq!(repo.session_user(&token("none")).await.unwrap(), None);
        repo.create_session(ana.id, &token("ana"), HOUR)
            .await
            .unwrap();
        assert_eq!(
            repo.session_user(&token("ana")).await.unwrap(),
            Some(described)
        );
        assert_eq!(repo.update_user(ben.id, None, None).await.unwrap(), ben);
    }

    #[tokio::test]
    async fn identities_are_linked_to_one_user() {
        let repo = SqliteRepo::in_memory();
        let ana = sign_up(&repo, "a", "Ana").await;
        let ben = sign_up(&repo, "b", "Ben").await;
        let other = "https://other.test";
        assert_eq!(
            repo.link_identity(ana.id, other, "a2", "ana@other")
                .await
                .unwrap(),
            Linking::Linked
        );
        assert_eq!(
            repo.link_identity(ana.id, other, "a2", "ana@renamed")
                .await
                .unwrap(),
            Linking::AlreadyYours
        );
        assert!(matches!(
            repo.link_identity(ana.id, ISSUER, "b", "").await,
            Err(Error::IdentityTaken)
        ));
        // One identity per provider.
        assert!(matches!(
            repo.link_identity(ana.id, other, "a3", "").await,
            Err(Error::ProviderAlreadyLinked)
        ));
        assert_eq!(
            repo.identities(ana.id).await.unwrap(),
            vec![identity(ISSUER, ""), identity(other, "ana@renamed")]
        );
        assert_eq!(
            repo.identities(ben.id).await.unwrap(),
            vec![identity(ISSUER, "")]
        );
        assert_eq!(
            repo.identity_user(other, "a2", "ana@renamed")
                .await
                .unwrap(),
            Some(ana)
        );
    }

    #[tokio::test]
    async fn sessions_resolve_until_deleted_or_expired() {
        let repo = SqliteRepo::in_memory();
        let ana = sign_up(&repo, "a", "Ana").await;
        repo.create_session(ana.id, &token("live"), SESSION_TTL)
            .await
            .unwrap();
        repo.create_session(ana.id, &token("expired"), Duration::ZERO)
            .await
            .unwrap();
        assert_eq!(repo.session_user(&token("live")).await.unwrap(), Some(ana));
        assert_eq!(repo.session_user(&token("expired")).await.unwrap(), None);
        assert_eq!(repo.session_user(&token("unknown")).await.unwrap(), None);
        repo.purge_expired().await.unwrap();
        repo.delete_session(&token("live")).await.unwrap();
        assert_eq!(repo.session_user(&token("live")).await.unwrap(), None);
    }

    #[tokio::test]
    async fn a_sign_in_is_taken_once_and_before_it_expires() {
        let repo = SqliteRepo::in_memory();
        let ana = sign_up(&repo, "a", "Ana").await;
        let id = |name: &str| SignInId::from_cookie(name);
        repo.create_sign_in(&id("live"), &pending("dex", None), HOUR)
            .await
            .unwrap();
        repo.create_sign_in(&id("link"), &pending("dex", Some(ana.id)), HOUR)
            .await
            .unwrap();
        repo.create_sign_in(&id("expired"), &pending("dex", None), Duration::ZERO)
            .await
            .unwrap();
        assert_eq!(
            repo.take_sign_in(&id("live")).await.unwrap(),
            Some(pending("dex", None))
        );
        assert_eq!(
            repo.take_sign_in(&id("link")).await.unwrap(),
            Some(pending("dex", Some(ana.id)))
        );
        assert_eq!(repo.take_sign_in(&id("live")).await.unwrap(), None);
        assert_eq!(repo.take_sign_in(&id("expired")).await.unwrap(), None);
        assert_eq!(repo.take_sign_in(&id("unknown")).await.unwrap(), None);
    }

    fn brune_name() -> CharacterName {
        CharacterName::parse("Brune").unwrap()
    }

    #[tokio::test]
    async fn characters_belong_to_their_user() {
        let repo = SqliteRepo::in_memory();
        let ana = sign_up(&repo, "a", "Ana").await;
        let ben = sign_up(&repo, "b", "Ben").await;
        let fantasy = Document::empty(Engine::Fantasy);
        let brune = repo
            .create_character(ana.id, &brune_name(), &fantasy)
            .await
            .unwrap();
        let max = repo
            .create_character(
                ben.id,
                &CharacterName::parse("Max").unwrap(),
                &Document::empty(Engine::Contemporain),
            )
            .await
            .unwrap();
        assert_ne!(brune.id, max.id);
        assert_eq!(brune.engine, Engine::Fantasy);
        assert_eq!(brune.version, Version::FIRST);
        assert_eq!(repo.characters(ana.id).await.unwrap(), vec![brune.clone()]);
        assert_eq!(repo.characters(ben.id).await.unwrap(), vec![max]);

        assert_eq!(repo.character(ben.id, brune.id).await.unwrap(), None);
        assert_eq!(
            repo.rename_character(ben.id, brune.id, &brune_name())
                .await
                .unwrap(),
            None
        );
        assert!(matches!(
            repo.replace_document(ben.id, brune.id, Version::FIRST, &fantasy)
                .await,
            Err(Error::CharacterNotFound)
        ));
        assert!(!repo.delete_character(ben.id, brune.id).await.unwrap());
        assert_eq!(repo.character(ana.id, brune.id).await.unwrap(), Some(brune));
    }

    #[tokio::test]
    async fn documents_are_replaced_at_the_expected_version_only() {
        let repo = SqliteRepo::in_memory();
        let ana = sign_up(&repo, "a", "Ana").await;
        let brune = repo
            .create_character(ana.id, &brune_name(), &Document::empty(Engine::Fantasy))
            .await
            .unwrap();
        let mut document = palimpseste_document::FantasyDocument::empty();
        document.profil = Some("cof.profil.guerrier".into());
        let document = Document::Fantasy(document);

        assert_eq!(
            repo.replace_document(ana.id, brune.id, Version::FIRST, &document)
                .await
                .unwrap(),
            Version::from(2)
        );
        assert!(matches!(
            repo.replace_document(ana.id, brune.id, Version::FIRST, &document)
                .await,
            Err(Error::Stale(version)) if version == Version::from(2)
        ));
        assert!(matches!(
            repo.replace_document(ana.id, CharacterId::from(9), Version::FIRST, &document)
                .await,
            Err(Error::CharacterNotFound)
        ));
        let stored = repo.character(ana.id, brune.id).await.unwrap().unwrap();
        assert_eq!(stored.document, document);
        assert_eq!(stored.version, Version::from(2));
    }

    #[tokio::test]
    async fn characters_are_renamed_and_deleted() {
        let repo = SqliteRepo::in_memory();
        let ana = sign_up(&repo, "a", "Ana").await;
        let brune = repo
            .create_character(ana.id, &brune_name(), &Document::empty(Engine::Fantasy))
            .await
            .unwrap();
        let new_name = CharacterName::parse("Bruna").unwrap();
        let renamed = repo
            .rename_character(ana.id, brune.id, &new_name)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(renamed.name, new_name);
        assert!(repo.delete_character(ana.id, brune.id).await.unwrap());
        assert!(!repo.delete_character(ana.id, brune.id).await.unwrap());
        assert_eq!(
            repo.rename_character(ana.id, brune.id, &new_name)
                .await
                .unwrap(),
            None
        );
        assert_eq!(repo.characters(ana.id).await.unwrap(), vec![]);
    }

    #[tokio::test]
    async fn invitations_wait_until_revoked_or_expired() {
        let repo = SqliteRepo::in_memory();
        let ana = sign_up(&repo, "a", "Ana").await;
        let ben = sign_up(&repo, "b", "Ben").await;
        let week = repo
            .create_invitation(ana.id, &token("week"), HOUR * 24 * 7)
            .await
            .unwrap();
        let hour = repo
            .create_invitation(ana.id, &token("hour"), HOUR)
            .await
            .unwrap();
        repo.create_invitation(ana.id, &token("expired"), Duration::ZERO)
            .await
            .unwrap();
        assert!(hour.expires_at < week.expires_at);
        assert_eq!(
            repo.invitations(ana.id).await.unwrap(),
            vec![hour.clone(), week.clone()]
        );
        assert_eq!(repo.invitations(ben.id).await.unwrap(), vec![]);
        assert_eq!(
            repo.invitation_inviter(&token("week")).await.unwrap(),
            Some(ana.clone())
        );
        for unknown in ["expired", "none"] {
            assert_eq!(
                repo.invitation_inviter(&token(unknown)).await.unwrap(),
                None
            );
        }

        assert!(!repo.delete_invitation(ben.id, week.id).await.unwrap());
        assert!(repo.delete_invitation(ana.id, week.id).await.unwrap());
        assert!(!repo.delete_invitation(ana.id, week.id).await.unwrap());
        assert_eq!(repo.invitation_inviter(&token("week")).await.unwrap(), None);
        let kept = || -> i64 {
            lock(&repo.connection)
                .query_row("SELECT COUNT(*) FROM invitations", [], |row| row.get(0))
                .unwrap()
        };
        // Hidden once expired, removed by the purge.
        assert_eq!(kept(), 2);
        repo.purge_expired().await.unwrap();
        assert_eq!(kept(), 1);
        assert_eq!(repo.invitations(ana.id).await.unwrap(), vec![hour]);
    }

    #[tokio::test]
    async fn an_accepted_invitation_makes_friends_of_both() {
        let repo = SqliteRepo::in_memory();
        let ana = sign_up(&repo, "a", "Ana").await;
        let ben = sign_up(&repo, "b", "Ben").await;
        let cy = sign_up(&repo, "c", "Cy").await;
        for name in ["first", "second", "expired"] {
            let ttl = if name == "expired" {
                Duration::ZERO
            } else {
                HOUR
            };
            repo.create_invitation(ben.id, &token(name), ttl)
                .await
                .unwrap();
        }
        // Their sender cannot use it, and it stays for someone else.
        assert!(matches!(
            repo.accept_invitation(ben.id, &token("first")).await,
            Err(Error::OwnInvitation)
        ));
        for unknown in ["expired", "none"] {
            assert!(matches!(
                repo.accept_invitation(ana.id, &token(unknown)).await,
                Err(Error::InvitationNotFound)
            ));
        }
        let ben_for_ana = repo
            .accept_invitation(ana.id, &token("first"))
            .await
            .unwrap();
        assert_eq!(ben_for_ana.nickname, ben.nickname);
        // Used up.
        assert!(matches!(
            repo.accept_invitation(cy.id, &token("first")).await,
            Err(Error::InvitationNotFound)
        ));
        assert_eq!(
            repo.invitation_inviter(&token("first")).await.unwrap(),
            None
        );
        assert_eq!(repo.invitations(ben.id).await.unwrap().len(), 1);
        // Friends already: the second invitation stays for someone else.
        assert!(matches!(
            repo.accept_invitation(ana.id, &token("second")).await,
            Err(Error::AlreadyFriends)
        ));
        let friendship = ben_for_ana.friendship;
        let ana_for_ben = Friend {
            friendship,
            nickname: ana.nickname.clone(),
            profile: ana.profile.clone(),
        };
        assert_eq!(
            repo.friends(ana.id).await.unwrap(),
            vec![ben_for_ana.clone()]
        );
        assert_eq!(
            repo.friends(ben.id).await.unwrap(),
            vec![ana_for_ben.clone()]
        );
        assert_eq!(repo.friends(cy.id).await.unwrap(), vec![]);
        assert_eq!(
            repo.friend(ben.id, friendship).await.unwrap(),
            Some(ana_for_ben)
        );
        assert_eq!(repo.friend(cy.id, friendship).await.unwrap(), None);
        for (one, other) in [(ana.id, ben.id), (ben.id, ana.id)] {
            assert_eq!(
                repo.friendship_between(one, other).await.unwrap(),
                Some(friendship)
            );
        }
        assert_eq!(repo.friendship_between(ana.id, cy.id).await.unwrap(), None);

        // Cy accepts the second one: Ben's friends, by nickname.
        repo.accept_invitation(cy.id, &token("second"))
            .await
            .unwrap();
        let nicknames: Vec<String> = repo
            .friends(ben.id)
            .await
            .unwrap()
            .into_iter()
            .map(|friend| friend.nickname.0)
            .collect();
        assert_eq!(nicknames, ["Ana", "Cy"]);
    }

    #[tokio::test]
    async fn either_friend_ends_the_friendship_for_both() {
        let repo = SqliteRepo::in_memory();
        let ana = sign_up(&repo, "a", "Ana").await;
        let ben = sign_up(&repo, "b", "Ben").await;
        let cy = sign_up(&repo, "c", "Cy").await;
        repo.create_invitation(ana.id, &token("ana"), HOUR)
            .await
            .unwrap();
        let friendship = repo
            .accept_invitation(ben.id, &token("ana"))
            .await
            .unwrap()
            .friendship;
        assert!(!repo.delete_friendship(cy.id, friendship).await.unwrap());
        assert!(repo.delete_friendship(ben.id, friendship).await.unwrap());
        assert!(!repo.delete_friendship(ana.id, friendship).await.unwrap());
        assert_eq!(repo.friends(ana.id).await.unwrap(), vec![]);
        assert_eq!(repo.friends(ben.id).await.unwrap(), vec![]);
        assert_eq!(repo.friendship_between(ana.id, ben.id).await.unwrap(), None);
        // Friends again through a new invitation.
        repo.create_invitation(ben.id, &token("ben"), HOUR)
            .await
            .unwrap();
        repo.accept_invitation(ana.id, &token("ben")).await.unwrap();
        assert!(
            repo.friendship_between(ana.id, ben.id)
                .await
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn friends_read_each_others_characters_only() {
        let repo = SqliteRepo::in_memory();
        let ana = sign_up(&repo, "a", "Ana").await;
        let ben = sign_up(&repo, "b", "Ben").await;
        let cy = sign_up(&repo, "c", "Cy").await;
        let fantasy = Document::empty(Engine::Fantasy);
        let brune = repo
            .create_character(ana.id, &brune_name(), &fantasy)
            .await
            .unwrap();
        let max = repo
            .create_character(ben.id, &CharacterName::parse("Max").unwrap(), &fantasy)
            .await
            .unwrap();
        let cyd = repo
            .create_character(cy.id, &CharacterName::parse("Cyd").unwrap(), &fantasy)
            .await
            .unwrap();
        repo.create_invitation(ana.id, &token("ana"), HOUR)
            .await
            .unwrap();
        let friendship = repo
            .accept_invitation(ben.id, &token("ana"))
            .await
            .unwrap()
            .friendship;

        assert_eq!(
            repo.friend_characters(ben.id, friendship).await.unwrap(),
            Some(vec![brune.clone()])
        );
        assert_eq!(
            repo.friend_characters(ana.id, friendship).await.unwrap(),
            Some(vec![max.clone()])
        );
        assert_eq!(
            repo.friend_character(ben.id, friendship, brune.id)
                .await
                .unwrap(),
            Some(brune.clone())
        );
        // Not one's own character, nor a third user's, through a friendship.
        for character in [max.id, cyd.id] {
            assert_eq!(
                repo.friend_character(ben.id, friendship, character)
                    .await
                    .unwrap(),
                None
            );
        }
        // Not through someone else's friendship.
        assert_eq!(
            repo.friend_characters(cy.id, friendship).await.unwrap(),
            None
        );
        assert_eq!(
            repo.friend_character(cy.id, friendship, brune.id)
                .await
                .unwrap(),
            None
        );
        // Reading never writes.
        assert_eq!(repo.character(ben.id, brune.id).await.unwrap(), None);

        repo.delete_friendship(ana.id, friendship).await.unwrap();
        assert_eq!(
            repo.friend_characters(ben.id, friendship).await.unwrap(),
            None
        );
        assert_eq!(
            repo.friend_character(ben.id, friendship, brune.id)
                .await
                .unwrap(),
            None
        );
    }

    #[tokio::test]
    async fn migrates_a_database_of_version_1() {
        let mut connection = Connection::open_in_memory().unwrap();
        migrate(&mut connection, &MIGRATIONS[..0]).unwrap();
        assert_eq!(schema_version(&connection), 1);
        let repo = SqliteRepo {
            connection: Arc::new(Mutex::new(connection)),
        };
        let ana = sign_up(&repo, "a", "Ana").await;
        let ben = sign_up(&repo, "b", "Ben").await;
        migrate(&mut lock(&repo.connection), &MIGRATIONS).unwrap();
        assert_eq!(schema_version(&lock(&repo.connection)), last_version());
        repo.create_invitation(ana.id, &token("ana"), HOUR)
            .await
            .unwrap();
        let ana_for_ben = repo.accept_invitation(ben.id, &token("ana")).await.unwrap();
        assert_eq!(ana_for_ben.nickname, ana.nickname);
    }

    #[tokio::test]
    async fn migrating_to_version_3_deletes_the_characters_only() {
        let mut connection = Connection::open_in_memory().unwrap();
        migrate(&mut connection, &MIGRATIONS[..1]).unwrap();
        assert_eq!(schema_version(&connection), 2);
        let repo = SqliteRepo {
            connection: Arc::new(Mutex::new(connection)),
        };
        let ana = sign_up(&repo, "a", "Ana").await;
        repo.create_character(ana.id, &brune_name(), &Document::empty(Engine::Fantasy))
            .await
            .unwrap();
        migrate(&mut lock(&repo.connection), &MIGRATIONS).unwrap();
        assert_eq!(schema_version(&lock(&repo.connection)), last_version());
        assert_eq!(repo.characters(ana.id).await.unwrap(), []);
        // The account stays, with its identity.
        assert_eq!(repo.identities(ana.id).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn migrates_a_database_of_adr_0009() {
        let path = std::env::temp_dir().join(format!(
            "palimpseste-store-migration-{}.sqlite3",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        Connection::open(&path)
            .unwrap()
            .execute_batch(
                "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);
                 CREATE TABLE identities (
                     issuer TEXT NOT NULL, subject TEXT NOT NULL,
                     user_id INTEGER NOT NULL REFERENCES users (id),
                     PRIMARY KEY (issuer, subject));
                 CREATE TABLE logins (
                     id TEXT PRIMARY KEY, provider TEXT NOT NULL, state TEXT NOT NULL,
                     nonce TEXT NOT NULL, pkce_verifier TEXT NOT NULL,
                     expires_at INTEGER NOT NULL);
                 INSERT INTO users (name) VALUES ('Ana'), ('Ben'), ('ana');
                 INSERT INTO identities VALUES ('https://issuer.test', 'a', 1);",
            )
            .unwrap();
        for _ in 0..2 {
            let repo = SqliteRepo::open(&path).await.unwrap();
            assert_eq!(schema_version(&lock(&repo.connection)), last_version());
            let ana = repo.identity_user(ISSUER, "a", "").await.unwrap().unwrap();
            assert_eq!(ana.nickname.as_str(), "Ana");
            assert_eq!(
                repo.identities(ana.id).await.unwrap(),
                vec![identity(ISSUER, "")]
            );
            for taken in ["ana 3", "Ben"] {
                assert!(matches!(
                    repo.update_user(ana.id, Some(nickname(taken)), None).await,
                    Err(Error::NicknameTaken)
                ));
            }
        }
        std::fs::remove_file(&path).unwrap();
    }

    #[tokio::test]
    async fn a_database_from_before_the_versioning_keeps_its_accounts() {
        let path = std::env::temp_dir().join(format!(
            "palimpseste-store-unversioned-{}.sqlite3",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let (ana, brune) = {
            // The schema of version 1, as opened before the version was kept.
            let mut connection = Connection::open(&path).unwrap();
            migrate(&mut connection, &[]).unwrap();
            connection.pragma_update(None, "user_version", 0).unwrap();
            let repo = SqliteRepo {
                connection: Arc::new(Mutex::new(connection)),
            };
            let ana = sign_up(&repo, "a", "Ana").await;
            let brune = repo
                .create_character(ana.id, &brune_name(), &Document::empty(Engine::Fantasy))
                .await
                .unwrap();
            (ana, brune)
        };
        let reopened = SqliteRepo::open(&path).await.unwrap();
        assert_eq!(schema_version(&lock(&reopened.connection)), last_version());
        // The account stays; the character goes with version 3.
        assert_eq!(reopened.identities(ana.id).await.unwrap().len(), 1);
        assert_eq!(reopened.character(ana.id, brune.id).await.unwrap(), None);
        drop(reopened);
        std::fs::remove_file(&path).unwrap();
    }

    #[tokio::test]
    async fn data_outlives_the_repository() {
        let path =
            std::env::temp_dir().join(format!("palimpseste-store-{}.sqlite3", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let (ana, brune) = {
            let repo = SqliteRepo::open(&path).await.unwrap();
            let ana = sign_up(&repo, "a", "Ana").await;
            repo.create_session(ana.id, &token("token"), HOUR)
                .await
                .unwrap();
            let brune = repo
                .create_character(ana.id, &brune_name(), &Document::empty(Engine::Fantasy))
                .await
                .unwrap();
            (ana, brune)
        };
        let reopened = SqliteRepo::open(&path).await.unwrap();
        assert_eq!(
            reopened.character(ana.id, brune.id).await.unwrap(),
            Some(brune)
        );
        assert_eq!(
            reopened.session_user(&token("token")).await.unwrap(),
            Some(ana)
        );
        drop(reopened);
        std::fs::remove_file(&path).unwrap();
    }
}
