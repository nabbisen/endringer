//! Repository handle and constructors.
//!
//! Use [`repository`] to open a Git repository, or [`jj_repository`] to open
//! a Jujutsu repository.  Both return the same [`Repository`] type, which
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
/// Requires the `jj` binary to be on `$PATH`.
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
/// Returned by [`repository`] (Git) or [`jj_repository`] (Jujutsu).
/// All methods are backend-agnostic; the active backend is visible via
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

// ── Unit tests ───────────────────────────────────────────────────────────── //

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::time::{Duration, SystemTime};

    use super::*;

    fn open() -> Repository {
        repository(Path::new(".")).expect("failed to open repository")
    }

    #[test]
    fn it_works_repository() {
        assert!(repository(Path::new(".")).is_ok());
        assert!(repository(Path::new("/no/such/repo")).is_err());
    }

    #[test]
    fn it_works_backend_kind() {
        assert_eq!(open().backend_kind(), BackendKind::Git);
    }

    #[test]
    fn it_works_status_digest() {
        let digest = open().status_digest().expect("status digest");
        assert!(!digest.repo_name.is_empty());
        assert_ne!(digest.repo_name, "unknown");
        assert!(!digest.current_branch.starts_with("refs/"));
        assert_eq!(digest.last_commit_id.short().len(), 7);
    }

    #[test]
    fn it_works_local_branches() {
        let branches = open().local_branches().expect("local_branches");
        assert!(!branches.is_empty());
        for b in &branches {
            assert!(!b.name.is_empty());
            assert!(b.full_name.starts_with("refs/heads/"));
            assert_eq!(b.last_commit_id.short().len(), 7);
        }
    }

    #[test]
    fn it_works_remote_branches() {
        let branches = open().remote_branches().expect("remote_branches");
        for b in &branches {
            assert!(b.full_name.starts_with("refs/remotes/"));
        }
    }

    #[test]
    fn it_works_list_commits() {
        let commits = open().list_commits().expect("list_commits");
        assert!(!commits.is_empty());
        for c in &commits {
            assert_eq!(c.commit_id.short().len(), 7);
            assert!(!c.summary.is_empty());
            assert!(!c.author.is_empty());
        }
        for w in commits.windows(2) {
            assert!(w[0].timestamp >= w[1].timestamp, "commits must be newest-first");
        }
    }

    #[test]
    fn it_works_list_commits_sorted() {
        use crate::types::SortOrder;
        let repo = open();

        let newest_first = repo.list_commits_sorted(SortOrder::NewestFirst).expect("sorted");
        for w in newest_first.windows(2) {
            assert!(w[0].timestamp >= w[1].timestamp, "NewestFirst violated");
        }

        let oldest_first = repo.list_commits_sorted(SortOrder::OldestFirst).expect("sorted");
        for w in oldest_first.windows(2) {
            assert!(w[0].timestamp <= w[1].timestamp, "OldestFirst violated");
        }

        let mut ids_a: Vec<_> = newest_first.iter().map(|c| c.commit_id.to_string()).collect();
        let mut ids_b: Vec<_> = oldest_first.iter().map(|c| c.commit_id.to_string()).collect();
        ids_a.sort();
        ids_b.sort();
        assert_eq!(ids_a, ids_b);
    }

    #[test]
    fn it_works_log_since() {
        let until = SystemTime::now();
        let since = until - Duration::from_secs(365 * 24 * 3600);

        let commits = open().log_since(since, until).expect("log_since");
        assert!(!commits.is_empty());
        for c in &commits {
            assert!(c.timestamp >= since);
            assert!(c.timestamp <= until);
        }

        let ancient = SystemTime::UNIX_EPOCH + Duration::from_secs(86400);
        let empty = open().log_since(SystemTime::UNIX_EPOCH, ancient).expect("log_since ancient");
        assert!(empty.is_empty());
    }

    #[test]
    fn it_works_commit_id_from_hex() {
        use crate::types::CommitId;

        let digest = open().status_digest().expect("status digest");
        let hex = digest.last_commit_id.to_string();
        assert_eq!(hex.len(), 40);

        let parsed = CommitId::from_hex(&hex).expect("round-trip");
        assert_eq!(parsed, digest.last_commit_id);
        assert_eq!(parsed.short(), digest.last_commit_id.short());

        assert!(CommitId::from_hex("not-a-hash").is_err());
        assert!(CommitId::from_hex("abc123").is_err());
        assert!(CommitId::from_hex(&"z".repeat(40)).is_err());
        assert!(CommitId::from_hex(&"0".repeat(39)).is_err());
    }

    #[test]
    fn it_works_find_commit() {
        let repo = open();
        let commits = repo.list_commits().expect("list commits");
        let expected = &commits[0];
        let found = repo.find_commit(&expected.commit_id).expect("find_commit");

        assert_eq!(found.commit_id, expected.commit_id);
        assert_eq!(found.author, expected.author);
        assert_eq!(found.summary, expected.summary);
        assert!(!found.committer.is_empty());
    }

    #[test]
    fn it_works_commit_id_to_short_id() {
        let digest = open().status_digest().expect("status digest");
        let short = crate::commit_id_to_short_id(digest.last_commit_id);
        assert_eq!(short.len(), 7);
    }

    #[test]
    fn it_works_commit_info_committer_fields() {
        let commits = open().list_commits().expect("list commits");
        for c in &commits {
            assert!(!c.author.is_empty());
            assert!(!c.committer.is_empty());
            let y2020 = SystemTime::UNIX_EPOCH + Duration::from_secs(1_577_836_800);
            assert!(c.committer_timestamp >= y2020);
        }
    }

    #[test]
    fn it_works_list_tags() {
        let tags = open().list_tags().expect("list_tags");
        for t in &tags {
            assert!(t.full_name.starts_with("refs/tags/"));
            assert_eq!(t.commit_id.short().len(), 7);
            assert!(!t.commit_summary.is_empty());
        }
    }

    #[test]
    fn it_works_create_and_delete_tag() {
        let repo = open();
        let tag_name = "endringer-test-tag-temp";
        let _ = repo.delete_tag(tag_name);

        repo.create_tag(tag_name).expect("create tag");
        let tags = repo.list_tags().expect("list tags");
        assert!(tags.iter().any(|t| t.name == tag_name));

        repo.delete_tag(tag_name).expect("delete tag");
        let tags = repo.list_tags().expect("list tags after delete");
        assert!(!tags.iter().any(|t| t.name == tag_name));
    }

    #[test]
    fn it_works_create_and_delete_annotated_tag() {
        let repo = open();
        let tag_name = "endringer-annotated-test-tag-temp";
        let _ = repo.delete_tag(tag_name);

        repo.create_annotated_tag(tag_name, "Test annotated tag").expect("create annotated tag");
        let tags = repo.list_tags().expect("list tags");
        let found = tags.iter().find(|t| t.name == tag_name);
        assert!(found.is_some());

        let t = found.unwrap();
        assert!(t.full_name.starts_with("refs/tags/"));
        assert_eq!(t.commit_id.short().len(), 7);

        repo.delete_tag(tag_name).expect("delete annotated tag");
    }

    #[test]
    fn it_works_diff() {
        let repo = open();
        let commits = repo.list_commits().expect("list commits");
        if commits.len() < 2 {
            return;
        }

        let d = repo.diff(&commits[1].commit_id, &commits[0].commit_id).expect("diff");
        let total = d.added.len() + d.modified.len() + d.deleted.len();
        assert!(total > 0);

        let empty = repo.diff(&commits[0].commit_id, &commits[0].commit_id).expect("self diff");
        assert!(empty.added.is_empty() && empty.modified.is_empty() && empty.deleted.is_empty());
    }

    #[test]
    fn it_works_remote_url() {
        let url = open().remote_url("origin");
        assert!(url.is_none());
    }
}
