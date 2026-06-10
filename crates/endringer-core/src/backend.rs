//! The [`VcsBackend`] trait that all VCS backends implement.
//!
//! This trait is `pub` so that downstream crates can implement custom
//! backends and inject them via [`endringer::repository::Repository::with_backend`].
//!
//! # Stability
//!
//! **Before v1.0, this trait is implementable but not fully stable.**
//! New required methods may still be added; however, any method that has a
//! truthful default implementation will be given one, so that adding it does
//! not break existing custom backends. Methods without a default are the
//! *required core methods* listed first in the trait below.
//!
//! Consumers that depend only on [`endringer::repository::Repository`]
//! receive stronger stability guarantees than consumers that implement
//! `VcsBackend` directly.
//!
//! # Method categories
//!
//! | Category | Default | Notes |
//! |---|---|---|
//! | Required core | none | must be implemented; no safe default exists |
//! | Optional-empty | `Ok(vec![])` or `None` | backend may have no such data |
//! | Write-side exception | unsupported error | tags are the only in-scope writes |

use std::time::SystemTime;

use crate::error::Result;
use crate::types::{
    AheadBehind, BlameEntry, BranchInfo, BranchTrackingInfo, CommitId, CommitInfo,
    ConflictSummary, DiffSummary, OperationState, RepositoryInfo, SortOrder,
    StashEntry, StatusDigest, SubmoduleInfo, TagInfo, TreeEntry, WorktreeInfo,
    WorktreeStatus,
};

/// Common interface implemented by every VCS backend.
///
/// All methods take `&self` and are safe to call concurrently from multiple
/// threads (`Send + Sync` bound).
///
/// See the [module documentation][self] for the stability policy and method
/// classification.
pub trait VcsBackend: Send + Sync {
    // ── Required core methods ──────────────────────────────────────────── //
    //
    // No defaults: a safe stand-in would be misleading or wrong.

    fn status_digest(&self) -> Result<StatusDigest>;

    fn local_branches(&self) -> Result<Vec<BranchInfo>>;
    fn remote_branches(&self) -> Result<Vec<BranchInfo>>;

    fn list_commits(&self) -> Result<Vec<CommitInfo>>;
    fn list_commits_sorted(&self, order: SortOrder) -> Result<Vec<CommitInfo>>;
    fn log_since(&self, since: SystemTime, until: SystemTime) -> Result<Vec<CommitInfo>>;
    fn find_commit(&self, id: &CommitId) -> Result<CommitInfo>;

    fn list_tags(&self) -> Result<Vec<TagInfo>>;
    fn list_tags_sorted(&self, order: SortOrder) -> Result<Vec<TagInfo>>;

    fn diff(&self, from: &CommitId, to: &CommitId) -> Result<DiffSummary>;

    /// Returns `true` if the working tree has any uncommitted changes
    /// (staged or unstaged). Bare repositories always return `false`.
    fn is_dirty(&self) -> Result<bool>;

    /// Returns the best common ancestor of `a` and `b`, or `None` if there
    /// is no shared history (unrelated histories).
    fn merge_base(&self, a: &CommitId, b: &CommitId) -> Result<Option<CommitId>>;

    /// Returns `true` if `candidate` is a direct or transitive ancestor of
    /// `descendant`. A commit is considered its own ancestor.
    fn is_ancestor(&self, candidate: &CommitId, descendant: &CommitId) -> Result<bool>;

    /// Returns per-line commit attribution for `path` at HEAD.
    ///
    /// `path` is relative to the repository root.
    /// Entries are in ascending line order.
    fn blame(&self, path: &std::path::Path) -> Result<Vec<BlameEntry>>;

    /// Returns per-file working-tree status: staged changes, unstaged
    /// changes, and untracked files (with gitignore applied).
    fn worktree_status(&self) -> Result<WorktreeStatus>;

    /// Returns the raw bytes of `path` (relative to the repository root)
    /// as it exists in the tree of `commit_id`.
    fn file_at_commit(&self, path: &std::path::Path, commit_id: &CommitId) -> Result<Vec<u8>>;

    /// Returns ahead/behind counts between `local` and `upstream` commit tips.
    ///
    /// See [`AheadBehind`] for the full contract and edge cases.
    fn ahead_behind(&self, local: &CommitId, upstream: &CommitId) -> Result<AheadBehind>;

    /// Returns a lightweight metadata snapshot of the repository.
    ///
    /// Includes backend kind, working tree path, HEAD state, object format,
    /// and backend capabilities. All fields are a fresh read; this is not a
    /// subscription.
    fn repository_info(&self) -> Result<RepositoryInfo>;

    // ── Optional-empty methods ─────────────────────────────────────────── //
    //
    // Returning empty is semantically valid when the backend has no such data.

    /// Returns the fetch URL of the named remote, or `Ok(None)` if not configured.
    ///
    /// Returns `Err` only on an actual I/O or config parsing failure.
    ///
    /// Default: `Ok(None)`.
    fn remote_url(&self, _name: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Returns metadata for all submodules declared in `.gitmodules`.
    ///
    /// Default: empty `Vec`.
    fn submodules(&self) -> Result<Vec<SubmoduleInfo>> {
        Ok(Vec::new())
    }

    /// Returns all stash entries, newest first.
    ///
    /// Default: empty `Vec`.
    fn stash_entries(&self) -> Result<Vec<StashEntry>> {
        Ok(Vec::new())
    }

    /// Returns all linked worktrees. The main worktree is not included.
    ///
    /// Default: empty `Vec`.
    fn worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        Ok(Vec::new())
    }

    // ── Write-side exception methods ───────────────────────────────────── //
    //
    // Tags are the only in-scope writes. Custom backends that do not support
    // tag operations receive unsupported-feature errors by default.

    /// Creates a lightweight tag at HEAD.
    ///
    /// Default: unsupported-feature error.
    fn create_tag(&self, _name: &str) -> Result<()> {
        return Err(crate::error::Error::UnsupportedBackendFeature { backend: None, feature: "create_tag" })
    }

    /// Creates an annotated tag at HEAD.
    ///
    /// Requires `user.name` and `user.email` in git config. On the jj
    /// backend this returns an unsupported error; use
    /// [`create_tag`][VcsBackend::create_tag] instead.
    ///
    /// Default: unsupported-feature error.
    fn create_annotated_tag(&self, _name: &str, _message: &str) -> Result<()> {
        return Err(crate::error::Error::UnsupportedBackendFeature { backend: None, feature: "create_annotated_tag" })
    }

    /// Deletes the named tag.
    ///
    /// Default: unsupported-feature error.
    fn delete_tag(&self, _name: &str) -> Result<()> {
        return Err(crate::error::Error::UnsupportedBackendFeature { backend: None, feature: "delete_tag" })
    }

    // ── Optional-unsupported methods ───────────────────────────────────── //
    //
    // These have a truthful default but backends should override where possible.

    /// Returns tracking metadata and divergence data for `branch`.
    ///
    /// Default: unsupported-feature error.
    fn branch_tracking(&self, _branch: &str) -> Result<BranchTrackingInfo> {
        return Err(crate::error::Error::UnsupportedBackendFeature { backend: None, feature: "branch_tracking" })
    }

    /// Returns tracking metadata for all local branches, sorted ascending
    /// by full ref name.
    ///
    /// Default: unsupported-feature error.
    fn local_branch_tracking(&self) -> Result<Vec<BranchTrackingInfo>> {
        return Err(crate::error::Error::UnsupportedBackendFeature { backend: None, feature: "local_branch_tracking" })
    }

    /// Returns `true` if `branch` has been merged into `target`.
    ///
    /// Equivalent to `is_ancestor(branch_tip, target_tip)` but with named
    /// branches, preventing callers from accidentally reversing the arguments.
    ///
    /// Default: unsupported-feature error.
    fn is_merged_into(&self, _branch: &str, _target: &str) -> Result<bool> {
        Err(crate::error::Error::UnsupportedBackendFeature {
            backend: None,
            feature: "is_merged_into",
        })
    }

    /// Returns ahead/behind counts for the configured upstream of `branch`.
    ///
    /// Returns `Ok(None)` when the branch has no configured upstream.
    /// Returns an error when the configured upstream ref no longer exists
    /// locally.
    ///
    /// Default: unsupported-feature error.
    fn branch_ahead_behind(&self, _branch: &str) -> Result<Option<AheadBehind>> {
        return Err(crate::error::Error::UnsupportedBackendFeature { backend: None, feature: "branch_ahead_behind" })
    }

    // ── Operation and conflict state (RFC 008) ─────────────────────────── //

    /// Returns the current in-progress repository operation, if any.
    ///
    /// Reads Git marker files (`MERGE_HEAD`, `rebase-merge/`, `rebase-apply/`,
    /// `CHERRY_PICK_HEAD`, `REVERT_HEAD`, `BISECT_LOG`, `refs/bisect/`).
    ///
    /// Returns `Ok(OperationState::None)` when no operation is in progress.
    ///
    /// Default: unsupported-feature error.
    fn operation_state(&self) -> Result<OperationState> {
        Err(crate::error::Error::UnsupportedBackendFeature { backend: None, feature: "operation_state" })
    }

    /// Returns paths with unmerged (higher-stage) index entries.
    ///
    /// Returns a sorted, deduplicated list. Empty when no conflicts exist.
    ///
    /// Default: unsupported-feature error.
    fn unmerged_paths(&self) -> Result<Vec<std::path::PathBuf>> {
        Err(crate::error::Error::UnsupportedBackendFeature { backend: None, feature: "unmerged_paths" })
    }

    /// Returns a structured summary of all conflicted index entries.
    ///
    /// Includes per-stage object IDs. For a lighter-weight check,
    /// prefer [`unmerged_paths`][Self::unmerged_paths].
    ///
    /// Default: unsupported-feature error.
    fn conflict_summary(&self) -> Result<ConflictSummary> {
        Err(crate::error::Error::UnsupportedBackendFeature { backend: None, feature: "conflict_summary" })
    }

    // ── Point-in-time reads (RFC 010) ──────────────────────────────────── //

    /// Per-line commit attribution for `path` at `commit_id` (not HEAD).
    ///
    /// Default: unsupported-feature error.
    fn blame_at(&self, _path: &std::path::Path, _commit_id: &CommitId) -> Result<Vec<BlameEntry>> {
        Err(crate::error::Error::UnsupportedBackendFeature { backend: None, feature: "blame_at" })
    }

    /// Non-recursive root-level tree listing at `commit_id`, sorted ascending by name.
    ///
    /// Default: unsupported-feature error.
    fn tree_at_commit(&self, _commit_id: &CommitId) -> Result<Vec<TreeEntry>> {
        Err(crate::error::Error::UnsupportedBackendFeature { backend: None, feature: "tree_at_commit" })
    }

    /// Non-recursive tree listing of the directory at `path` within `commit_id`.
    ///
    /// Returns `Err(PathNotFound)` if `path` does not exist or is not a directory.
    /// Entries sorted ascending by name.
    ///
    /// Default: unsupported-feature error.
    fn tree_at_path(&self, _commit_id: &CommitId, _path: &std::path::Path) -> Result<Vec<TreeEntry>> {
        Err(crate::error::Error::UnsupportedBackendFeature { backend: None, feature: "tree_at_path" })
    }
}
