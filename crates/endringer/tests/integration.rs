//! Integration tests against a deterministic fixture repository.
//!
//! The fixture is created fresh in a temporary directory for each test group,
//! so tests are fully independent of the host environment's git configuration
//! or commit history.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

use endringer::repository::{jj_repository, repository};
use endringer::SortOrder;

// ── Fixture helpers ───────────────────────────────────────────────────────── //

struct Fixture {
    _dir: tempfile::TempDir,
    path: PathBuf,
}

impl Fixture {
    /// Creates a minimal git repository with two commits and one tag.
    ///
    /// Layout:
    /// ```text
    ///   commit A  "initial commit"   (tagged v0.1.0)
    ///   commit B  "add feature"      ← HEAD, branch: main
    /// ```
    fn new() -> Self {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().to_path_buf();

        let git = |args: &[&str]| {
            let ok = Command::new("git")
                .args(args)
                .current_dir(&path)
                .status()
                .expect("git")
                .success();
            assert!(ok, "git {} failed", args.join(" "));
        };

        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "fixture@test.local"]);
        git(&["config", "user.name", "Fixture"]);

        // Commit A
        std::fs::write(path.join("README.md"), "# fixture\n").unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "initial commit"]);
        git(&["tag", "v0.1.0"]);

        // Commit B — adds a file so diff(A, B) is non-empty
        std::fs::write(path.join("src.rs"), "fn main() {}\n").unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "add feature"]);

        Fixture { _dir: dir, path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

// ── Git backend tests ─────────────────────────────────────────────────────── //

#[test]
fn fixture_repository_opens() {
    let f = Fixture::new();
    assert!(repository(f.path()).is_ok());
    assert!(repository(Path::new("/no/such/path")).is_err());
}

#[test]
fn fixture_status_digest() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let d = repo.status_digest().unwrap();

    assert_eq!(d.current_branch, "main");
    assert_eq!(d.last_commit_summary, "add feature");
    assert_eq!(d.last_commit_id.short().len(), 7);
    assert!(d.last_commit_timestamp > SystemTime::UNIX_EPOCH);
}

#[test]
fn fixture_local_branches() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let branches = repo.local_branches().unwrap();

    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].name, "main");
    assert!(branches[0].full_name.starts_with("refs/heads/"));
    assert_eq!(branches[0].last_commit_summary, "add feature");
}

#[test]
fn fixture_remote_branches_empty() {
    let f = Fixture::new();
    let branches = repository(f.path()).unwrap().remote_branches().unwrap();
    assert!(branches.is_empty());
}

#[test]
fn fixture_list_commits() {
    let f = Fixture::new();
    let commits = repository(f.path()).unwrap().list_commits().unwrap();

    assert_eq!(commits.len(), 2);
    assert_eq!(commits[0].summary, "add feature");
    assert_eq!(commits[1].summary, "initial commit");
    // Newest-first order
    assert!(commits[0].timestamp >= commits[1].timestamp);
    // Authors set
    for c in &commits {
        assert_eq!(c.author, "Fixture");
        assert_eq!(c.committer, "Fixture");
    }
}

#[test]
fn fixture_list_commits_sorted() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();

    let newest = repo.list_commits_sorted(SortOrder::NewestFirst).unwrap();
    assert_eq!(newest.len(), 2);
    // Each sort order must be internally consistent.
    assert!(newest[0].timestamp >= newest[1].timestamp);

    let oldest = repo.list_commits_sorted(SortOrder::OldestFirst).unwrap();
    assert!(oldest[0].timestamp <= oldest[1].timestamp);

    // Both orderings must contain the same set of commit IDs.
    let mut n_ids: Vec<_> = newest.iter().map(|c| c.commit_id.to_string()).collect();
    let mut o_ids: Vec<_> = oldest.iter().map(|c| c.commit_id.to_string()).collect();
    n_ids.sort();
    o_ids.sort();
    assert_eq!(n_ids, o_ids);
}

#[test]
fn fixture_log_since() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();

    let until = SystemTime::now();
    let since = until - Duration::from_secs(3600);
    let commits = repo.log_since(since, until).unwrap();
    assert!(!commits.is_empty());

    // Nothing in the ancient past
    let ancient = SystemTime::UNIX_EPOCH + Duration::from_secs(86400);
    let empty = repo.log_since(SystemTime::UNIX_EPOCH, ancient).unwrap();
    assert!(empty.is_empty());
}

#[test]
fn fixture_find_commit() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let commits = repo.list_commits().unwrap();

    let found = repo.find_commit(&commits[0].commit_id).unwrap();
    assert_eq!(found.commit_id, commits[0].commit_id);
    assert_eq!(found.summary, commits[0].summary);
    assert_eq!(found.author, "Fixture");
}

#[test]
fn fixture_commit_id_ord() {
    let f = Fixture::new();
    let commits = repository(f.path()).unwrap().list_commits().unwrap();
    let mut ids: Vec<_> = commits.iter().map(|c| c.commit_id.clone()).collect();
    let before = ids.clone();
    ids.sort();
    ids.sort(); // idempotent
    // Just verify it doesn't panic and is deterministic
    let mut again = before.clone();
    again.sort();
    assert_eq!(ids, again);
}

#[test]
fn fixture_list_tags() {
    let f = Fixture::new();
    let tags = repository(f.path()).unwrap().list_tags().unwrap();

    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "v0.1.0");
    assert!(tags[0].full_name.starts_with("refs/tags/"));
    assert_eq!(tags[0].commit_summary, "initial commit");
}

#[test]
fn fixture_create_and_delete_tag() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let name = "test-tag";

    repo.create_tag(name).unwrap();
    let tags = repo.list_tags().unwrap();
    assert!(tags.iter().any(|t| t.name == name));

    repo.delete_tag(name).unwrap();
    let tags = repo.list_tags().unwrap();
    assert!(!tags.iter().any(|t| t.name == name));
}

#[test]
fn fixture_create_annotated_tag() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let name = "v1.0.0-annotated";

    match repo.create_annotated_tag(name, "Release 1.0.0") {
        Err(e)
            if e.to_string().contains("user.name")
                || e.to_string().contains("committer")
                || e.to_string().contains("identity") =>
        {
            return; // no git identity in this environment; skip
        }
        Err(e) => panic!("unexpected error: {e}"),
        Ok(()) => {}
    }

    let tags = repo.list_tags().unwrap();
    assert!(tags.iter().any(|t| t.name == name));
    repo.delete_tag(name).unwrap();
}

#[test]
fn fixture_diff() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let commits = repo.list_commits().unwrap();

    // B vs A should show src.rs added
    let d = repo.diff(&commits[1].commit_id, &commits[0].commit_id).unwrap();
    assert!(d.added.iter().any(|p| p.ends_with("src.rs")));
    assert!(d.modified.is_empty());
    assert!(d.deleted.is_empty());

    // Diff with self is empty
    let empty = repo.diff(&commits[0].commit_id, &commits[0].commit_id).unwrap();
    assert!(empty.added.is_empty() && empty.modified.is_empty() && empty.deleted.is_empty());
}

#[test]
fn fixture_diff_paths_are_sorted() {
    let f = Fixture::new();
    // Add a few more files so we get multiple entries
    let git = |args: &[&str]| {
        Command::new("git").args(args).current_dir(f.path()).status().unwrap();
    };
    std::fs::write(f.path().join("z.rs"), "").unwrap();
    std::fs::write(f.path().join("a.rs"), "").unwrap();
    std::fs::write(f.path().join("m.rs"), "").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "multi-file"]);

    let repo = repository(f.path()).unwrap();
    let commits = repo.list_commits().unwrap();
    let d = repo.diff(&commits[1].commit_id, &commits[0].commit_id).unwrap();

    // Each category must be sorted
    for w in d.added.windows(2) {
        assert!(w[0] <= w[1], "added not sorted: {:?}", d.added);
    }
}

#[test]
fn fixture_remote_url_none() {
    let f = Fixture::new();
    assert!(repository(f.path()).unwrap().remote_url("origin").is_none());
}

// ── jj backend smoke test ─────────────────────────────────────────────────── //

#[test]
fn jj_repository_rejects_plain_git() {
    let f = Fixture::new();
    // A plain git repo (no .jj/) must be rejected.
    assert!(jj_repository(f.path()).is_err());
}
