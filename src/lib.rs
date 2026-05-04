//! # endringer
//!
//! Lightweight VCS repository introspection library.
//! Supports **Jujutsu** and **Git**, both via [`gix`] — no external binaries
//! required.
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
//! ## Async (requires `async` feature)
//!
//! ```no_run
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! use endringer::async_api::AsyncRepository;
//!
//! let repo = AsyncRepository::open(std::path::Path::new(".")).await?;
//! let commits = repo.list_commits().await?;
//! # Ok(())
//! # }
//! ```

pub(crate) mod backend;
pub(crate) mod git;
pub(crate) mod jj;
pub mod repository;
pub mod types;
mod util;

#[cfg(feature = "async")]
pub mod async_api;

pub use types::BackendKind;
pub use types::CommitIdFromHexError;
pub use types::DiffSummary;
pub use types::SortOrder;

/// Converts a [`types::CommitId`] to its 7-character hex abbreviation.
///
/// Convenience wrapper around [`types::CommitId::short`].
pub fn commit_id_to_short_id(commit_id: types::CommitId) -> String {
    util::commit_id_to_short_id(commit_id)
}
