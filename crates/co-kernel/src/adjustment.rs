//! Manual adjustments: attributed contributions added by a user.

/// Who made an adjustment: the Game Master or a player.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Author(String);

impl Author {
    /// Creates an author.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Returns the author's name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A signed amount added by hand to a derived value or a budget.
///
/// It carries no target: the engine owning the value decides where it applies.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Adjustment {
    amount: i32,
    author: Author,
    reason: String,
}

impl Adjustment {
    /// Creates an adjustment.
    #[must_use]
    pub fn new(amount: i32, author: Author, reason: impl Into<String>) -> Self {
        Self {
            amount,
            author,
            reason: reason.into(),
        }
    }

    /// Returns the signed amount.
    #[must_use]
    pub const fn amount(&self) -> i32 {
        self.amount
    }

    /// Returns who made the adjustment.
    #[must_use]
    pub const fn author(&self) -> &Author {
        &self.author
    }

    /// Returns why the adjustment was made.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}
