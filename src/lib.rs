//! **endringer** — lightweight Git repository introspection.
//!
//! This library provides a thin, ergonomic layer on top of `gix` for reading
//! common repository state: branches, commit history, tags, and a quick
//! status digest.  It follows the UNIX philosophy of doing one thing well:
//! endringer reads and inspects a repository; it delegates everything else
//! (persistence, UI, scheduling) to the caller.
//!
//! # Quick start
//!
//! ```no_run
//! use std::path::Path;
//! use endringer::repository::repository;
//!
//! let repo = repository(Path::new(".")).expect("open repo");
//!
//! // Current state
//! let digest = repo.status_digest().expect("status digest");
//! println!("{} @ {}", digest.current_branch, digest.last_commit_id.short());
//!
//! // Recent commits
//! let commits = repo.list_commits().expect("list commits");
//! for c in &commits {
//!     println!("{} {}", c.commit_id.short(), c.summary);
//! }
//!
//! // Tags
//! let tags = repo.list_tags().expect("list tags");
//! for t in &tags {
//!     println!("{}", t.name);
//! }
//! ```
//!
//! # Public surface
//!
//! - [`repository`] — open a repository at a given path
//! - [`repository::Repository`] — the main handle for all operations
//! - [`types`] — all public data types (`CommitId`, `BranchInfo`, …)
//! - [`commit_id_to_short_id`] — convenience helper

pub mod repository;
pub mod types;
mod util;

/// Converts a [`types::CommitId`] to its conventional 7-character hex
/// abbreviation.
///
/// This is a free-function convenience wrapper around [`types::CommitId::short`].
pub fn commit_id_to_short_id(commit_id: types::CommitId) -> String {
    util::commit_id_to_short_id(commit_id)
}
