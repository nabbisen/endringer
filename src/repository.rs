//! Repository handle and constructors.
//!
//! Use [`repository`] to open a Git repository, or [`jj_repository`] to open
//! a Jujutsu repository. Both return the same [`Repository`] type, which
//! dispatches all operations to the appropriate backend.

use std::path::Path;
use std::time::SystemTime;

use anyhow::Result;

use crate::{
    backend::VcsBackend,
    git::GitBackend,
    jj::JjBackend,
    types::{
        BackendKind, BranchInfo, CommitId, CommitInfo, DiffSummary, SortOrder, StatusDigest,
        TagInfo,
    },
};

// ── Constructors ─────────────────────────────────────────────────────────── //

/// Opens a Git repository at `repo_path`.
///
/// # Example
///
/// ```no_run
/// use endringer::repository::repository;
///
/// let repo = repository(std::path::Path::new(".")).expect("open repo");
/// let digest = repo.status_digest().expect("status digest");
/// println!("{}", digest.current_branch);
/// ```
pub fn repository(repo_path: &Path) -> Result<Repository> {
    let backend = GitBackend::open(repo_path)?;
    Ok(Repository {
        backend: Box::new(backend),
        kind: BackendKind::Git,
    })
}

/// Opens a Jujutsu repository at `repo_path`.
///
/// The `jj` binary is **not** required. The repository's underlying git object
/// store is opened directly with gix.
///
/// # Example
///
/// ```no_run
/// use endringer::repository::jj_repository;
///
/// let repo = jj_repository(std::path::Path::new(".")).expect("open jj repo");
/// let digest = repo.status_digest().expect("status digest");
/// println!("{}", digest.current_branch);
/// ```
pub fn jj_repository(repo_path: &Path) -> Result<Repository> {
    let backend = JjBackend::open(repo_path)?;
    Ok(Repository {
        backend: Box::new(backend),
        kind: BackendKind::Jj,
    })
}

// ── Repository ───────────────────────────────────────────────────────────── //

/// Handle for all VCS operations.
///
/// Returned by [`repository`] (Git) or [`jj_repository`] (Jujutsu). All
/// methods are backend-agnostic; the active backend is visible via
/// [`Repository::backend_kind`].
pub struct Repository {
    backend: Box<dyn VcsBackend>,
    kind: BackendKind,
}

impl Repository {
    /// Returns which VCS backend this repository uses.
    pub fn backend_kind(&self) -> BackendKind {
        self.kind
    }

    // ── Status ─────────────────────────────────────────────────────────── //

    /// Returns a lightweight snapshot of the repository's current state.
    pub fn status_digest(&self) -> Result<StatusDigest> {
        self.backend.status_digest()
    }

    // ── Branches ───────────────────────────────────────────────────────── //

    /// Returns all local branches.
    pub fn local_branches(&self) -> Result<Vec<BranchInfo>> {
        self.backend.local_branches()
    }

    /// Returns all remote-tracking branches.
    pub fn remote_branches(&self) -> Result<Vec<BranchInfo>> {
        self.backend.remote_branches()
    }

    // ── Commits ────────────────────────────────────────────────────────── //

    /// Returns the full commit history reachable from HEAD.
    pub fn list_commits(&self) -> Result<Vec<CommitInfo>> {
        self.backend.list_commits()
    }

    /// Returns the full commit history sorted by `order`.
    ///
    /// ```no_run
    /// use endringer::{repository::repository, types::SortOrder};
    ///
    /// let repo = repository(std::path::Path::new(".")).expect("open repo");
    /// let commits = repo.list_commits_sorted(SortOrder::NewestFirst).expect("commits");
    /// ```
    pub fn list_commits_sorted(&self, order: SortOrder) -> Result<Vec<CommitInfo>> {
        self.backend.list_commits_sorted(order)
    }

    /// Returns commits whose **author** timestamp falls within `[since, until]`.
    pub fn log_since(&self, since: SystemTime, until: SystemTime) -> Result<Vec<CommitInfo>> {
        self.backend.log_since(since, until)
    }

    /// O(1) lookup of a single commit by its [`CommitId`].
    pub fn find_commit(&self, id: &CommitId) -> Result<CommitInfo> {
        self.backend.find_commit(id)
    }

    // ── Tags ───────────────────────────────────────────────────────────── //

    /// Returns all tags.
    pub fn list_tags(&self) -> Result<Vec<TagInfo>> {
        self.backend.list_tags()
    }

    /// Returns all tags sorted by `order`.
    ///
    /// ```no_run
    /// use endringer::{repository::repository, types::SortOrder};
    ///
    /// let repo = repository(std::path::Path::new(".")).expect("open repo");
    /// let tags = repo.list_tags_sorted(SortOrder::ByName).expect("tags");
    /// ```
    pub fn list_tags_sorted(&self, order: SortOrder) -> Result<Vec<TagInfo>> {
        self.backend.list_tags_sorted(order)
    }

    /// Creates a lightweight tag at HEAD.
    pub fn create_tag(&self, name: &str) -> Result<()> {
        self.backend.create_tag(name)
    }

    /// Creates an annotated tag at HEAD.
    pub fn create_annotated_tag(&self, name: &str, message: &str) -> Result<()> {
        self.backend.create_annotated_tag(name, message)
    }

    /// Deletes the named tag.
    pub fn delete_tag(&self, name: &str) -> Result<()> {
        self.backend.delete_tag(name)
    }

    // ── Diff ───────────────────────────────────────────────────────────── //

    /// Returns a file-level diff summary between two commits.
    pub fn diff(&self, from: &CommitId, to: &CommitId) -> Result<DiffSummary> {
        self.backend.diff(from, to)
    }

    // ── Remotes ────────────────────────────────────────────────────────── //

    /// Returns the fetch URL of the named remote, or `None` if not configured.
    pub fn remote_url(&self, name: &str) -> Option<String> {
        self.backend.remote_url(name)
    }
}

#[cfg(test)]
mod tests;
