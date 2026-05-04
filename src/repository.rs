//! Repository handle and constructor.
//!
//! The entry point is the [`repository`] function, which opens a Git
//! repository at a given path and returns a [`Repository`] handle.  All VCS
//! operations are exposed as methods on that handle; the underlying `gix`
//! types are never part of the public API.

use std::path::Path;
use std::time::SystemTime;

use anyhow::{Context, Result};

use crate::types::{BranchInfo, CommitInfo, SortOrder, StatusDigest, TagInfo};

pub(crate) mod branch;
pub(crate) mod commit;
pub(crate) mod tag;

/// A handle to an open Git repository.
///
/// Obtain one via [`repository`].  All methods return owned data and do not
/// hold locks on the repository after they return, so multiple calls can be
/// interleaved freely.
///
/// # Example
///
/// ```no_run
/// use endringer::repository::repository;
///
/// let repo = repository(std::path::Path::new(".")).expect("open repo");
/// let digest = repo.status_digest().expect("status_digest");
/// println!("on branch {}", digest.current_branch);
/// ```
#[derive(Clone, Debug)]
pub struct Repository {
    inner: gix::Repository,
}

impl Repository {
    // ------------------------------------------------------------------ //
    // Branch queries
    // ------------------------------------------------------------------ //

    /// Returns metadata for every local branch (`refs/heads/`), in the order
    /// they appear in the packed-refs / loose-refs store.
    pub fn local_branches(&self) -> Result<Vec<BranchInfo>> {
        branch::local_branches(&self.inner)
    }

    /// Returns metadata for every remote-tracking branch (`refs/remotes/`).
    pub fn remote_branches(&self) -> Result<Vec<BranchInfo>> {
        branch::remote_branches(&self.inner)
    }

    // ------------------------------------------------------------------ //
    // Commit queries
    // ------------------------------------------------------------------ //

    /// Returns a lightweight snapshot of the repository's current state
    /// (branch name, HEAD commit ID, HEAD commit message, timestamp).
    ///
    /// This is cheaper than a full history walk and suitable for use as a
    /// change-detection probe.
    pub fn status_digest(&self) -> Result<StatusDigest> {
        commit::status_digest(&self.inner)
    }

    // ------------------------------------------------------------------ //
    // Commit history
    // ------------------------------------------------------------------ //

    /// Returns the full commit history reachable from HEAD in ref-store order.
    ///
    /// Use [`list_commits_sorted`][Self::list_commits_sorted] for a stable
    /// ordering.
    pub fn list_commits(&self) -> Result<Vec<CommitInfo>> {
        branch::list_commits(&self.inner)
    }

    /// Returns the full commit history reachable from HEAD, sorted by `order`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use endringer::{repository::repository, types::SortOrder};
    ///
    /// let repo = repository(std::path::Path::new(".")).expect("open repo");
    /// let commits = repo.list_commits_sorted(SortOrder::NewestFirst).expect("commits");
    /// ```
    pub fn list_commits_sorted(&self, order: SortOrder) -> Result<Vec<CommitInfo>> {
        branch::list_commits_sorted(&self.inner, order)
    }

    /// Returns commits reachable from HEAD whose **author** timestamp falls
    /// within `[since, until]` (inclusive on both ends).
    ///
    /// The filter uses author time (the timestamp stored on the author
    /// signature), consistent with `git log`'s default display.  Because Git
    /// history is a DAG and commit timestamps are author-controlled, every
    /// ancestor is inspected.  This is correct but `O(n)` in history depth;
    /// use with reasonably narrow time windows on large repositories.
    pub fn log_since(&self, since: SystemTime, until: SystemTime) -> Result<Vec<CommitInfo>> {
        branch::log_since(&self.inner, since, until)
    }

    // ------------------------------------------------------------------ //
    // Tag operations
    // ------------------------------------------------------------------ //

    /// Returns metadata for every tag in the repository in ref-store order.
    ///
    /// Both lightweight and annotated tags are returned; annotated tag objects
    /// are automatically peeled to their underlying commit.
    pub fn list_tags(&self) -> Result<Vec<TagInfo>> {
        tag::list_tags(&self.inner)
    }

    /// Returns metadata for every tag, sorted by `order`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use endringer::{repository::repository, types::SortOrder};
    ///
    /// let repo = repository(std::path::Path::new(".")).expect("open repo");
    /// let tags = repo.list_tags_sorted(SortOrder::ByName).expect("tags");
    /// ```
    pub fn list_tags_sorted(&self, order: SortOrder) -> Result<Vec<TagInfo>> {
        tag::list_tags_sorted(&self.inner, order)
    }

    /// Creates a new **lightweight** tag pointing to the current HEAD commit.
    ///
    /// Returns an error if a tag with `name` already exists.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use endringer::repository::repository;
    ///
    /// let repo = repository(std::path::Path::new(".")).expect("open repo");
    /// repo.create_tag("v1.0.0").expect("create tag");
    /// ```
    pub fn create_tag(&self, name: &str) -> Result<()> {
        tag::create_tag(&self.inner, name)
    }

    /// Creates a new **annotated** tag pointing to the current HEAD commit.
    ///
    /// Unlike a lightweight tag (which is simply a ref pointing to a commit),
    /// an annotated tag is a full Git object that records the tagger's identity
    /// (from `user.name` / `user.email` in git config), the current timestamp,
    /// and a `message`.  Annotated tags are the standard choice for release
    /// milestones.
    ///
    /// Returns an error if a tag with `name` already exists, or if the
    /// repository has no configured user identity.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use endringer::repository::repository;
    ///
    /// let repo = repository(std::path::Path::new(".")).expect("open repo");
    /// repo.create_annotated_tag("v1.0.0", "Release v1.0.0").expect("annotated tag");
    /// ```
    pub fn create_annotated_tag(&self, name: &str, message: &str) -> Result<()> {
        tag::create_annotated_tag(&self.inner, name, message)
    }

    /// Deletes the tag with the given `name`.
    ///
    /// Returns an error if no tag with that name exists.
    pub fn delete_tag(&self, name: &str) -> Result<()> {
        tag::delete_tag(&self.inner, name)
    }
}

/// Opens the Git repository at `repo_path` and returns a [`Repository`]
/// handle.
///
/// The path may point to the working tree root or to the `.git` directory
/// itself.
///
/// # Errors
///
/// Returns an error if `repo_path` does not contain a valid Git repository.
pub fn repository(repo_path: &Path) -> Result<Repository> {
    let inner =
        gix::open(repo_path).context("failed to open git repository")?;
    Ok(Repository { inner })
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use crate::commit_id_to_short_id;

    use super::*;

    fn open() -> Repository {
        repository(Path::new(".")).expect("failed to open repository")
    }

    #[test]
    fn it_works_repository() {
        // Opening from a valid git working tree must succeed.
        assert!(repository(Path::new(".")).is_ok());
        // Opening from a non-existent path must fail.
        assert!(repository(Path::new("/no/such/repo")).is_err());
    }

    #[test]
    fn it_works_local_branches() {
        let branches = open().local_branches().expect("local_branches");
        println!("{:?}", branches);
        // The test repo has at least one local branch (the current one).
        assert!(!branches.is_empty(), "expected at least one local branch");
        // Every branch must have a non-empty name and a full ref.
        for b in &branches {
            assert!(!b.name.is_empty());
            assert!(b.full_name.starts_with("refs/heads/"));
            assert_eq!(b.last_commit_id.short().len(), 7);
        }
    }

    #[test]
    fn it_works_remote_branches() {
        // No remotes in this test repo — must succeed and return an empty list.
        let branches = open().remote_branches().expect("remote_branches");
        println!("{:?}", branches);
        // Shape checks (works even if the list is empty).
        for b in &branches {
            assert!(b.full_name.starts_with("refs/remotes/"));
        }
    }

    #[test]
    fn it_works_list_commits() {
        let commits = open().list_commits().expect("list_commits");
        println!("{:?}", commits);
        // The test repo has at least the initial archive commit.
        assert!(!commits.is_empty(), "expected at least one commit");
        // Every entry must have a valid 7-char short ID and a non-empty summary.
        for c in &commits {
            assert_eq!(c.commit_id.short().len(), 7);
            assert!(!c.summary.is_empty());
            assert!(!c.author.is_empty());
        }
        // Results are newest-first: each timestamp ≥ the next.
        for window in commits.windows(2) {
            assert!(
                window[0].timestamp >= window[1].timestamp,
                "commits should be newest-first"
            );
        }
    }

    #[test]
    fn it_works_status_digest() {
        let digest = open().status_digest().expect("status digest");
        println!("{:?}", digest);

        // repo_name should be the directory name, not "unknown".
        assert!(!digest.repo_name.is_empty());
        assert_ne!(digest.repo_name, "unknown", "workdir name should resolve");

        // current_branch must be a short name (no "refs/heads/" prefix).
        assert!(
            !digest.current_branch.starts_with("refs/"),
            "current_branch should be shortened, got: {}",
            digest.current_branch
        );
    }

    #[test]
    fn it_works_commit_id_to_short_id() {
        let digest = open().status_digest().expect("status digest");
        let short = commit_id_to_short_id(digest.last_commit_id);
        println!("{:?}", short);
        assert_eq!(short.len(), 7);
    }

    #[test]
    fn it_works_commit_id_short_method() {
        let digest = open().status_digest().expect("status digest");
        let short = digest.last_commit_id.short();
        assert_eq!(short.len(), 7);
    }

    #[test]
    fn it_works_log_since() {
        let until = SystemTime::now();
        let since = until - Duration::from_secs(365 * 24 * 3600); // last year

        let commits = open().log_since(since, until).expect("log_since");
        println!("{:?}", commits);

        // The test repo was created today; all commits must fall within range.
        assert!(!commits.is_empty(), "expected commits within the last year");

        // All returned commits must lie within [since, until].
        for c in &commits {
            assert!(c.timestamp >= since, "commit timestamp before 'since'");
            assert!(c.timestamp <= until, "commit timestamp after 'until'");
        }

        // Requesting a window in the distant past must return an empty list.
        let ancient = SystemTime::UNIX_EPOCH + Duration::from_secs(86400);
        let empty = open()
            .log_since(SystemTime::UNIX_EPOCH, ancient)
            .expect("log_since ancient");
        assert!(empty.is_empty(), "expected no commits in 1970");
    }

    #[test]
    fn it_works_list_tags() {
        // No tags in this repo at baseline — must succeed and return empty.
        let tags = open().list_tags().expect("list_tags");
        println!("{:?}", tags);
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

        // Clean up in case a previous run left it behind.
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

        // Clean up in case a previous run left it behind.
        let _ = repo.delete_tag(tag_name);

        repo.create_annotated_tag(tag_name, "Test annotated tag")
            .expect("create annotated tag");

        let tags = repo.list_tags().expect("list tags");
        let found = tags.iter().find(|t| t.name == tag_name);
        assert!(found.is_some(), "annotated tag must appear in list_tags");

        // Annotated tags are peeled to the commit — all fields must be valid.
        let t = found.unwrap();
        assert!(t.full_name.starts_with("refs/tags/"));
        assert_eq!(t.commit_id.short().len(), 7);
        assert!(!t.commit_summary.is_empty());

        repo.delete_tag(tag_name).expect("delete annotated tag");
        let tags = repo.list_tags().expect("list tags after delete");
        assert!(!tags.iter().any(|t| t.name == tag_name));
    }

    #[test]
    fn it_works_list_commits_sorted() {
        use crate::types::SortOrder;

        let repo = open();

        let newest_first = repo
            .list_commits_sorted(SortOrder::NewestFirst)
            .expect("sorted commits NewestFirst");
        // NewestFirst: each timestamp >= the next.
        for w in newest_first.windows(2) {
            assert!(
                w[0].timestamp >= w[1].timestamp,
                "NewestFirst order violated"
            );
        }

        let oldest_first = repo
            .list_commits_sorted(SortOrder::OldestFirst)
            .expect("sorted commits OldestFirst");
        // OldestFirst: each timestamp <= the next.
        for w in oldest_first.windows(2) {
            assert!(
                w[0].timestamp <= w[1].timestamp,
                "OldestFirst order violated"
            );
        }

        // Sorted sets are the same size and contain the same commit IDs.
        assert_eq!(newest_first.len(), oldest_first.len());
        let mut ids_a: Vec<_> = newest_first.iter().map(|c| c.commit_id.clone()).collect();
        let mut ids_b: Vec<_> = oldest_first.iter().map(|c| c.commit_id.clone()).collect();
        ids_a.sort_by_key(|id| id.to_string());
        ids_b.sort_by_key(|id| id.to_string());
        assert_eq!(ids_a, ids_b, "sorted variants must contain the same commits");
    }

    #[test]
    fn it_works_commit_id_from_hex() {
        use crate::types::CommitId;

        // Round-trip: get a known commit id, convert to hex, parse back.
        let digest = open().status_digest().expect("status digest");
        let hex = digest.last_commit_id.to_string();
        assert_eq!(hex.len(), 40);

        let parsed = CommitId::from_hex(&hex).expect("from_hex round-trip");
        assert_eq!(parsed, digest.last_commit_id);
        assert_eq!(parsed.short(), digest.last_commit_id.short());

        // Error cases.
        assert!(CommitId::from_hex("not-a-hash").is_err());
        assert!(CommitId::from_hex("abc123").is_err()); // too short
        assert!(CommitId::from_hex(&"z".repeat(40)).is_err()); // invalid char
        assert!(CommitId::from_hex(&"0".repeat(39)).is_err()); // 39 chars
    }
}
