//! The repository in one SQLite database (ADR BE-STORE), through
//! `rusqlite`. Queries block, so each runs off the async runtime, on
//! `spawn_blocking`, behind one connection.

use std::{
    path::Path,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    time::Duration,
};

use anyhow::Context as _;
use async_trait::async_trait;
use palimpseste_document::{Document, Engine};
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

use crate::{
    Character, CharacterId, CharacterName, Error, Identity, Linking, Nickname, PendingSignIn,
    Profile, ProviderId, SignInId, TokenHash, User, UserId, Version, repo::PalimpsesteRepository,
    user::nickname_candidates,
};

const SCHEMA: &str = "
    PRAGMA foreign_keys = ON;
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
";

/// Brings a database of an earlier schema to the current one, then indexes
/// it. Before nicknames and linked identities (ADR BE-USERS and
/// ADR BE-IDENTITIES), a user had a `name` (not unique), an identity no
/// label, and a sign-in nothing to link.
const MIGRATIONS: [(&str, &str, &str); 3] = [
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

const INDEXES: &str = "
    CREATE UNIQUE INDEX IF NOT EXISTS users_nickname ON users (nickname COLLATE NOCASE);
";

const USER_COLUMNS: &str = "users.id, users.nickname, users.profile";

const CHARACTER_COLUMNS: &str = "id, name, engine, document, version";

/// Users, their identities, sessions, sign-ins and characters, in one
/// SQLite database. Clones share the connection.
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
    /// [`Error::Underlying`] when the database cannot be opened or migrated.
    pub async fn open(path: &Path) -> Result<Self, Error> {
        let path = path.to_owned();
        let repo = tokio::task::spawn_blocking(move || Connection::open(path).and_then(Self::with))
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

    fn with(mut connection: Connection) -> rusqlite::Result<Self> {
        let transaction = connection.transaction()?;
        transaction.execute_batch(SCHEMA)?;
        for (table, column, migration) in MIGRATIONS {
            let current: bool = transaction.query_row(
                "SELECT EXISTS (SELECT 1 FROM pragma_table_info(?1) WHERE name = ?2)",
                params![table, column],
                |row| row.get(0),
            )?;
            if !current {
                transaction.execute_batch(migration)?;
            }
        }
        transaction.execute_batch(INDEXES)?;
        transaction.commit()?;
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
    async fn user_for_identity(
        &self,
        issuer: &str,
        subject: &str,
        wanted_nickname: &str,
        label: &str,
    ) -> Result<(User, bool), Error> {
        let (issuer, subject, wanted, label) = (
            issuer.to_owned(),
            subject.to_owned(),
            wanted_nickname.to_owned(),
            label.to_owned(),
        );
        self.call(
            "finding or creating the user of an identity",
            move |connection| {
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
                let created = known.is_none();
                let user = if let Some(known) = known {
                    transaction.execute(
                        "UPDATE identities SET label = ?1 WHERE issuer = ?2 AND subject = ?3",
                        params![label, issuer, subject],
                    )?;
                    known
                } else {
                    let nickname = free_nickname(&transaction, &wanted)?;
                    transaction.execute(
                        "INSERT INTO users (nickname) VALUES (?1)",
                        params![nickname.0],
                    )?;
                    let id = UserId(transaction.last_insert_rowid());
                    transaction.execute(
                        "INSERT INTO identities (issuer, subject, user_id, label)
                     VALUES (?1, ?2, ?3, ?4)",
                        params![issuer, subject, id.0, label],
                    )?;
                    User {
                        id,
                        nickname,
                        profile: Profile::default(),
                    }
                };
                transaction.commit()?;
                Ok((user, created))
            },
        )
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
                Some(owner) if owner != user.0 => return Ok(None),
                Some(_) => {
                    transaction.execute(
                        "UPDATE identities SET label = ?1 WHERE issuer = ?2 AND subject = ?3",
                        params![label, issuer, subject],
                    )?;
                    Linking::AlreadyYours
                }
                None => {
                    transaction.execute(
                        "INSERT INTO identities (issuer, subject, user_id, label)
                         VALUES (?1, ?2, ?3, ?4)",
                        params![issuer, subject, user.0, label],
                    )?;
                    Linking::Linked
                }
            };
            transaction.commit()?;
            Ok(Some(linked))
        })
        .await?
        .ok_or(Error::IdentityTaken)
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
        self.call("purging expired sessions and sign-ins", |connection| {
            connection.execute_batch(
                "DELETE FROM sessions WHERE expires_at <= unixepoch();
                 DELETE FROM logins WHERE expires_at <= unixepoch();",
            )
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
            let mut statement = connection.prepare(&format!(
                "SELECT {CHARACTER_COLUMNS} FROM characters WHERE user_id = ?1 ORDER BY id"
            ))?;
            statement.query_map(params![owner.0], character)?.collect()
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

    async fn sign_in(repo: &SqliteRepo, subject: &str, nickname: &str) -> User {
        repo.user_for_identity(ISSUER, subject, nickname, "")
            .await
            .unwrap()
            .0
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

    #[tokio::test]
    async fn an_identity_is_one_user() {
        let repo = SqliteRepo::in_memory();
        let (ana, created) = repo
            .user_for_identity(ISSUER, "a", "Ana", "ana@mail.test")
            .await
            .unwrap();
        assert!(created);
        assert_eq!(ana.nickname.as_str(), "Ana");
        assert_eq!(ana.profile.as_str(), "");
        let (again, created) = repo
            .user_for_identity(ISSUER, "a", "Other", "ana@new.test")
            .await
            .unwrap();
        assert!(!created);
        assert_eq!(again, ana);
        assert_eq!(
            repo.identities(ana.id).await.unwrap(),
            vec![identity(ISSUER, "ana@new.test")]
        );
        let (elsewhere, created) = repo
            .user_for_identity("https://other.test", "a", "Ana", "")
            .await
            .unwrap();
        assert!(created);
        assert_ne!(elsewhere.id, ana.id);
        assert_ne!(sign_in(&repo, "b", "Ben").await.id, ana.id);
    }

    #[tokio::test]
    async fn new_users_get_a_free_nickname() {
        let repo = SqliteRepo::in_memory();
        let mut nicknames = Vec::new();
        for (subject, wanted) in ["Ana", "ana", "ANA", "Ben"].iter().enumerate() {
            nicknames.push(
                sign_in(&repo, &subject.to_string(), wanted)
                    .await
                    .nickname
                    .0,
            );
        }
        assert_eq!(nicknames, ["Ana", "ana 2", "ANA 3", "Ben"]);
        let long = sign_in(&repo, "long", &"x".repeat(40)).await.nickname;
        assert_eq!(long.0, "x".repeat(28));
    }

    #[tokio::test]
    async fn profiles_are_updated_with_a_free_nickname() {
        let repo = SqliteRepo::in_memory();
        let ana = sign_in(&repo, "a", "Ana").await;
        let ben = sign_in(&repo, "b", "Ben").await;
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
        let ana = sign_in(&repo, "a", "Ana").await;
        let ben = sign_in(&repo, "b", "Ben").await;
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
        assert_eq!(
            repo.identities(ana.id).await.unwrap(),
            vec![identity(ISSUER, ""), identity(other, "ana@renamed")]
        );
        assert_eq!(
            repo.identities(ben.id).await.unwrap(),
            vec![identity(ISSUER, "")]
        );
        let (signed_in, created) = repo
            .user_for_identity(other, "a2", "Someone", "")
            .await
            .unwrap();
        assert!(!created);
        assert_eq!(signed_in, ana);
    }

    #[tokio::test]
    async fn sessions_resolve_until_deleted_or_expired() {
        let repo = SqliteRepo::in_memory();
        let ana = sign_in(&repo, "a", "Ana").await;
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
        let ana = sign_in(&repo, "a", "Ana").await;
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
        let ana = sign_in(&repo, "a", "Ana").await;
        let ben = sign_in(&repo, "b", "Ben").await;
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
        let ana = sign_in(&repo, "a", "Ana").await;
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
        let ana = sign_in(&repo, "a", "Ana").await;
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
            let (ana, created) = repo.user_for_identity(ISSUER, "a", "X", "").await.unwrap();
            assert!(!created);
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
    async fn data_outlives_the_repository() {
        let path =
            std::env::temp_dir().join(format!("palimpseste-store-{}.sqlite3", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let (ana, brune) = {
            let repo = SqliteRepo::open(&path).await.unwrap();
            let ana = sign_in(&repo, "a", "Ana").await;
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
