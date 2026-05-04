//! Integration tests for [`AsyncRepository`].
//!
//! Uses a fixture git repository so tests are environment-independent.

use std::path::{Path, PathBuf};
use std::process::Command;

use endringer_async::AsyncRepository;

struct Fixture {
    _dir: tempfile::TempDir,
    path: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().to_path_buf();

        let git = |args: &[&str]| {
            assert!(
                Command::new("git").args(args).current_dir(&path).status().unwrap().success(),
                "git {} failed", args.join(" ")
            );
        };

        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "async@test.local"]);
        git(&["config", "user.name", "AsyncTest"]);

        std::fs::write(path.join("file.txt"), "hello\n").unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "async fixture commit"]);

        Fixture { _dir: dir, path }
    }

    fn path(&self) -> &Path { &self.path }
}

#[tokio::test]
async fn async_open_and_status_digest() {
    let f = Fixture::new();
    let repo = AsyncRepository::open(f.path()).await.expect("open");
    let digest = repo.status_digest().await.expect("status_digest");

    assert_eq!(digest.current_branch, "main");
    assert_eq!(digest.last_commit_summary, "async fixture commit");
    assert_eq!(digest.last_commit_id.short().len(), 7);
}

#[tokio::test]
async fn async_local_branches() {
    let f = Fixture::new();
    let repo = AsyncRepository::open(f.path()).await.unwrap();
    let branches = repo.local_branches().await.unwrap();

    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].name, "main");
}

#[tokio::test]
async fn async_list_commits() {
    let f = Fixture::new();
    let repo = AsyncRepository::open(f.path()).await.unwrap();
    let commits = repo.list_commits().await.unwrap();

    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].author, "AsyncTest");
    assert_eq!(commits[0].summary, "async fixture commit");
}

#[tokio::test]
async fn async_find_commit() {
    let f = Fixture::new();
    let repo = AsyncRepository::open(f.path()).await.unwrap();
    let commits = repo.list_commits().await.unwrap();
    let found = repo.find_commit(commits[0].commit_id.clone()).await.unwrap();

    assert_eq!(found.commit_id, commits[0].commit_id);
    assert_eq!(found.summary, commits[0].summary);
}

#[tokio::test]
async fn async_clone_shares_state() {
    let f = Fixture::new();
    let repo = AsyncRepository::open(f.path()).await.unwrap();
    let clone = repo.clone();

    let (d1, d2) = tokio::join!(repo.status_digest(), clone.status_digest());
    assert_eq!(d1.unwrap().last_commit_id, d2.unwrap().last_commit_id);
}

#[tokio::test]
async fn async_create_and_delete_tag() {
    let f = Fixture::new();
    let repo = AsyncRepository::open(f.path()).await.unwrap();

    repo.create_tag("v0.1.0-async".to_owned()).await.unwrap();
    let tags = repo.list_tags().await.unwrap();
    assert!(tags.iter().any(|t| t.name == "v0.1.0-async"));

    repo.delete_tag("v0.1.0-async".to_owned()).await.unwrap();
    let tags = repo.list_tags().await.unwrap();
    assert!(!tags.iter().any(|t| t.name == "v0.1.0-async"));
}

#[tokio::test]
async fn async_open_invalid_path() {
    assert!(AsyncRepository::open(Path::new("/no/such/repo")).await.is_err());
}
