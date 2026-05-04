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
    DiffSummary, SortOrder, StashEntry, StatusDigest, StatusEntry, SubmoduleInfo, TagAnnotation,
    TagInfo, WorktreeInfo, WorktreeStatus,
};

/// The [`VcsBackend`] trait, re-exported for implementing custom backends.
///
/// **Stability note**: the trait signature may change before v1.0.
pub use endringer_core::backend::VcsBackend;

pub mod repository;

/// Converts a [`CommitId`] to its 7-character hex abbreviation.
///
/// # Deprecation
///
/// Call [`CommitId::short`] directly instead:
///
/// ```
/// # use endringer::CommitId;
/// # let id = CommitId::from_hex("0000000000000000000000000000000000000000").unwrap();
/// let short = id.short(); // preferred
/// ```
#[deprecated(since = "0.18.0", note = "use `commit_id.short()` directly")]
#[allow(dead_code)]
pub fn commit_id_to_short_id(commit_id: CommitId) -> String {
    commit_id.short()
}
