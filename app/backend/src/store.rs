//! Storage of users and their characters in SQLite (ADR 0007, D7).
//!
//! The store keeps what it is given: names, engines and choices documents.
//! It never reads the rules; the API checks that a document is readable
//! before storing it. Methods block, so the API calls them from
//! `spawn_blocking`.

use std::{
    fmt,
    path::Path,
    sync::{Mutex, MutexGuard, PoisonError},
};

use palimpseste_document::{Document, Engine};
use rusqlite::{Connection, OptionalExtension, Row, params};
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

/// A user, owner of a list of characters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct User {
    pub id: UserId,
    pub name: String,
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

/// An error of the database, or a stored document it cannot read back.
pub type Result<T> = rusqlite::Result<T>;

const SCHEMA: &str = "
    PRAGMA foreign_keys = ON;
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS characters (
        id INTEGER PRIMARY KEY,
        user_id INTEGER NOT NULL REFERENCES users (id),
        name TEXT NOT NULL,
        engine TEXT NOT NULL,
        document TEXT NOT NULL,
        version INTEGER NOT NULL
    );
";

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

    fn with(connection: Connection) -> Result<Self> {
        connection.execute_batch(SCHEMA)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub fn create_user(&self, name: &str) -> Result<User> {
        let connection = self.lock();
        connection.execute("INSERT INTO users (name) VALUES (?1)", params![name])?;
        Ok(User {
            id: UserId(connection.last_insert_rowid()),
            name: name.to_owned(),
        })
    }

    pub fn users(&self) -> Result<Vec<User>> {
        let connection = self.lock();
        let mut statement = connection.prepare("SELECT id, name FROM users ORDER BY id")?;
        statement.query_map([], user)?.collect()
    }

    pub fn user(&self, id: UserId) -> Result<Option<User>> {
        self.lock()
            .query_row(
                "SELECT id, name FROM users WHERE id = ?1",
                params![id.0],
                user,
            )
            .optional()
    }

    /// Creates a character with the empty document of its engine; `None`
    /// when the user is unknown.
    pub fn create_character(
        &self,
        user: UserId,
        name: &str,
        engine: Engine,
    ) -> Result<Option<Character>> {
        if self.user(user)?.is_none() {
            return Ok(None);
        }
        let document = Document::empty(engine);
        let connection = self.lock();
        connection.execute(
            "INSERT INTO characters (user_id, name, engine, document, version)
             VALUES (?1, ?2, ?3, ?4, 1)",
            params![user.0, name, engine_name(engine), to_json(&document)?],
        )?;
        Ok(Some(Character {
            id: CharacterId(connection.last_insert_rowid()),
            name: name.to_owned(),
            engine,
            document,
            version: 1,
        }))
    }

    /// The characters of `user`; `None` when the user is unknown.
    pub fn characters(&self, user: UserId) -> Result<Option<Vec<Character>>> {
        if self.user(user)?.is_none() {
            return Ok(None);
        }
        let connection = self.lock();
        let mut statement = connection.prepare(&format!(
            "SELECT {CHARACTER_COLUMNS} FROM characters WHERE user_id = ?1 ORDER BY id"
        ))?;
        statement
            .query_map(params![user.0], character)?
            .collect::<Result<_>>()
            .map(Some)
    }

    pub fn character(&self, id: CharacterId) -> Result<Option<Character>> {
        self.lock()
            .query_row(
                &format!("SELECT {CHARACTER_COLUMNS} FROM characters WHERE id = ?1"),
                params![id.0],
                character,
            )
            .optional()
    }

    /// Replaces the document of character `id` if its version is
    /// `expected`, and increments the version.
    pub fn replace_document(
        &self,
        id: CharacterId,
        expected: i64,
        document: &Document,
    ) -> Result<Replaced> {
        let mut connection = self.lock();
        let transaction = connection.transaction()?;
        let current = transaction
            .query_row(
                "SELECT engine, version FROM characters WHERE id = ?1",
                params![id.0],
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

    /// Renames character `id`; `None` when it is unknown.
    pub fn rename_character(&self, id: CharacterId, name: &str) -> Result<Option<Character>> {
        let changed = self.lock().execute(
            "UPDATE characters SET name = ?1 WHERE id = ?2",
            params![name, id.0],
        )?;
        if changed == 0 {
            return Ok(None);
        }
        self.character(id)
    }

    /// Deletes character `id`; `false` when it is unknown.
    pub fn delete_character(&self, id: CharacterId) -> Result<bool> {
        let deleted = self
            .lock()
            .execute("DELETE FROM characters WHERE id = ?1", params![id.0])?;
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

const fn engine_name(engine: Engine) -> &'static str {
    match engine {
        Engine::Contemporain => "contemporain",
        Engine::Fantasy => "fantasy",
    }
}

fn user(row: &Row<'_>) -> Result<User> {
    Ok(User {
        id: UserId(row.get(0)?),
        name: row.get(1)?,
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

    fn store() -> Store {
        Store::in_memory().unwrap()
    }

    #[test]
    fn allocates_distinct_user_ids() {
        let store = store();
        let ana = store.create_user("Ana").unwrap();
        let ben = store.create_user("Ben").unwrap();
        assert_ne!(ana.id, ben.id);
        assert_eq!(store.users().unwrap(), vec![ana.clone(), ben]);
        assert_eq!(store.user(ana.id).unwrap(), Some(ana));
    }

    #[test]
    fn unknown_user_has_no_characters() {
        let store = store();
        let unknown = UserId(42);
        assert_eq!(store.user(unknown).unwrap(), None);
        assert_eq!(store.characters(unknown).unwrap(), None);
        assert_eq!(
            store
                .create_character(unknown, "Brune", Engine::Fantasy)
                .unwrap(),
            None
        );
    }

    #[test]
    fn characters_belong_to_their_user() {
        let store = store();
        let ana = store.create_user("Ana").unwrap();
        let ben = store.create_user("Ben").unwrap();
        let brune = store
            .create_character(ana.id, "Brune", Engine::Fantasy)
            .unwrap()
            .unwrap();
        let max = store
            .create_character(ben.id, "Max", Engine::Contemporain)
            .unwrap()
            .unwrap();
        assert_ne!(brune.id, max.id);
        assert_eq!(brune.document, Document::empty(Engine::Fantasy));
        assert_eq!(store.characters(ana.id).unwrap(), Some(vec![brune]));
        assert_eq!(store.characters(ben.id).unwrap(), Some(vec![max]));
    }

    #[test]
    fn documents_are_replaced_at_the_expected_version_only() {
        let store = store();
        let ana = store.create_user("Ana").unwrap();
        let brune = store
            .create_character(ana.id, "Brune", Engine::Fantasy)
            .unwrap()
            .unwrap();
        let mut document = palimpseste_document::FantasyDocument::empty();
        document.profil = Some("cof.profil.guerrier".into());
        let document = Document::Fantasy(document);

        assert_eq!(
            store.replace_document(brune.id, 1, &document).unwrap(),
            Replaced::Stored(2)
        );
        assert_eq!(
            store.replace_document(brune.id, 1, &document).unwrap(),
            Replaced::Stale(2)
        );
        assert_eq!(
            store
                .replace_document(brune.id, 2, &Document::empty(Engine::Contemporain))
                .unwrap(),
            Replaced::OtherEngine
        );
        assert_eq!(
            store
                .replace_document(CharacterId(9), 1, &document)
                .unwrap(),
            Replaced::NotFound
        );
        let stored = store.character(brune.id).unwrap().unwrap();
        assert_eq!(stored.document, document);
        assert_eq!(stored.version, 2);
    }

    #[test]
    fn characters_are_renamed_and_deleted() {
        let store = store();
        let ana = store.create_user("Ana").unwrap();
        let brune = store
            .create_character(ana.id, "Brune", Engine::Fantasy)
            .unwrap()
            .unwrap();
        let renamed = store.rename_character(brune.id, "Bruna").unwrap().unwrap();
        assert_eq!(renamed.name, "Bruna");
        assert!(store.delete_character(brune.id).unwrap());
        assert!(!store.delete_character(brune.id).unwrap());
        assert_eq!(store.rename_character(brune.id, "X").unwrap(), None);
        assert_eq!(store.characters(ana.id).unwrap(), Some(vec![]));
    }

    #[test]
    fn data_outlives_the_store() {
        let path =
            std::env::temp_dir().join(format!("palimpseste-store-{}.sqlite3", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let brune = {
            let store = Store::open(&path).unwrap();
            let ana = store.create_user("Ana").unwrap();
            store
                .create_character(ana.id, "Brune", Engine::Fantasy)
                .unwrap()
                .unwrap()
        };
        let reopened = Store::open(&path).unwrap();
        assert_eq!(reopened.character(brune.id).unwrap(), Some(brune));
        drop(reopened);
        std::fs::remove_file(&path).unwrap();
    }
}
