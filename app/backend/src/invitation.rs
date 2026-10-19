//! Invitations (ADR BE-INVITATIONS): a link a user sends to become friends
//! with the one person who accepts it, within a week. The server keeps only
//! its token's hash, so the link is shown once, when created.

use std::{fmt, time::Duration};

use crate::{
    FriendshipId, TokenHash, User,
    session::{hash_token, random_token},
};

/// How long an invitation waits to be accepted.
pub const INVITATION_TTL: Duration = Duration::from_hours(7 * 24);

/// The secret an invitation link carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvitationToken(String);

impl InvitationToken {
    /// A new random token.
    pub(crate) fn random() -> Self {
        Self(random_token())
    }

    /// The token a link carried.
    #[must_use]
    pub fn from_link(value: &str) -> Self {
        Self(value.to_owned())
    }

    /// The token, to send to its creator once.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// What the repository keeps: reading it gives no usable invitation.
    pub(crate) fn hash(&self) -> TokenHash {
        hash_token(&self.0)
    }
}

/// Identifier of an invitation, for its creator to revoke it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InvitationId(i64);

impl InvitationId {
    /// The id as a number, as URLs carry it.
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }
}

impl From<i64> for InvitationId {
    fn from(id: i64) -> Self {
        Self(id)
    }
}

impl fmt::Display for InvitationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A moment, in seconds since the Unix epoch, as the store keeps it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnixTime(pub(crate) i64);

impl UnixTime {
    /// The seconds since the Unix epoch.
    #[must_use]
    pub const fn seconds(self) -> i64 {
        self.0
    }
}

/// An invitation waiting to be accepted, as its creator sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invitation {
    /// Its id.
    pub id: InvitationId,
    /// When it stops being accepted.
    pub expires_at: UnixTime,
}

/// An invitation as the user who opened its link sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceivedInvitation {
    /// Who sent it: only their nickname and profile are shown.
    pub inviter: User,
    /// What accepting it would do.
    pub status: InvitationStatus,
}

/// What accepting an invitation would do for the user who opened it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvitationStatus {
    /// Make them friends.
    Open,
    /// Nothing: it is their own.
    Own,
    /// Nothing: they are already friends, through this friendship.
    Friends(FriendshipId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_random_and_hashed() {
        let token = InvitationToken::random();
        assert_eq!(token.as_str().len(), 64);
        assert_ne!(token, InvitationToken::random());
        assert_eq!(
            token.hash(),
            InvitationToken::from_link(token.as_str()).hash()
        );
        assert_ne!(token.hash().0, token.as_str());
    }
}
