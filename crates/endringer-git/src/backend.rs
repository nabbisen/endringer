//! [`GitBackend`] — wraps `gix::Repository` and implements [`VcsBackend`].

use std::sync::Mutex;
use std::time::SystemTime;

use anyhow::Result;
use endringer_core::backend::VcsBackend;
use endringer_core::types::{BranchInfo, CommitId, CommitInfo, DiffSummary, SortOrder, StatusDigest, TagInfo};

use crate::{branch, commit, diff, tag};

/// Git backend.
///
/// `gix::Repository` is `Send` but not `Sync` (interior mutability via
/// `RefCell`). The [`Mutex`] restores `Sync` while still permitting shared
/// references across threads.
pub struct GitBackend {
    inner: Mutex<gix::Repository>,
}

impl GitBackend {
    /// Opens a Git repository at `path` using gix.
    pub fn open(path: &std::path::Path) -> Result<Self> {
        let inner = gix::discover(path)?;
        Ok(GitBackend { inner: Mutex::new(inner) })
    }
}

/// Locks the inner `Mutex` and panics with a clear message on poison.
macro_rules! repo {
    ($self:expr) => {
        $self.inner.lock().expect("gix repository mutex poisoned")
    };
}

impl VcsBackend for GitBackend {
    fn status_digest(&self) -> Result<StatusDigest> {
        commit::status_digest(&*repo!(self))
    }

    fn local_branches(&self) -> Result<Vec<BranchInfo>> {
        branch::local_branches(&*repo!(self))
    }

    fn remote_branches(&self) -> Result<Vec<BranchInfo>> {
        branch::remote_branches(&*repo!(self))
    }

    fn list_commits(&self) -> Result<Vec<CommitInfo>> {
        branch::list_commits(&*repo!(self))
    }

    fn list_commits_sorted(&self, order: SortOrder) -> Result<Vec<CommitInfo>> {
        branch::list_commits_sorted(&*repo!(self), order)
    }

    fn log_since(&self, since: SystemTime, until: SystemTime) -> Result<Vec<CommitInfo>> {
        branch::log_since(&*repo!(self), since, until)
    }

    fn find_commit(&self, id: &CommitId) -> Result<CommitInfo> {
        branch::find_commit(&*repo!(self), id)
    }

    fn list_tags(&self) -> Result<Vec<TagInfo>> {
        tag::list_tags(&*repo!(self))
    }

    fn list_tags_sorted(&self, order: SortOrder) -> Result<Vec<TagInfo>> {
        tag::list_tags_sorted(&*repo!(self), order)
    }

    fn create_tag(&self, name: &str) -> Result<()> {
        tag::create_tag(&*repo!(self), name)
    }

    fn create_annotated_tag(&self, name: &str, message: &str) -> Result<()> {
        tag::create_annotated_tag(&*repo!(self), name, message)
    }

    fn delete_tag(&self, name: &str) -> Result<()> {
        tag::delete_tag(&*repo!(self), name)
    }

    fn diff(&self, from: &CommitId, to: &CommitId) -> Result<DiffSummary> {
        diff::diff(&*repo!(self), from, to)
    }

    fn remote_url(&self, name: &str) -> Option<String> {
        let repo = repo!(self);
        let remote = repo.find_remote(name).ok()?;
        let url = remote.url(gix::remote::Direction::Fetch)?;
        Some(url.to_bstring().to_string())
    }
}
