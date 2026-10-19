//! Friends (ADR BE-FRIENDS): two users who accepted to see each other's
//! nickname, profile and characters. A friendship is mutual: one of them
//! ending it ends it for both.

use std::fmt;

use crate::{Nickname, Profile, UserId};

/// Identifier of a friendship, naming the friend to either side of it, so a
/// user's id never leaves the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FriendshipId(i64);

impl FriendshipId {
    /// The id as a number, as URLs carry it.
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }
}

impl From<i64> for FriendshipId {
    fn from(id: i64) -> Self {
        Self(id)
    }
}

impl fmt::Display for FriendshipId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A friend of the signed-in user, as they see them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Friend {
    /// The friendship, naming the friend.
    pub friendship: FriendshipId,
    /// The friend's public nickname.
    pub nickname: Nickname,
    /// The friend's public profile.
    pub profile: Profile,
}

/// The two users of a friendship in the order it is kept: one record per
/// pair, whichever of them invited the other.
pub(crate) fn pair(one: UserId, other: UserId) -> (UserId, UserId) {
    (one.min(other), one.max(other))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pair_is_the_same_both_ways() {
        let (ana, ben) = (UserId(1), UserId(2));
        assert_eq!(pair(ana, ben), (ana, ben));
        assert_eq!(pair(ben, ana), (ana, ben));
    }
}
