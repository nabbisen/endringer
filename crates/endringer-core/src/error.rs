//! Typed public error model for endringer (RFC 006).
//!
//! # Overview
//!
//! All public methods on [`crate::backend::VcsBackend`] and on the
//! `endringer::repository::Repository` façade return
//! `endringer_core::Result<T>` (equivalently `endringer::Result<T>`).
//!
//! The error type is `#[non_exhaustive]`: new variants may be added in minor
//! releases without a new breaking wave. Consumers must include a wildcard
//! arm when matching.
//!
//! # Matching errors
//!
//! Match on variants rather than on `Display` strings:
//!
//! ```rust,ignore
//! use endringer::{Error, NotFoundKind};
//!
//! match repo.find_commit(&id) {
//!     Ok(info) => { /* … */ }
//!     Err(Error::NotFound { kind: NotFoundKind::Commit, name }) => {
//!         eprintln!("commit {name} not found");
//!     }
//!     Err(err) => return Err(err),
//! }
//! ```

use std::fmt;

use crate::types::{BackendKind, CommitId, ObjectId};

// ── Result alias ─────────────────────────────────────────────────────────── //

/// Convenience alias for `std::result::Result<T, endringer_core::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

// ── Error enum ───────────────────────────────────────────────────────────── //

/// The public error type for all endringer operations.
///
/// Marked `#[non_exhaustive]` — new variants may be added in minor releases.
/// Always include a wildcard arm when matching.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The path does not contain a recognised repository.
    NotARepository { path: std::path::PathBuf },

    /// The repository exists but has no commits yet.
    EmptyRepository,

    /// A named object (commit, ref, branch, tag, remote, …) was not found.
    NotFound { kind: NotFoundKind, name: String },

    /// A commit ID hex string was invalid (wrong length or non-hex chars).
    InvalidCommitId { value: String },

    /// An object ID hex string was invalid (wrong length or non-hex chars).
    InvalidObjectId { value: String },

    /// A ref name string was invalid.
    InvalidRefName { value: String },

    /// An object was found but is not a commit.
    NotACommit { id: CommitId },

    /// An object was found but is not a tree.
    NotATree { id: ObjectId },

    /// A path was not present in the tree of the given commit.
    PathNotFound {
        path: std::path::PathBuf,
        commit: Option<CommitId>,
    },

    /// A path could not be represented as valid UTF-8.
    NonUtf8Path { path: std::path::PathBuf },

    /// The operation is not supported on a bare repository.
    BareRepositoryUnsupported { operation: &'static str },

    /// The backend does not support this feature for this repository.
    UnsupportedBackendFeature {
        backend: Option<BackendKind>,
        feature: &'static str,
    },

    /// The repository uses an object format endringer does not fully support.
    UnsupportedObjectFormat { format: String },

    /// A SHA-1 collision was detected by gix's collision-detecting hasher.
    HashCollision,

    /// The repository data appears corrupt.
    CorruptRepository { message: String },

    /// An I/O error.
    Io(std::io::Error),

    /// A `tokio::task::spawn_blocking` task failed to join.
    TaskJoin { message: String },

    /// An unclassified error from the backend.
    ///
    /// `message` carries a human-readable description. `source` carries the
    /// original error chain for debugging.
    Backend {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

// ── NotFoundKind ─────────────────────────────────────────────────────────── //

/// The kind of named object that was not found.
///
/// Marked `#[non_exhaustive]` — new variants may be added in minor releases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum NotFoundKind {
    Commit,
    Ref,
    Branch,
    Tag,
    Remote,
    Path,
    Worktree,
    Submodule,
}

impl fmt::Display for NotFoundKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotFoundKind::Commit    => write!(f, "commit"),
            NotFoundKind::Ref      => write!(f, "ref"),
            NotFoundKind::Branch   => write!(f, "branch"),
            NotFoundKind::Tag      => write!(f, "tag"),
            NotFoundKind::Remote   => write!(f, "remote"),
            NotFoundKind::Path     => write!(f, "path"),
            NotFoundKind::Worktree => write!(f, "worktree"),
            NotFoundKind::Submodule => write!(f, "submodule"),
        }
    }
}

// ── Display ──────────────────────────────────────────────────────────────── //

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotARepository { path } =>
                write!(f, "not a repository: {}", path.display()),
            Error::EmptyRepository =>
                write!(f, "repository has no commits"),
            Error::NotFound { kind, name } =>
                write!(f, "{kind} not found: {name}"),
            Error::InvalidCommitId { value } =>
                write!(f, "invalid commit id {value:?}: expected 40 (SHA-1) or 64 (SHA-256) hex chars"),
            Error::InvalidObjectId { value } =>
                write!(f, "invalid object id {value:?}: expected 40 (SHA-1) or 64 (SHA-256) hex chars"),
            Error::InvalidRefName { value } =>
                write!(f, "invalid ref name: {value:?}"),
            Error::NotACommit { id } =>
                write!(f, "object {} is not a commit", id.short()),
            Error::NotATree { id } =>
                write!(f, "object {} is not a tree", id.short()),
            Error::PathNotFound { path, commit: None } =>
                write!(f, "path not found: {}", path.display()),
            Error::PathNotFound { path, commit: Some(c) } =>
                write!(f, "path not found at commit {}: {}", c.short(), path.display()),
            Error::NonUtf8Path { path } =>
                write!(f, "path is not valid UTF-8: {}", path.display()),
            Error::BareRepositoryUnsupported { operation } =>
                write!(f, "operation not supported on bare repository: {operation}"),
            Error::UnsupportedBackendFeature { backend: None, feature } =>
                write!(f, "backend does not support {feature}"),
            Error::UnsupportedBackendFeature { backend: Some(b), feature } =>
                write!(f, "{b:?} backend does not support {feature}"),
            Error::UnsupportedObjectFormat { format } =>
                write!(f, "unsupported object format: {format}"),
            Error::HashCollision =>
                write!(f, "SHA-1 hash collision detected"),
            Error::CorruptRepository { message } =>
                write!(f, "corrupt repository: {message}"),
            Error::Io(e) =>
                write!(f, "I/O error: {e}"),
            Error::TaskJoin { message } =>
                write!(f, "async task failed to join: {message}"),
            Error::Backend { message, .. } =>
                write!(f, "backend error: {message}"),
        }
    }
}

// ── std::error::Error ────────────────────────────────────────────────────── //

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Backend { source: Some(s), .. } => Some(&**s),
            _ => None,
        }
    }
}

// ── From conversions ─────────────────────────────────────────────────────── //

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

/// Convert an `anyhow::Error` into a `Backend` variant.
///
/// Used by backend crates during the transition from `anyhow` to typed errors.
/// When a gix call produces an `anyhow::Error` that hasn't been classified,
/// this wraps it as `Backend { message, source: None }`.
pub fn anyhow_to_backend(err: anyhow::Error) -> Error {
    Error::Backend {
        message: err.to_string(),
        source: None,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────── //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_not_a_repository() {
        let e = Error::NotARepository { path: "/tmp/nope".into() };
        assert!(e.to_string().contains("not a repository"));
        assert!(e.to_string().contains("nope"));
    }

    #[test]
    fn display_not_found_commit() {
        let e = Error::NotFound {
            kind: NotFoundKind::Commit,
            name: "abc123".into(),
        };
        assert!(e.to_string().contains("commit"));
        assert!(e.to_string().contains("abc123"));
    }

    #[test]
    fn display_unsupported_jj_tag() {
        let e = Error::UnsupportedBackendFeature {
            backend: Some(BackendKind::Jj),
            feature: "create_annotated_tag",
        };
        let s = e.to_string();
        assert!(s.contains("Jj") || s.contains("jj"));
        assert!(s.contains("create_annotated_tag"));
    }

    #[test]
    fn display_io_error() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let e = Error::Io(io);
        assert!(e.to_string().contains("I/O"));
    }

    #[test]
    fn source_for_io_error() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let e = Error::Io(io);
        assert!(std::error::Error::source(&e).is_some());
    }

    #[test]
    fn source_for_backend_without_source() {
        let e = Error::Backend { message: "oops".into(), source: None };
        assert!(std::error::Error::source(&e).is_none());
    }

    #[test]
    fn from_io_error() {
        let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let e: Error = io.into();
        assert!(matches!(e, Error::Io(_)));
    }

    #[test]
    fn not_found_kind_display() {
        assert_eq!(NotFoundKind::Commit.to_string(),    "commit");
        assert_eq!(NotFoundKind::Branch.to_string(),    "branch");
        assert_eq!(NotFoundKind::Tag.to_string(),       "tag");
        assert_eq!(NotFoundKind::Remote.to_string(),    "remote");
        assert_eq!(NotFoundKind::Worktree.to_string(),  "worktree");
        assert_eq!(NotFoundKind::Submodule.to_string(), "submodule");
    }

    #[test]
    fn hash_collision_display() {
        let e = Error::HashCollision;
        assert!(e.to_string().contains("collision"));
    }
}
