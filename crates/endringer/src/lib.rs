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

pub use endringer_core::error::{Error, NotFoundKind, Result};

pub use endringer_core::types::{
    AheadBehind, BackendKind, BlameEntry, BranchInfo, BranchTrackingInfo, ChangeKind,
    CommitId, CommitIdFromHexError, CommitInfo, CommitQuery, CommitQueryResult,
    CommitQueryStart, ConflictPath, ConflictStage,
    ConflictSummary, DiffSummary, HeadState, ObjectFormat, ObjectId,
    ObjectIdFromHexError, OperationState, RebaseKind, RefInfo, RefKind, RefTarget,
    RemoteInfo, RepositoryCapabilities,
    RepositoryInfo, SortOrder, StashDetail, StashEntry, StashId,
    StatusDigest, StatusEntry, SubmoduleInfo, SubmoduleSummary, SubmoduleState,
    TagAnnotation, TagInfo, TreeEntry, TreeEntryKind,
    WorktreeDetail, WorktreeInfo, WorktreeState, WorktreeStatus,
};

/// The [`VcsBackend`] trait, re-exported for implementing custom backends.
///
/// **Stability note**: the trait signature may change before v1.0.
pub use endringer_core::backend::VcsBackend;

pub mod repository;


