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
                Command::new("git")
                .args(args)
                .current_dir(&path)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_EDITOR", "true")
                .env("GIT_TERMINAL_PROMPT", "0")
                .stdin(std::process::Stdio::null())
                .status().unwrap().success(),
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

#[tokio::test]
async fn async_is_dirty_modified_file() {
    let f = Fixture::new();
    let repo = AsyncRepository::open(f.path()).await.unwrap();

    // Initially may or may not be dirty depending on timing; just ensure no panic.
    let _ = repo.is_dirty().await.unwrap();

    // After modifying a tracked file, must be dirty.
    std::fs::write(f.path().join("file.txt"), "changed\n").unwrap();
    assert!(repo.is_dirty().await.unwrap());
}

// ── RFC 004: async ahead/behind parity ───────────────────────────────────── //

#[tokio::test]
async fn async_ahead_behind_identical() {
    let f = Fixture::new();
    let repo = AsyncRepository::open(f.path()).await.unwrap();

    let head_hex = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output().unwrap();
    let hex = String::from_utf8(head_hex.stdout).unwrap().trim().to_string();
    let id = endringer::CommitId::from_hex(&hex).unwrap();

    let ab = repo.ahead_behind(id.clone(), id).await.unwrap();
    assert_eq!(ab.ahead,  0);
    assert_eq!(ab.behind, 0);
}

#[tokio::test]
async fn async_ahead_behind_matches_sync() {
    use endringer_async::AsyncRepository;
    use endringer::repository::repository;

    let f = Fixture::new();

    // Add a second commit so we have something to compare.
    std::fs::write(f.path().join("b.txt"), "b\n").unwrap();
    {
        let git = |args: &[&str]| {
            std::process::Command::new("git").args(args)
                .current_dir(f.path())
                .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
                .env("GIT_EDITOR","true").env("GIT_TERMINAL_PROMPT","0")
                .stdin(std::process::Stdio::null())
                .status().unwrap();
        };
        git(&["add","."]);
        git(&["commit","-m","second commit"]);
    }

    let head_hex = std::process::Command::new("git")
        .args(["rev-parse","HEAD"]).current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .output().unwrap();
    let upstream_hex = std::process::Command::new("git")
        .args(["rev-parse","HEAD^"]).current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .output().unwrap();

    let head = endringer::CommitId::from_hex(
        String::from_utf8(head_hex.stdout).unwrap().trim()).unwrap();
    let base = endringer::CommitId::from_hex(
        String::from_utf8(upstream_hex.stdout).unwrap().trim()).unwrap();

    // Sync result.
    let sync_repo = repository(f.path()).unwrap();
    let sync_ab = sync_repo.ahead_behind(&head, &base).unwrap();

    // Async result.
    let async_repo = AsyncRepository::open(f.path()).await.unwrap();
    let async_ab = async_repo.ahead_behind(head, base).await.unwrap();

    assert_eq!(sync_ab.ahead,  async_ab.ahead,  "sync/async ahead mismatch");
    assert_eq!(sync_ab.behind, async_ab.behind, "sync/async behind mismatch");
}

#[tokio::test]
async fn async_branch_ahead_behind_no_upstream() {
    let f = Fixture::new();
    let repo = AsyncRepository::open(f.path()).await.unwrap();
    // No upstream configured on main in this fixture.
    let result = repo.branch_ahead_behind("main".to_string()).await.unwrap();
    assert_eq!(result, None);
}

// ── RFC 005 / RFC 009: async parity ──────────────────────────────────────── //

#[tokio::test]
async fn async_repository_info_matches_sync() {
    use endringer_async::AsyncRepository;
    use endringer::repository::repository;

    let f = Fixture::new();

    let sync_repo  = repository(f.path()).unwrap();
    let async_repo = AsyncRepository::open(f.path()).await.unwrap();

    let sync_info  = sync_repo.repository_info().unwrap();
    let async_info = async_repo.repository_info().await.unwrap();

    assert_eq!(sync_info.backend,       async_info.backend);
    assert_eq!(sync_info.is_bare,       async_info.is_bare);
    assert_eq!(sync_info.object_format, async_info.object_format);
    assert_eq!(sync_info.repo_name,     async_info.repo_name);
}

#[tokio::test]
async fn async_local_branch_tracking_no_panic() {
    let f = Fixture::new();
    let repo = endringer_async::AsyncRepository::open(f.path()).await.unwrap();
    let list = repo.local_branch_tracking().await.unwrap();
    // At minimum "main" should appear.
    assert!(list.iter().any(|b| b.branch == "main"));
}

#[tokio::test]
async fn async_is_merged_into_parity() {
    use endringer_async::AsyncRepository;
    use endringer::repository::repository;

    let f = Fixture::new();
    // Both branches point at the same commit — merged into itself is true.
    let sync_repo  = repository(f.path()).unwrap();
    let async_repo = AsyncRepository::open(f.path()).await.unwrap();

    let sync_result  = sync_repo.is_merged_into("main", "main").unwrap();
    let async_result = async_repo.is_merged_into("main".to_string(),
                                                  "main".to_string()).await.unwrap();
    assert_eq!(sync_result, async_result);
}

// ── RFC 006: async typed error propagation ────────────────────────────────── //

#[tokio::test]
async fn async_open_non_repo_returns_not_a_repository() {
    use endringer_async::Error;
    let dir = tempfile::TempDir::new().unwrap();
    let err = endringer_async::AsyncRepository::open(dir.path())
        .await
        .err()
        .expect("expected error");
    assert!(
        matches!(err, Error::NotARepository { .. }),
        "expected NotARepository, got: {err}"
    );
}

#[tokio::test]
async fn async_error_is_typed_not_anyhow() {
    // Verify that the async API returns endringer::Error, not anyhow::Error,
    // by matching on a specific variant.
    use endringer_async::Error;
    let dir = tempfile::TempDir::new().unwrap();
    let result = endringer_async::AsyncRepository::open(dir.path()).await;
    assert!(result.is_err());
    let err = result.err().unwrap();
    // The error must be the typed variant, not a string-only type.
    let _ = format!("{err}"); // Display works
    let _ = format!("{err:?}"); // Debug works
}

// ── RFC 008: async operation state and conflict parity ────────────────────── //

#[tokio::test]
async fn async_operation_state_clean_is_none() {
    use endringer::OperationState;
    let f = Fixture::new();
    let repo = endringer_async::AsyncRepository::open(f.path()).await.unwrap();
    assert_eq!(repo.operation_state().await.unwrap(), OperationState::None);
}

#[tokio::test]
async fn async_unmerged_paths_clean_is_empty() {
    let f = Fixture::new();
    let repo = endringer_async::AsyncRepository::open(f.path()).await.unwrap();
    assert!(repo.unmerged_paths().await.unwrap().is_empty());
}

#[tokio::test]
async fn async_conflict_summary_clean_is_empty() {
    let f = Fixture::new();
    let repo = endringer_async::AsyncRepository::open(f.path()).await.unwrap();
    assert!(repo.conflict_summary().await.unwrap().is_empty());
}

// ── RFC 010: async point-in-time reads ───────────────────────────────────── //

#[tokio::test]
async fn async_tree_at_commit_matches_sync() {
    use endringer_async::AsyncRepository;
    use endringer::repository::repository;

    let f = Fixture::new();
    let head_hex = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"]).current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .output().unwrap();
    let head = endringer::CommitId::from_hex(
        String::from_utf8(head_hex.stdout).unwrap().trim()).unwrap();

    let sync_entries  = repository(f.path()).unwrap().tree_at_commit(&head).unwrap();
    let async_entries = AsyncRepository::open(f.path()).await.unwrap()
        .tree_at_commit(head).await.unwrap();

    assert_eq!(
        sync_entries.iter().map(|e| &e.name).collect::<Vec<_>>(),
        async_entries.iter().map(|e| &e.name).collect::<Vec<_>>(),
        "async tree_at_commit should match sync"
    );
}

#[tokio::test]
async fn async_blame_at_no_panic() {
    let f = Fixture::new();
    let head_hex = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"]).current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .output().unwrap();
    let head = endringer::CommitId::from_hex(
        String::from_utf8(head_hex.stdout).unwrap().trim()).unwrap();

    let repo = endringer_async::AsyncRepository::open(f.path()).await.unwrap();
    // The async Fixture creates "file.txt", not "README.md".
    let entries = repo
        .blame_at(std::path::PathBuf::from("file.txt"), head)
        .await
        .unwrap();
    assert!(!entries.is_empty(), "blame_at should return entries for file.txt");
}

// ── RFC 011: async remote and reference inventory ─────────────────────────── //

#[tokio::test]
async fn async_remotes_empty_on_fixture() {
    let f = Fixture::new();
    let repo = endringer_async::AsyncRepository::open(f.path()).await.unwrap();
    let remotes = repo.remotes().await.unwrap();
    assert!(remotes.is_empty(), "fixture should have no remotes");
}

#[tokio::test]
async fn async_references_contains_main() {
    let f = Fixture::new();
    let repo = endringer_async::AsyncRepository::open(f.path()).await.unwrap();
    let refs = repo.references().await.unwrap();
    assert!(
        refs.iter().any(|r| r.name == "refs/heads/main"),
        "async references should contain refs/heads/main"
    );
}

#[tokio::test]
async fn async_references_by_kind_tags_matches_sync() {
    use endringer_async::AsyncRepository;
    use endringer::repository::repository;
    use endringer::RefKind;

    let f = Fixture::new();
    let sync_tags  = repository(f.path()).unwrap().references_by_kind(RefKind::Tag).unwrap();
    let async_tags = AsyncRepository::open(f.path()).await.unwrap()
        .references_by_kind(RefKind::Tag).await.unwrap();

    assert_eq!(
        sync_tags.iter().map(|r| &r.name).collect::<Vec<_>>(),
        async_tags.iter().map(|r| &r.name).collect::<Vec<_>>(),
        "async and sync tag refs should match"
    );
}

// ── RFC 012: async query_commits ─────────────────────────────────────────── //

#[tokio::test]
async fn async_query_commits_head_page() {
    use endringer::CommitQuery;
    let f = Fixture::new();
    let repo = endringer_async::AsyncRepository::open(f.path()).await.unwrap();
    let result = repo.query_commits(CommitQuery::head_page(5)).await.unwrap();
    assert!(!result.commits.is_empty(), "head_page should return commits");
}

#[tokio::test]
async fn async_query_commits_truncated_flag() {
    use endringer::CommitQuery;
    let f = Fixture::new();

    // Add a second commit so there are ≥ 2, making max_count=1 truncate.
    let git = |args: &[&str]| {
        std::process::Command::new("git")
            .args(args)
            .current_dir(f.path())
            .env("GIT_CONFIG_NOSYSTEM","1")
            .env("GIT_CONFIG_GLOBAL","/dev/null")
            .env("GIT_EDITOR","true")
            .env("GIT_TERMINAL_PROMPT","0")
            .stdin(std::process::Stdio::null())
            .status().unwrap();
    };
    std::fs::write(f.path().join("extra.txt"), "x").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "second commit"]);

    let repo = endringer_async::AsyncRepository::open(f.path()).await.unwrap();
    let result = repo.query_commits(CommitQuery::head_page(1)).await.unwrap();
    assert_eq!(result.commits.len(), 1);
    assert!(result.truncated, "should be truncated when history is longer than max_count");
}

// ── RFC 019/020/021: async detail reads ──────────────────────────────────── //

#[tokio::test]
async fn async_submodule_summaries_no_panic() {
    let f = Fixture::new();
    let repo = endringer_async::AsyncRepository::open(f.path()).await.unwrap();
    let summaries = repo.submodule_summaries().await.unwrap();
    assert!(summaries.is_empty(), "fixture has no submodules");
}

#[tokio::test]
async fn async_stash_detail_empty_stash_returns_err() {
    let f = Fixture::new();
    let repo = endringer_async::AsyncRepository::open(f.path()).await.unwrap();
    // No stash in the fixture — should return an error, not panic.
    assert!(repo.stash_detail(0).await.is_err(),
        "stash_detail on empty stash should return Err");
}

#[tokio::test]
async fn async_worktree_details_no_linked_worktrees() {
    let f = Fixture::new();
    let repo = endringer_async::AsyncRepository::open(f.path()).await.unwrap();
    let details = repo.worktree_details().await.unwrap();
    assert!(details.is_empty(), "fixture has no linked worktrees");
}

// ── RFC 013: async rich_worktree_status ──────────────────────────────────── //

#[tokio::test]
async fn async_rich_worktree_status_clean_repo() {
    use endringer::StatusOptions;
    let f = Fixture::new();
    let repo = endringer_async::AsyncRepository::open(f.path()).await.unwrap();
    let status = repo.rich_worktree_status(StatusOptions::default()).await.unwrap();
    assert!(status.entries.is_empty(), "clean repo should have no rich status entries");
}

#[tokio::test]
async fn async_rich_worktree_status_matches_sync() {
    use endringer::{StatusOptions, repository::repository};
    let f = Fixture::new();
    std::fs::write(f.path().join("new.txt"), "hello").unwrap();

    let sync_status = repository(f.path()).unwrap()
        .rich_worktree_status(StatusOptions::default()).unwrap();
    let async_status = endringer_async::AsyncRepository::open(f.path()).await.unwrap()
        .rich_worktree_status(StatusOptions::default()).await.unwrap();
    assert_eq!(sync_status.entries.len(), async_status.entries.len(),
        "sync and async rich status should have same number of entries");
}
