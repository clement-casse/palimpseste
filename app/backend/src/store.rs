//! Storage of users and their characters in SQLite (ADR 0007, D7), and of
//! what signing in needs (ADR 0009, ADR 0010): identities, sessions and
//! sign-ins in progress.
//!
//! A user is known inside the application by a private id, and to others by
//! a nickname, unique regardless of ASCII case, and a profile; they sign in
//! with any of their identities. The store keeps what it is given:
//! nicknames, profiles, names, engines and choices documents.
//! It never reads the rules; the API checks that a document is readable
//! before storing it. A character is reached through its owner only. Methods
//! block, so the API calls them from `spawn_blocking`.

use std::{
    fmt,
    path::Path,
    sync::{Mutex, MutexGuard, PoisonError},
    time::Duration,
};

use palimpseste_document::{Document, Engine};
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};
use serde::{Deserialize, Serialize};

/// Identifier of a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(i64);

/// Identifier of a character, unique across all users.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CharacterId(i64);

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for CharacterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A user, owner of a list of characters. Their id is private: it never
/// leaves the server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct User {
    #[serde(skip)]
    pub id: UserId,
    /// Public, unique regardless of ASCII case.
    pub nickname: String,
    /// Public free text, empty until written.
    pub profile: String,
}

/// An identity a user signs in with, as its owner sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    /// The issuer of the identity, naming its provider.
    pub issuer: String,
    /// Private: what the provider last said the identity is (an email, a
    /// username), so its owner tells their identities apart; may be empty.
    pub label: String,
}

/// What changing a user's nickname or profile did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileUpdate {
    /// Changed: the user as they now are.
    Updated(User),
    /// Unchanged: another user has the nickname.
    NicknameTaken,
}

/// What linking an identity to a user did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Linking {
    /// Linked: the user can now sign in with it.
    Linked,
    /// Already one of the user's identities.
    AlreadyYours,
    /// Unchanged: the identity belongs to another user.
    Taken,
}

/// A character of a user, tied to its engine, with its choices document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Character {
    pub id: CharacterId,
    pub name: String,
    pub engine: Engine,
    pub document: Document,
    /// Incremented by each replacement of the document; the `ETag`.
    pub version: i64,
}

/// What replacing a document did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Replaced {
    /// Stored, with its new version.
    Stored(i64),
    /// No such character.
    NotFound,
    /// The character's version is not the one expected: it is given.
    Stale(i64),
    /// The document is of another engine than the character's.
    OtherEngine,
}

/// A sign-in in progress, between the redirect to a provider and its
/// callback (ADR 0009).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingLogin {
    /// The id of the provider signed in with.
    pub provider: String,
    /// The `state` sent to the provider, to be given back.
    pub state: String,
    /// The `nonce` the ID token must hold.
    pub nonce: String,
    /// The PKCE verifier of the challenge sent to the provider.
    pub pkce_verifier: String,
    /// The user the identity is linked to, when linking one rather than
    /// signing in (ADR 0010).
    pub link: Option<UserId>,
}

/// An error of the database, or a stored document it cannot read back.
pub type Result<T> = rusqlite::Result<T>;

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
/// it. Before ADR 0010, a user had a `name` (not unique), an identity no
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

/// How many characters of a wanted nickname a new user keeps, leaving room
/// for a number making it unique.
const NICKNAME_BASE_LENGTH: usize = 28;

const CHARACTER_COLUMNS: &str = "id, name, engine, document, version";

/// Users and their characters, in one SQLite database.
#[derive(Debug)]
pub struct Store {
    connection: Mutex<Connection>,
}

impl Store {
    /// Opens the database at `path`, creating it and its tables if needed.
    pub fn open(path: &Path) -> Result<Self> {
        Self::with(Connection::open(path)?)
    }

    /// A database in memory, lost when dropped.
    #[cfg(test)]
    pub fn in_memory() -> Result<Self> {
        Self::with(Connection::open_in_memory()?)
    }

    fn with(mut connection: Connection) -> Result<Self> {
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
            connection: Mutex::new(connection),
        })
    }

    /// The user signed in as `subject` at `issuer`, created on their first
    /// sign-in with `nickname`, or the first free `nickname 2`, `nickname
    /// 3`, ... The identity's label becomes `label`. Also tells whether the
    /// user was created.
    pub fn user_for_identity(
        &self,
        issuer: &str,
        subject: &str,
        nickname: &str,
        label: &str,
    ) -> Result<(User, bool)> {
        let mut connection = self.lock();
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
            let nickname = free_nickname(&transaction, nickname)?;
            transaction.execute(
                "INSERT INTO users (nickname) VALUES (?1)",
                params![nickname],
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
                profile: String::new(),
            }
        };
        transaction.commit()?;
        Ok((user, created))
    }

    /// Changes the nickname and the profile of user `id`, those given,
    /// unless another user has the nickname.
    pub fn update_user(
        &self,
        id: UserId,
        nickname: Option<&str>,
        profile: Option<&str>,
    ) -> Result<ProfileUpdate> {
        let mut connection = self.lock();
        let transaction = connection.transaction()?;
        if let Some(nickname) = nickname {
            if nickname_taken(&transaction, nickname, Some(id))? {
                return Ok(ProfileUpdate::NicknameTaken);
            }
            transaction.execute(
                "UPDATE users SET nickname = ?1 WHERE id = ?2",
                params![nickname, id.0],
            )?;
        }
        if let Some(profile) = profile {
            transaction.execute(
                "UPDATE users SET profile = ?1 WHERE id = ?2",
                params![profile, id.0],
            )?;
        }
        let user = transaction.query_row(
            &format!("SELECT {USER_COLUMNS} FROM users WHERE id = ?1"),
            params![id.0],
            user,
        )?;
        transaction.commit()?;
        Ok(ProfileUpdate::Updated(user))
    }

    /// The identities of `user`, in the order they were linked.
    pub fn identities(&self, user: UserId) -> Result<Vec<Identity>> {
        let connection = self.lock();
        let mut statement = connection
            .prepare("SELECT issuer, label FROM identities WHERE user_id = ?1 ORDER BY rowid")?;
        statement
            .query_map(params![user.0], |row| {
                Ok(Identity {
                    issuer: row.get(0)?,
                    label: row.get(1)?,
                })
            })?
            .collect()
    }

    /// Links the identity `subject` at `issuer` to `user`, labelled `label`,
    /// unless it belongs to another user.
    pub fn link_identity(
        &self,
        user: UserId,
        issuer: &str,
        subject: &str,
        label: &str,
    ) -> Result<Linking> {
        let mut connection = self.lock();
        let transaction = connection.transaction()?;
        let owner: Option<i64> = transaction
            .query_row(
                "SELECT user_id FROM identities WHERE issuer = ?1 AND subject = ?2",
                params![issuer, subject],
                |row| row.get(0),
            )
            .optional()?;
        let linked = match owner {
            Some(owner) if owner != user.0 => return Ok(Linking::Taken),
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
        Ok(linked)
    }

    /// Opens a session of `user` for `ttl`, known by the hash of its token.
    pub fn create_session(&self, user: UserId, token_hash: &str, ttl: Duration) -> Result<()> {
        self.lock().execute(
            "INSERT INTO sessions (token_hash, user_id, expires_at)
             VALUES (?1, ?2, unixepoch() + ?3)",
            params![token_hash, user.0, seconds(ttl)],
        )?;
        Ok(())
    }

    /// The user of the session known by `token_hash`, unless it is unknown
    /// or expired.
    pub fn session_user(&self, token_hash: &str) -> Result<Option<User>> {
        self.lock()
            .query_row(
                &format!(
                    "SELECT {USER_COLUMNS} FROM sessions
                     JOIN users ON users.id = sessions.user_id
                     WHERE sessions.token_hash = ?1 AND sessions.expires_at > unixepoch()"
                ),
                params![token_hash],
                user,
            )
            .optional()
    }

    /// Ends the session known by `token_hash`, if any.
    pub fn delete_session(&self, token_hash: &str) -> Result<()> {
        self.lock().execute(
            "DELETE FROM sessions WHERE token_hash = ?1",
            params![token_hash],
        )?;
        Ok(())
    }

    /// Keeps a sign-in in progress under `id` for `ttl`.
    pub fn create_login(&self, id: &str, login: &PendingLogin, ttl: Duration) -> Result<()> {
        self.lock().execute(
            "INSERT INTO logins (id, provider, state, nonce, pkce_verifier, link_user, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, unixepoch() + ?7)",
            params![
                id,
                login.provider,
                login.state,
                login.nonce,
                login.pkce_verifier,
                login.link.map(|user| user.0),
                seconds(ttl)
            ],
        )?;
        Ok(())
    }

    /// Removes the sign-in in progress `id` and gives it back, unless it is
    /// unknown or expired: a sign-in is completed once at most.
    pub fn take_login(&self, id: &str) -> Result<Option<PendingLogin>> {
        let taken = self
            .lock()
            .query_row(
                "DELETE FROM logins WHERE id = ?1
                 RETURNING provider, state, nonce, pkce_verifier, link_user,
                           expires_at > unixepoch()",
                params![id],
                |row| {
                    let login = PendingLogin {
                        provider: row.get(0)?,
                        state: row.get(1)?,
                        nonce: row.get(2)?,
                        pkce_verifier: row.get(3)?,
                        link: row.get::<_, Option<i64>>(4)?.map(UserId),
                    };
                    Ok((login, row.get::<_, bool>(5)?))
                },
            )
            .optional()?;
        Ok(taken.and_then(|(login, live)| live.then_some(login)))
    }

    /// Removes the expired sessions and sign-ins.
    pub fn purge_expired(&self) -> Result<()> {
        self.lock().execute_batch(
            "DELETE FROM sessions WHERE expires_at <= unixepoch();
             DELETE FROM logins WHERE expires_at <= unixepoch();",
        )
    }

    /// Creates a character of `user` with the empty document of its engine.
    pub fn create_character(&self, user: UserId, name: &str, engine: Engine) -> Result<Character> {
        let document = Document::empty(engine);
        let connection = self.lock();
        connection.execute(
            "INSERT INTO characters (user_id, name, engine, document, version)
             VALUES (?1, ?2, ?3, ?4, 1)",
            params![user.0, name, engine_name(engine), to_json(&document)?],
        )?;
        Ok(Character {
            id: CharacterId(connection.last_insert_rowid()),
            name: name.to_owned(),
            engine,
            document,
            version: 1,
        })
    }

    /// The characters of `user`.
    pub fn characters(&self, user: UserId) -> Result<Vec<Character>> {
        let connection = self.lock();
        let mut statement = connection.prepare(&format!(
            "SELECT {CHARACTER_COLUMNS} FROM characters WHERE user_id = ?1 ORDER BY id"
        ))?;
        statement.query_map(params![user.0], character)?.collect()
    }

    /// Character `id`, unless it is unknown or not `owner`'s.
    pub fn character(&self, owner: UserId, id: CharacterId) -> Result<Option<Character>> {
        self.lock()
            .query_row(
                &format!(
                    "SELECT {CHARACTER_COLUMNS} FROM characters WHERE id = ?1 AND user_id = ?2"
                ),
                params![id.0, owner.0],
                character,
            )
            .optional()
    }

    /// Replaces the document of `owner`'s character `id` if its version is
    /// `expected`, and increments the version.
    pub fn replace_document(
        &self,
        owner: UserId,
        id: CharacterId,
        expected: i64,
        document: &Document,
    ) -> Result<Replaced> {
        let mut connection = self.lock();
        let transaction = connection.transaction()?;
        let current = transaction
            .query_row(
                "SELECT engine, version FROM characters WHERE id = ?1 AND user_id = ?2",
                params![id.0, owner.0],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()?;
        let Some((engine, version)) = current else {
            return Ok(Replaced::NotFound);
        };
        if engine != engine_name(document.engine()) {
            return Ok(Replaced::OtherEngine);
        }
        if version != expected {
            return Ok(Replaced::Stale(version));
        }
        let next = version + 1;
        transaction.execute(
            "UPDATE characters SET document = ?1, version = ?2 WHERE id = ?3",
            params![to_json(document)?, next, id.0],
        )?;
        transaction.commit()?;
        Ok(Replaced::Stored(next))
    }

    /// Renames `owner`'s character `id`; `None` when it is unknown or not
    /// theirs.
    pub fn rename_character(
        &self,
        owner: UserId,
        id: CharacterId,
        name: &str,
    ) -> Result<Option<Character>> {
        let changed = self.lock().execute(
            "UPDATE characters SET name = ?1 WHERE id = ?2 AND user_id = ?3",
            params![name, id.0, owner.0],
        )?;
        if changed == 0 {
            return Ok(None);
        }
        self.character(owner, id)
    }

    /// Deletes `owner`'s character `id`; `false` when it is unknown or not
    /// theirs.
    pub fn delete_character(&self, owner: UserId, id: CharacterId) -> Result<bool> {
        let deleted = self.lock().execute(
            "DELETE FROM characters WHERE id = ?1 AND user_id = ?2",
            params![id.0, owner.0],
        )?;
        Ok(deleted > 0)
    }

    fn lock(&self) -> MutexGuard<'_, Connection> {
        // A panic never leaves a transaction half applied: SQLite rolls it
        // back, so a poisoned lock is safe to reuse.
        self.connection
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
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
) -> Result<bool> {
    transaction.query_row(
        "SELECT EXISTS (SELECT 1 FROM users WHERE nickname = ?1 COLLATE NOCASE AND id IS NOT ?2)",
        params![nickname, except.map(|user| user.0)],
        |row| row.get(0),
    )
}

/// `wanted`, cut to `NICKNAME_BASE_LENGTH` characters, or the first of
/// `wanted 2`, `wanted 3`, ... no user has.
fn free_nickname(transaction: &Transaction<'_>, wanted: &str) -> Result<String> {
    let base: String = wanted.chars().take(NICKNAME_BASE_LENGTH).collect();
    let mut candidate = base.clone();
    let mut number = 1;
    while nickname_taken(transaction, &candidate, None)? {
        number += 1;
        candidate = format!("{base} {number}");
    }
    Ok(candidate)
}

fn user(row: &Row<'_>) -> Result<User> {
    Ok(User {
        id: UserId(row.get(0)?),
        nickname: row.get(1)?,
        profile: row.get(2)?,
    })
}

fn character(row: &Row<'_>) -> Result<Character> {
    let document: Document = from_json(3, &row.get::<_, String>(3)?)?;
    Ok(Character {
        id: CharacterId(row.get(0)?),
        name: row.get(1)?,
        engine: document.engine(),
        document,
        version: row.get(4)?,
    })
}

fn to_json(document: &Document) -> Result<String> {
    serde_json::to_string(document)
        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(error.into()))
}

fn from_json(column: usize, json: &str) -> Result<Document> {
    serde_json::from_str(json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(column, rusqlite::types::Type::Text, error.into())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ISSUER: &str = "https://issuer.test";
    const HOUR: Duration = Duration::from_hours(1);

    fn store() -> Store {
        Store::in_memory().unwrap()
    }

    fn sign_in(store: &Store, subject: &str, nickname: &str) -> User {
        store
            .user_for_identity(ISSUER, subject, nickname, "")
            .unwrap()
            .0
    }

    fn login(provider: &str, link: Option<UserId>) -> PendingLogin {
        PendingLogin {
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

    #[test]
    fn an_identity_is_one_user() {
        let store = store();
        let (ana, created) = store
            .user_for_identity(ISSUER, "a", "Ana", "ana@mail.test")
            .unwrap();
        assert!(created);
        assert_eq!(ana.nickname, "Ana");
        assert_eq!(ana.profile, "");
        let (again, created) = store
            .user_for_identity(ISSUER, "a", "Other", "ana@new.test")
            .unwrap();
        assert!(!created);
        assert_eq!(again, ana);
        assert_eq!(
            store.identities(ana.id).unwrap(),
            vec![identity(ISSUER, "ana@new.test")]
        );
        let (elsewhere, created) = store
            .user_for_identity("https://other.test", "a", "Ana", "")
            .unwrap();
        assert!(created);
        assert_ne!(elsewhere.id, ana.id);
        assert_ne!(sign_in(&store, "b", "Ben").id, ana.id);
    }

    #[test]
    fn new_users_get_a_free_nickname() {
        let store = store();
        let nicknames: Vec<String> = ["Ana", "ana", "ANA", "Ben"]
            .iter()
            .enumerate()
            .map(|(subject, nickname)| sign_in(&store, &subject.to_string(), nickname).nickname)
            .collect();
        assert_eq!(nicknames, ["Ana", "ana 2", "ANA 3", "Ben"]);
        let long = sign_in(&store, "long", &"x".repeat(40)).nickname;
        assert_eq!(long, "x".repeat(NICKNAME_BASE_LENGTH));
    }

    #[test]
    fn profiles_are_updated_with_a_free_nickname() {
        let store = store();
        let ana = sign_in(&store, "a", "Ana");
        let ben = sign_in(&store, "b", "Ben");
        assert_eq!(
            store.update_user(ben.id, Some("aNa"), Some("x")).unwrap(),
            ProfileUpdate::NicknameTaken
        );
        let ProfileUpdate::Updated(renamed) = store.update_user(ana.id, Some("ANA"), None).unwrap()
        else {
            panic!("ANA is Ana's own nickname");
        };
        assert_eq!(renamed.nickname, "ANA");
        let ProfileUpdate::Updated(described) = store
            .update_user(ana.id, None, Some("Joue une barbare."))
            .unwrap()
        else {
            panic!("profile not updated");
        };
        assert_eq!(described.nickname, "ANA");
        assert_eq!(described.profile, "Joue une barbare.");
        assert_eq!(store.session_user("none").unwrap(), None);
        store.create_session(ana.id, "ana", HOUR).unwrap();
        assert_eq!(store.session_user("ana").unwrap(), Some(described));
        assert_eq!(
            store.update_user(ben.id, None, None).unwrap(),
            ProfileUpdate::Updated(ben)
        );
    }

    #[test]
    fn identities_are_linked_to_one_user() {
        let store = store();
        let ana = sign_in(&store, "a", "Ana");
        let ben = sign_in(&store, "b", "Ben");
        let other = "https://other.test";
        assert_eq!(
            store
                .link_identity(ana.id, other, "a2", "ana@other")
                .unwrap(),
            Linking::Linked
        );
        assert_eq!(
            store
                .link_identity(ana.id, other, "a2", "ana@renamed")
                .unwrap(),
            Linking::AlreadyYours
        );
        assert_eq!(
            store.link_identity(ana.id, ISSUER, "b", "").unwrap(),
            Linking::Taken
        );
        assert_eq!(
            store.identities(ana.id).unwrap(),
            vec![identity(ISSUER, ""), identity(other, "ana@renamed")]
        );
        assert_eq!(
            store.identities(ben.id).unwrap(),
            vec![identity(ISSUER, "")]
        );
        let (signed_in, created) = store.user_for_identity(other, "a2", "Someone", "").unwrap();
        assert!(!created);
        assert_eq!(signed_in, ana);
    }

    #[test]
    fn sessions_resolve_until_deleted_or_expired() {
        let store = store();
        let ana = sign_in(&store, "a", "Ana");
        store.create_session(ana.id, "live", HOUR).unwrap();
        store
            .create_session(ana.id, "expired", Duration::ZERO)
            .unwrap();
        assert_eq!(store.session_user("live").unwrap(), Some(ana));
        assert_eq!(store.session_user("expired").unwrap(), None);
        assert_eq!(store.session_user("unknown").unwrap(), None);
        store.purge_expired().unwrap();
        store.delete_session("live").unwrap();
        assert_eq!(store.session_user("live").unwrap(), None);
    }

    #[test]
    fn a_login_is_taken_once_and_before_it_expires() {
        let store = store();
        let ana = sign_in(&store, "a", "Ana");
        store
            .create_login("live", &login("dex", None), HOUR)
            .unwrap();
        store
            .create_login("link", &login("dex", Some(ana.id)), HOUR)
            .unwrap();
        store
            .create_login("expired", &login("dex", None), Duration::ZERO)
            .unwrap();
        assert_eq!(store.take_login("live").unwrap(), Some(login("dex", None)));
        assert_eq!(
            store.take_login("link").unwrap(),
            Some(login("dex", Some(ana.id)))
        );
        assert_eq!(store.take_login("live").unwrap(), None);
        assert_eq!(store.take_login("expired").unwrap(), None);
        assert_eq!(store.take_login("unknown").unwrap(), None);
    }

    #[test]
    fn characters_belong_to_their_user() {
        let store = store();
        let ana = sign_in(&store, "a", "Ana");
        let ben = sign_in(&store, "b", "Ben");
        let brune = store
            .create_character(ana.id, "Brune", Engine::Fantasy)
            .unwrap();
        let max = store
            .create_character(ben.id, "Max", Engine::Contemporain)
            .unwrap();
        assert_ne!(brune.id, max.id);
        assert_eq!(brune.document, Document::empty(Engine::Fantasy));
        assert_eq!(store.characters(ana.id).unwrap(), vec![brune.clone()]);
        assert_eq!(store.characters(ben.id).unwrap(), vec![max]);

        assert_eq!(store.character(ben.id, brune.id).unwrap(), None);
        assert_eq!(store.rename_character(ben.id, brune.id, "X").unwrap(), None);
        assert_eq!(
            store
                .replace_document(ben.id, brune.id, 1, &Document::empty(Engine::Fantasy))
                .unwrap(),
            Replaced::NotFound
        );
        assert!(!store.delete_character(ben.id, brune.id).unwrap());
        assert_eq!(store.character(ana.id, brune.id).unwrap(), Some(brune));
    }

    #[test]
    fn documents_are_replaced_at_the_expected_version_only() {
        let store = store();
        let ana = sign_in(&store, "a", "Ana");
        let brune = store
            .create_character(ana.id, "Brune", Engine::Fantasy)
            .unwrap();
        let mut document = palimpseste_document::FantasyDocument::empty();
        document.profil = Some("cof.profil.guerrier".into());
        let document = Document::Fantasy(document);

        assert_eq!(
            store
                .replace_document(ana.id, brune.id, 1, &document)
                .unwrap(),
            Replaced::Stored(2)
        );
        assert_eq!(
            store
                .replace_document(ana.id, brune.id, 1, &document)
                .unwrap(),
            Replaced::Stale(2)
        );
        assert_eq!(
            store
                .replace_document(ana.id, brune.id, 2, &Document::empty(Engine::Contemporain))
                .unwrap(),
            Replaced::OtherEngine
        );
        assert_eq!(
            store
                .replace_document(ana.id, CharacterId(9), 1, &document)
                .unwrap(),
            Replaced::NotFound
        );
        let stored = store.character(ana.id, brune.id).unwrap().unwrap();
        assert_eq!(stored.document, document);
        assert_eq!(stored.version, 2);
    }

    #[test]
    fn characters_are_renamed_and_deleted() {
        let store = store();
        let ana = sign_in(&store, "a", "Ana");
        let brune = store
            .create_character(ana.id, "Brune", Engine::Fantasy)
            .unwrap();
        let renamed = store
            .rename_character(ana.id, brune.id, "Bruna")
            .unwrap()
            .unwrap();
        assert_eq!(renamed.name, "Bruna");
        assert!(store.delete_character(ana.id, brune.id).unwrap());
        assert!(!store.delete_character(ana.id, brune.id).unwrap());
        assert_eq!(store.rename_character(ana.id, brune.id, "X").unwrap(), None);
        assert_eq!(store.characters(ana.id).unwrap(), vec![]);
    }

    #[test]
    fn migrates_a_database_of_adr_0009() {
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
            let store = Store::open(&path).unwrap();
            let (ana, created) = store.user_for_identity(ISSUER, "a", "X", "").unwrap();
            assert!(!created);
            assert_eq!(ana.nickname, "Ana");
            assert_eq!(
                store.identities(ana.id).unwrap(),
                vec![identity(ISSUER, "")]
            );
            assert_eq!(
                store.update_user(ana.id, Some("ana 3"), None).unwrap(),
                ProfileUpdate::NicknameTaken
            );
            assert_eq!(
                store.update_user(ana.id, Some("Ben"), None).unwrap(),
                ProfileUpdate::NicknameTaken
            );
        }
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn data_outlives_the_store() {
        let path =
            std::env::temp_dir().join(format!("palimpseste-store-{}.sqlite3", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let (ana, brune) = {
            let store = Store::open(&path).unwrap();
            let ana = sign_in(&store, "a", "Ana");
            store.create_session(ana.id, "token", HOUR).unwrap();
            let brune = store
                .create_character(ana.id, "Brune", Engine::Fantasy)
                .unwrap();
            (ana, brune)
        };
        let reopened = Store::open(&path).unwrap();
        assert_eq!(reopened.character(ana.id, brune.id).unwrap(), Some(brune));
        assert_eq!(reopened.session_user("token").unwrap(), Some(ana));
        drop(reopened);
        std::fs::remove_file(&path).unwrap();
    }
}
