//! [`GitBackend`] — wraps `gix::ThreadSafeRepository` and implements [`VcsBackend`].

use std::time::SystemTime;

use anyhow::Result;
use endringer_core::backend::VcsBackend;
use endringer_core::types::{BlameEntry, BranchInfo, CommitId, CommitInfo, DiffSummary, SortOrder, StatusDigest, TagInfo};

use crate::{blame, branch, commit, diff, graph, status, tag};

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
    pub fn open(path: &std::path::Path) -> Result<Self> {
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

impl VcsBackend for GitBackend {
    fn status_digest(&self) -> Result<StatusDigest> {
        commit::status_digest(&repo!(self))
    }

    fn local_branches(&self) -> Result<Vec<BranchInfo>> {
        branch::local_branches(&repo!(self))
    }

    fn remote_branches(&self) -> Result<Vec<BranchInfo>> {
        branch::remote_branches(&repo!(self))
    }

    fn list_commits(&self) -> Result<Vec<CommitInfo>> {
        branch::list_commits(&repo!(self))
    }

    fn list_commits_sorted(&self, order: SortOrder) -> Result<Vec<CommitInfo>> {
        branch::list_commits_sorted(&repo!(self), order)
    }

    fn log_since(&self, since: SystemTime, until: SystemTime) -> Result<Vec<CommitInfo>> {
        branch::log_since(&repo!(self), since, until)
    }

    fn find_commit(&self, id: &CommitId) -> Result<CommitInfo> {
        branch::find_commit(&repo!(self), id)
    }

    fn list_tags(&self) -> Result<Vec<TagInfo>> {
        tag::list_tags(&repo!(self))
    }

    fn list_tags_sorted(&self, order: SortOrder) -> Result<Vec<TagInfo>> {
        tag::list_tags_sorted(&repo!(self), order)
    }

    fn create_tag(&self, name: &str) -> Result<()> {
        tag::create_tag(&repo!(self), name)
    }

    fn create_annotated_tag(&self, name: &str, message: &str) -> Result<()> {
        tag::create_annotated_tag(&repo!(self), name, message)
    }

    fn delete_tag(&self, name: &str) -> Result<()> {
        tag::delete_tag(&repo!(self), name)
    }

    fn diff(&self, from: &CommitId, to: &CommitId) -> Result<DiffSummary> {
        diff::diff(&repo!(self), from, to)
    }

    fn remote_url(&self, name: &str) -> Option<String> {
        let repo = repo!(self);
        let remote = repo.find_remote(name).ok()?;
        let url = remote.url(gix::remote::Direction::Fetch)?;
        Some(url.to_bstring().to_string())
    }

    fn is_dirty(&self) -> Result<bool> {
        status::is_dirty(&repo!(self))
    }

    fn merge_base(&self, a: &CommitId, b: &CommitId) -> Result<Option<CommitId>> {
        graph::merge_base(&repo!(self), a, b)
    }

    fn is_ancestor(&self, candidate: &CommitId, descendant: &CommitId) -> Result<bool> {
        graph::is_ancestor(&repo!(self), candidate, descendant)
    }

    fn blame(&self, path: &std::path::Path) -> Result<Vec<BlameEntry>> {
        blame::blame(&repo!(self), path)
    }
}
