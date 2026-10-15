//! In-memory storage of users and their characters.

use std::{
    collections::BTreeMap,
    sync::{Mutex, PoisonError},
};

use serde::{Deserialize, Serialize};

/// Identifier of a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(u64);

/// Identifier of a character, unique across all users.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CharacterId(u64);

/// The engine a character is built with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    /// COC2, `co-contemporain`.
    Contemporain,
    /// COF2, `co-fantasy`.
    Fantasy,
}

/// A user, owner of a list of characters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct User {
    pub id: UserId,
    pub name: String,
}

/// A character of a user, tied to its engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Character {
    pub id: CharacterId,
    pub name: String,
    pub engine: Engine,
}

/// Users and their characters, lost when the process stops.
#[derive(Debug, Default)]
pub struct Store {
    inner: Mutex<Inner>,
}

#[derive(Debug, Default)]
struct Inner {
    users: BTreeMap<UserId, String>,
    characters: BTreeMap<UserId, Vec<Character>>,
    next_user: u64,
    next_character: u64,
}

impl Store {
    pub fn create_user(&self, name: String) -> User {
        let mut inner = self.lock();
        inner.next_user += 1;
        let id = UserId(inner.next_user);
        inner.users.insert(id, name.clone());
        inner.characters.insert(id, Vec::new());
        User { id, name }
    }

    pub fn users(&self) -> Vec<User> {
        self.lock()
            .users
            .iter()
            .map(|(&id, name)| User {
                id,
                name: name.clone(),
            })
            .collect()
    }

    pub fn user(&self, id: UserId) -> Option<User> {
        let inner = self.lock();
        let name = inner.users.get(&id)?.clone();
        Some(User { id, name })
    }

    /// Returns `None` when the user is unknown.
    pub fn create_character(
        &self,
        user: UserId,
        name: String,
        engine: Engine,
    ) -> Option<Character> {
        let mut inner = self.lock();
        let id = CharacterId(inner.next_character + 1);
        let characters = inner.characters.get_mut(&user)?;
        let character = Character { id, name, engine };
        characters.push(character.clone());
        inner.next_character += 1;
        Some(character)
    }

    /// Returns `None` when the user is unknown.
    pub fn characters(&self, user: UserId) -> Option<Vec<Character>> {
        self.lock().characters.get(&user).cloned()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        // Every update leaves `Inner` consistent, so a poisoned lock is safe to reuse.
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocates_distinct_user_ids() {
        let store = Store::default();
        let ana = store.create_user("Ana".into());
        let ben = store.create_user("Ben".into());
        assert_ne!(ana.id, ben.id);
        assert_eq!(store.users(), vec![ana.clone(), ben]);
        assert_eq!(store.user(ana.id), Some(ana));
    }

    #[test]
    fn unknown_user_has_no_characters() {
        let store = Store::default();
        let unknown = UserId(42);
        assert_eq!(store.user(unknown), None);
        assert_eq!(store.characters(unknown), None);
        assert_eq!(
            store.create_character(unknown, "Brune".into(), Engine::Fantasy),
            None
        );
    }

    #[test]
    fn characters_belong_to_their_user() {
        let store = Store::default();
        let ana = store.create_user("Ana".into());
        let ben = store.create_user("Ben".into());
        let brune = store
            .create_character(ana.id, "Brune".into(), Engine::Fantasy)
            .unwrap();
        let max = store
            .create_character(ben.id, "Max".into(), Engine::Contemporain)
            .unwrap();
        assert_ne!(brune.id, max.id);
        assert_eq!(store.characters(ana.id), Some(vec![brune]));
        assert_eq!(store.characters(ben.id), Some(vec![max]));
    }
}
