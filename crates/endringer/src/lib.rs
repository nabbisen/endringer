//! # endringer
//!
//! Lightweight VCS repository introspection library. Supports **Jujutsu** and
//! **Git** via [`gix`] — no external binaries required.
//!
//! ## Quick start (Git)
//!
//! ```no_run
//! use endringer::repository::repository;
//! use std::path::Path;
//!
//! let repo = repository(Path::new(".")).expect("open repo");
//! let digest = repo.status_digest().expect("status digest");
//! println!("{} @ {}", digest.current_branch, digest.last_commit_id.short());
//! ```
//!
//! ## Quick start (Jujutsu)
//!
//! ```no_run
//! use endringer::repository::jj_repository;
//! use std::path::Path;
//!
//! let repo = jj_repository(Path::new(".")).expect("open jj repo");
//! let digest = repo.status_digest().expect("status digest");
//! println!("{} @ {}", digest.current_branch, digest.last_commit_id.short());
//! ```
//!
//! ## Async
//!
//! Add the [`endringer-async`] crate for a `tokio::task::spawn_blocking`-based
//! async facade.

pub use endringer_core::types::{
    BackendKind, BlameEntry, BranchInfo, ChangeKind, CommitId, CommitIdFromHexError, CommitInfo,
    DiffSummary, SortOrder, StashEntry, StatusDigest, StatusEntry, SubmoduleInfo, TagInfo,
    WorktreeStatus,
};

/// The [`VcsBackend`] trait, re-exported for implementing custom backends.
///
/// **Stability note**: the trait signature may change before v1.0.
pub use endringer_core::backend::VcsBackend;

pub mod repository;

/// Converts a [`CommitId`] to its 7-character hex abbreviation.
///
/// Convenience wrapper around [`CommitId::short`].
pub fn commit_id_to_short_id(commit_id: CommitId) -> String {
    commit_id.short()
}
