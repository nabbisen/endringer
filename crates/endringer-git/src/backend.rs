//! [`GitBackend`] — wraps `gix::ThreadSafeRepository` and implements [`VcsBackend`].

use std::time::SystemTime;

use endringer_core::error::{anyhow_to_backend, Error as CrateError, NotFoundKind, Result};
use endringer_core::backend::VcsBackend;
use endringer_core::types::{
    AheadBehind, BlameEntry, BranchInfo, BranchTrackingInfo, CommitId, CommitInfo,
    DiffSummary, RepositoryInfo, SortOrder, StashEntry, StatusDigest, SubmoduleInfo,
    TagInfo, WorktreeInfo, WorktreeStatus,
};

use crate::{blame, branch, commit, diff, graph, info, object, stash, status, submodule, tag, worktree};

/// Git backend.
///
/// Uses [`gix::ThreadSafeRepository`] so the struct is natively `Send + Sync`
/// without any mutex. Each method obtains a cheap thread-local repository view
/// via [`gix::ThreadSafeRepository::to_thread_local`], eliminating serialization
/// under concurrent async load.
pub struct GitBackend {
    inner: gix::ThreadSafeRepository,
}

impl GitBackend {
    /// Opens or discovers a Git repository at or above `path`.
    ///
    /// Traverses parent directories until it finds a `.git` directory (or a
    /// bare repository), so callers may pass any subdirectory of the worktree.
    pub fn open(path: &std::path::Path) -> anyhow::Result<Self> {
        let inner = gix::discover(path)?.into_sync();
        Ok(GitBackend { inner })
    }
}

/// Obtains a thread-local [`gix::Repository`] view from the shared handle.
///
/// This is a zero-copy operation: no re-opening of files, no locking.
macro_rules! repo {
    ($self:expr) => {
        $self.inner.to_thread_local()
    };
}

/// Converts an `anyhow::Result` to `endringer_core::Result` at the dispatch boundary.
macro_rules! be {
    ($e:expr) => {
        $e.map_err(anyhow_to_backend)
    };
}

impl VcsBackend for GitBackend {
    fn status_digest(&self) -> Result<StatusDigest> {
        be!(commit::status_digest(&repo!(self)))
    }

    fn local_branches(&self) -> Result<Vec<BranchInfo>> {
        be!(branch::local_branches(&repo!(self)))
    }

    fn remote_branches(&self) -> Result<Vec<BranchInfo>> {
        be!(branch::remote_branches(&repo!(self)))
    }

    fn list_commits(&self) -> Result<Vec<CommitInfo>> {
        be!(branch::list_commits(&repo!(self)))
    }

    fn list_commits_sorted(&self, order: SortOrder) -> Result<Vec<CommitInfo>> {
        be!(branch::list_commits_sorted(&repo!(self), order))
    }

    fn log_since(&self, since: SystemTime, until: SystemTime) -> Result<Vec<CommitInfo>> {
        be!(branch::log_since(&repo!(self), since, until))
    }

    fn find_commit(&self, id: &CommitId) -> Result<CommitInfo> {
        branch::find_commit(&repo!(self), id).map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") || msg.contains("commit '") && msg.contains("not found") {
                CrateError::NotFound { kind: NotFoundKind::Commit, name: id.to_string() }
            } else if msg.contains("not a commit") {
                CrateError::NotACommit { id: id.clone() }
            } else {
                anyhow_to_backend(e)
            }
        })
    }

    fn list_tags(&self) -> Result<Vec<TagInfo>> {
        be!(tag::list_tags(&repo!(self)))
    }

    fn list_tags_sorted(&self, order: SortOrder) -> Result<Vec<TagInfo>> {
        be!(tag::list_tags_sorted(&repo!(self), order))
    }

    fn create_tag(&self, name: &str) -> Result<()> {
        be!(tag::create_tag(&repo!(self), name))
    }

    fn create_annotated_tag(&self, name: &str, message: &str) -> Result<()> {
        be!(tag::create_annotated_tag(&repo!(self), name, message))
    }

    fn delete_tag(&self, name: &str) -> Result<()> {
        be!(tag::delete_tag(&repo!(self), name))
    }

    fn diff(&self, from: &CommitId, to: &CommitId) -> Result<DiffSummary> {
        be!(diff::diff(&repo!(self), from, to))
    }

    fn remote_url(&self, name: &str) -> Result<Option<String>> {
        let repo = repo!(self);
        let remote = match repo.find_remote(name) {
            Ok(r) => r,
            Err(e) => {
                let msg = e.to_string();
                // gix reports "The remote named \"X\" did not exist" for
                // unknown remotes — treat as absent, not an error.
                if msg.contains("did not exist")
                    || msg.contains("not found")
                    || msg.contains("does not exist")
                {
                    return Ok(None);
                }
                return Err(anyhow_to_backend(anyhow::anyhow!(msg)));
            }
        };
        let url = remote.url(gix::remote::Direction::Fetch);
        Ok(url.map(|u| u.to_bstring().to_string()))
    }

    fn is_dirty(&self) -> Result<bool> {
        be!(status::is_dirty(&repo!(self)))
    }

    fn merge_base(&self, a: &CommitId, b: &CommitId) -> Result<Option<CommitId>> {
        be!(graph::merge_base(&repo!(self), a, b))
    }

    fn is_ancestor(&self, candidate: &CommitId, descendant: &CommitId) -> Result<bool> {
        be!(graph::is_ancestor(&repo!(self), candidate, descendant))
    }

    fn ahead_behind(&self, local: &CommitId, upstream: &CommitId) -> Result<AheadBehind> {
        be!(graph::ahead_behind(&repo!(self), local, upstream))
    }

    fn branch_ahead_behind(&self, branch: &str) -> Result<Option<AheadBehind>> {
        be!(graph::branch_ahead_behind(&repo!(self), branch))
    }

    fn repository_info(&self) -> Result<RepositoryInfo> {
        be!(info::repository_info(&repo!(self), endringer_core::types::BackendKind::Git))
    }

    fn branch_tracking(&self, branch: &str) -> Result<BranchTrackingInfo> {
        be!(branch::branch_tracking(&repo!(self), branch))
    }

    fn local_branch_tracking(&self) -> Result<Vec<BranchTrackingInfo>> {
        be!(branch::local_branch_tracking(&repo!(self)))
    }

    fn is_merged_into(&self, b: &str, target: &str) -> Result<bool> {
        be!(branch::is_merged_into(&repo!(self), b, target))
    }

    fn blame(&self, path: &std::path::Path) -> Result<Vec<BlameEntry>> {
        be!(blame::blame(&repo!(self), path))
    }

    fn worktree_status(&self) -> Result<WorktreeStatus> {
        be!(status::worktree_status(&repo!(self)))
    }

    fn file_at_commit(&self, path: &std::path::Path, commit_id: &CommitId) -> Result<Vec<u8>> {
        object::file_at_commit(&repo!(self), path, commit_id).map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") || msg.contains("does not exist") {
                CrateError::PathNotFound {
                    path: path.to_path_buf(),
                    commit: Some(commit_id.clone()),
                }
            } else {
                anyhow_to_backend(e)
            }
        })
    }

    fn submodules(&self) -> Result<Vec<SubmoduleInfo>> {
        be!(submodule::submodules(&repo!(self)))
    }

    fn stash_entries(&self) -> Result<Vec<StashEntry>> {
        be!(stash::stash_entries(&repo!(self)))
    }

    fn worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        be!(worktree::worktrees(&repo!(self)))
    }
}
