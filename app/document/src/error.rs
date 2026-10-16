//! Why a document cannot be read.

use std::fmt;

/// Why a document cannot be read. A rule broken is never one of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentError {
    /// The document's version is not [`VERSION`](crate::VERSION).
    UnsupportedVersion(u32),
    /// An identifier names no content of the engine.
    UnknownContent {
        /// What the identifier should name: a profil, a voie, a value...
        kind: &'static str,
        /// The identifier.
        id: String,
    },
    /// A field that must be filled is blank: an author, a reason, a rule,
    /// a subject, an item name.
    Empty(&'static str),
}

impl fmt::Display for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported document version {version}")
            }
            Self::UnknownContent { kind, id } => write!(f, "unknown {kind} `{id}`"),
            Self::Empty(field) => write!(f, "empty {field}"),
        }
    }
}

impl std::error::Error for DocumentError {}
