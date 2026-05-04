//! Repository handle and constructor.
//!
//! The entry point is the [`repository`] function, which opens a Git
//! repository at a given path and returns a [`Repository`] handle.  All VCS
//! operations are exposed as methods on that handle; the underlying `gix`
//! types are never part of the public API.

use std::path::Path;
use std::time::SystemTime;

use anyhow::{Context, Result};

use crate::types::{BranchInfo, CommitInfo, StatusDigest, TagInfo};

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

    /// Returns the full commit history reachable from HEAD, newest first.
    ///
    /// For large repositories, prefer [`log_since`][Self::log_since] to
    /// limit the walk to a specific time window.
    pub fn list_commits(&self) -> Result<Vec<CommitInfo>> {
        branch::list_commits(&self.inner)
    }

    /// Returns commits reachable from HEAD whose **author** timestamp falls
    /// within `[since, until]` (inclusive on both ends).
    ///
    /// The filter uses author time (the timestamp stored on the author
    /// signature), consistent with `git log`'s default display.  Because Git
    /// history is a DAG and commit timestamps are author-controlled, every
    /// ancestor is inspected.  This is correct but `O(n)` in history depth;
    /// use with reasonably narrow time windows on large repositories.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::time::{SystemTime, Duration};
    /// use endringer::repository::repository;
    ///
    /// let repo = repository(std::path::Path::new(".")).expect("open repo");
    /// let until = SystemTime::now();
    /// let since = until - Duration::from_secs(7 * 24 * 3600); // last week
    /// let commits = repo.log_since(since, until).expect("log_since");
    /// ```
    pub fn log_since(&self, since: SystemTime, until: SystemTime) -> Result<Vec<CommitInfo>> {
        branch::log_since(&self.inner, since, until)
    }

    /// Returns a lightweight snapshot of the repository's current state
    /// (branch name, HEAD commit ID, HEAD commit message, timestamp).
    ///
    /// This is cheaper than a full history walk and suitable for use as a
    /// change-detection probe.
    pub fn status_digest(&self) -> Result<StatusDigest> {
        commit::status_digest(&self.inner)
    }

    // ------------------------------------------------------------------ //
    // Tag operations
    // ------------------------------------------------------------------ //

    /// Returns metadata for every tag in the repository.
    ///
    /// Both lightweight and annotated tags are returned; annotated tag objects
    /// are automatically peeled to their underlying commit.
    pub fn list_tags(&self) -> Result<Vec<TagInfo>> {
        tag::list_tags(&self.inner)
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
}
