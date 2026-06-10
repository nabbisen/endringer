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

use anyhow::Result;

use crate::types::{
    AheadBehind, BlameEntry, BranchInfo, BranchTrackingInfo, CommitId, CommitInfo,
    DiffSummary, RepositoryInfo, SortOrder, StashEntry, StatusDigest, SubmoduleInfo,
    TagInfo, WorktreeInfo, WorktreeStatus,
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

    /// Returns the fetch URL of the named remote, or `None` if not configured.
    ///
    /// Default: `None`.
    fn remote_url(&self, _name: &str) -> Option<String> {
        None
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
    fn create_tag(&self, name: &str) -> Result<()> {
        anyhow::bail!("backend does not support lightweight tag creation: {name:?}")
    }

    /// Creates an annotated tag at HEAD.
    ///
    /// Requires `user.name` and `user.email` in git config. On the jj
    /// backend this returns an unsupported error; use
    /// [`create_tag`][VcsBackend::create_tag] instead.
    ///
    /// Default: unsupported-feature error.
    fn create_annotated_tag(&self, name: &str, _message: &str) -> Result<()> {
        anyhow::bail!("backend does not support annotated tag creation: {name:?}")
    }

    /// Deletes the named tag.
    ///
    /// Default: unsupported-feature error.
    fn delete_tag(&self, name: &str) -> Result<()> {
        anyhow::bail!("backend does not support tag deletion: {name:?}")
    }

    // ── Optional-unsupported methods ───────────────────────────────────── //
    //
    // These have a truthful default but backends should override where possible.

    /// Returns tracking metadata and divergence data for `branch`.
    ///
    /// Default: unsupported-feature error.
    fn branch_tracking(&self, branch: &str) -> Result<BranchTrackingInfo> {
        anyhow::bail!("backend does not support branch_tracking({branch:?})")
    }

    /// Returns tracking metadata for all local branches, sorted ascending
    /// by full ref name.
    ///
    /// Default: unsupported-feature error.
    fn local_branch_tracking(&self) -> Result<Vec<BranchTrackingInfo>> {
        anyhow::bail!("backend does not support local_branch_tracking")
    }

    /// Returns `true` if `branch` has been merged into `target`.
    ///
    /// Equivalent to `is_ancestor(branch_tip, target_tip)` but with named
    /// branches, preventing callers from accidentally reversing the arguments.
    ///
    /// Default: unsupported-feature error.
    fn is_merged_into(&self, branch: &str, target: &str) -> Result<bool> {
        anyhow::bail!(
            "backend does not support is_merged_into({branch:?}, {target:?})"
        )
    }

    /// Returns ahead/behind counts for the configured upstream of `branch`.
    ///
    /// Returns `Ok(None)` when the branch has no configured upstream.
    /// Returns an error when the configured upstream ref no longer exists
    /// locally.
    ///
    /// Default: unsupported-feature error.
    fn branch_ahead_behind(&self, branch: &str) -> Result<Option<AheadBehind>> {
        anyhow::bail!("backend does not support branch_ahead_behind({branch:?})")
    }
}
