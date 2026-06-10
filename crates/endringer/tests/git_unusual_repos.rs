//! Tests for unusual repository states (RFC 024).
//!
//! Documents and verifies endringer's behaviour for:
//! - Empty/unborn repositories (no commits yet)
//! - Detached HEAD
//! - Bare repositories
//!
//! Each test section corresponds to one row of the method behaviour matrix
//! documented in `docs/src/reference/unusual-repositories.md`.

use std::path::Path;
use std::process::Command;
use endringer::repository::repository;
use endringer::{Error, HeadState};

// ── Fixtures ──────────────────────────────────────────────────────────────── //

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_EDITOR", "true")
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(std::process::Stdio::null())
        .status()
        .expect("git")
        .success();
    assert!(ok, "git {} failed", args.join(" "));
}

fn git_output(dir: &Path, args: &[&str]) -> String {
    String::from_utf8(
        Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .output()
            .expect("git")
            .stdout,
    )
    .unwrap()
    .trim()
    .to_owned()
}

/// Creates an empty git repo (no commits, unborn HEAD pointing at `main`).
fn unborn_fixture() -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().unwrap();
    git(dir.path(), &["init", "-b", "main"]);
    git(dir.path(), &["config", "user.email", "fixture@test.local"]);
    git(dir.path(), &["config", "user.name", "Fixture"]);
    dir
}

/// Creates a repo with one commit then detaches HEAD.
fn detached_fixture() -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().unwrap();
    git(dir.path(), &["init", "-b", "main"]);
    git(dir.path(), &["config", "user.email", "fixture@test.local"]);
    git(dir.path(), &["config", "user.name", "Fixture"]);
    std::fs::write(dir.path().join("README.md"), "# repo\n").unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-m", "initial"]);
    // Detach HEAD at HEAD.
    let sha = git_output(dir.path(), &["rev-parse", "HEAD"]);
    git(dir.path(), &["checkout", "--detach", &sha]);
    dir
}

/// Creates a bare git repository seeded with one commit.
fn bare_fixture() -> (tempfile::TempDir, tempfile::TempDir) {
    // Create a normal repo to clone from.
    let src = tempfile::TempDir::new().unwrap();
    git(src.path(), &["init", "-b", "main"]);
    git(src.path(), &["config", "user.email", "fixture@test.local"]);
    git(src.path(), &["config", "user.name", "Fixture"]);
    std::fs::write(src.path().join("README.md"), "# bare\n").unwrap();
    git(src.path(), &["add", "."]);
    git(src.path(), &["commit", "-m", "initial"]);

    let bare = tempfile::TempDir::new().unwrap();
    Command::new("git")
        .args(["clone", "--bare",
               src.path().to_str().unwrap(),
               bare.path().to_str().unwrap()])
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(std::process::Stdio::null())
        .status().unwrap();

    (src, bare)
}

// ── Empty / unborn repository ─────────────────────────────────────────────── //

#[test]
fn unborn_repository_opens_without_error() {
    let dir = unborn_fixture();
    assert!(
        repository(dir.path()).is_ok(),
        "empty/unborn repository should open"
    );
}

#[test]
fn unborn_repository_info_head_state_is_unborn() {
    let dir = unborn_fixture();
    let repo = repository(dir.path()).unwrap();
    let info = repo.repository_info().unwrap();
    assert!(
        matches!(info.head, HeadState::Unborn { .. }),
        "unborn repository head state should be Unborn; got {:?}", info.head
    );
}

#[test]
fn unborn_repository_list_commits_returns_empty() {
    let dir = unborn_fixture();
    let repo = repository(dir.path()).unwrap();
    // No commits yet — should return empty vec, not panic or error.
    match repo.list_commits() {
        Ok(commits) => assert!(commits.is_empty(),
            "unborn repo should have no commits"),
        Err(e) => assert!(
            // Acceptable: error indicating HEAD is unborn.
            e.to_string().contains("unborn")
                || e.to_string().contains("HEAD")
                || matches!(e, Error::NotFound { .. }),
            "unexpected error from list_commits on unborn repo: {e}"
        ),
    }
}

#[test]
fn unborn_repository_status_digest_errors_gracefully() {
    let dir = unborn_fixture();
    let repo = repository(dir.path()).unwrap();
    // status_digest requires HEAD to point to a commit — should error clearly.
    let result = repo.status_digest();
    assert!(
        result.is_err(),
        "status_digest should fail on unborn repository"
    );
}

#[test]
fn unborn_repository_local_branches_returns_empty() {
    let dir = unborn_fixture();
    let repo = repository(dir.path()).unwrap();
    let branches = repo.local_branches().unwrap();
    assert!(
        branches.is_empty(),
        "unborn repo should have no local branches"
    );
}

#[test]
fn unborn_repository_list_tags_returns_empty() {
    let dir = unborn_fixture();
    let repo = repository(dir.path()).unwrap();
    let tags = repo.list_tags().unwrap();
    assert!(tags.is_empty(), "unborn repo should have no tags");
}

// ── Detached HEAD ─────────────────────────────────────────────────────────── //

#[test]
fn detached_head_opens_without_error() {
    let dir = detached_fixture();
    assert!(repository(dir.path()).is_ok());
}

#[test]
fn detached_head_repository_info_state() {
    let dir = detached_fixture();
    let repo = repository(dir.path()).unwrap();
    let info = repo.repository_info().unwrap();
    assert!(
        matches!(info.head, HeadState::Detached { .. }),
        "detached HEAD should have Detached state; got {:?}", info.head
    );
}

#[test]
fn detached_head_status_digest_shows_detached_string() {
    let dir = detached_fixture();
    let repo = repository(dir.path()).unwrap();
    let digest = repo.status_digest().unwrap();
    assert_eq!(
        digest.current_branch, "(detached)",
        "detached HEAD status_digest.current_branch should be \"(detached)\""
    );
}

#[test]
fn detached_head_list_commits_succeeds() {
    let dir = detached_fixture();
    let repo = repository(dir.path()).unwrap();
    let commits = repo.list_commits().unwrap();
    assert!(
        !commits.is_empty(),
        "detached HEAD repo should still have commits"
    );
}

#[test]
fn detached_head_query_commits_head_page() {
    let dir = detached_fixture();
    let repo = repository(dir.path()).unwrap();
    let result = repo.query_commits(endringer::CommitQuery::head_page(10)).unwrap();
    assert!(!result.commits.is_empty(),
        "query_commits should work with detached HEAD");
    assert!(!result.truncated, "small page on small repo should not be truncated");
}

// ── Bare repository ───────────────────────────────────────────────────────── //

#[test]
fn bare_repository_opens_without_error() {
    let (_src, bare) = bare_fixture();
    assert!(repository(bare.path()).is_ok());
}

#[test]
fn bare_repository_list_commits_succeeds() {
    let (_src, bare) = bare_fixture();
    let repo = repository(bare.path()).unwrap();
    let commits = repo.list_commits().unwrap();
    assert!(!commits.is_empty(), "bare repo should have commits");
}

#[test]
fn bare_repository_local_branches_succeeds() {
    let (_src, bare) = bare_fixture();
    let repo = repository(bare.path()).unwrap();
    let branches = repo.local_branches().unwrap();
    assert!(!branches.is_empty(), "bare repo should have local branches");
}

#[test]
fn bare_repository_worktree_status_returns_empty_or_unsupported() {
    let (_src, bare) = bare_fixture();
    let repo = repository(bare.path()).unwrap();
    match repo.worktree_status() {
        Ok(ws) => {
            // Bare repos have no working tree; empty status is acceptable.
            assert!(ws.staged.is_empty() && ws.unstaged.is_empty() && ws.untracked.is_empty(),
                "bare repo worktree_status should be empty");
        }
        Err(e) => {
            // An error indicating no working tree is also acceptable.
            assert!(
                e.to_string().contains("bare")
                    || e.to_string().contains("worktree")
                    || e.to_string().contains("working")
                    || matches!(e, Error::UnsupportedBackendFeature { .. }),
                "unexpected worktree_status error on bare repo: {e}"
            );
        }
    }
}

#[test]
fn bare_repository_query_commits_head_page() {
    let (_src, bare) = bare_fixture();
    let repo = repository(bare.path()).unwrap();
    let result = repo.query_commits(endringer::CommitQuery::head_page(5)).unwrap();
    assert!(!result.commits.is_empty());
}

// ── query_commits behaviour ───────────────────────────────────────────────── //
// These live here because they exercise the same unusual-state logic above
// and naturally extend the fixture reuse.

#[test]
fn query_commits_max_count_limits_results() {
    // Use the detached fixture which has ≥ 1 commit.
    let dir = detached_fixture();
    let repo = repository(dir.path()).unwrap();

    // Seed more commits so we can test the limit meaningfully.
    for i in 0..5 {
        std::fs::write(dir.path().join(format!("file{i}.txt")), format!("v{i}")).unwrap();
        git(dir.path(), &["add", "."]);
        git(dir.path(), &["commit", "-m", &format!("add file{i}")]);
    }
    // Re-open after adding commits.
    let repo = repository(dir.path()).unwrap();

    let result = repo.query_commits(endringer::CommitQuery::head_page(2)).unwrap();
    assert_eq!(result.commits.len(), 2, "should return exactly max_count commits");
    assert!(result.truncated, "should be truncated when history is longer than max_count");
}

#[test]
fn query_commits_no_limit_returns_all() {
    let dir = detached_fixture();
    let repo = repository(dir.path()).unwrap();
    let all = repo.list_commits().unwrap();
    let queried = repo.query_commits(endringer::CommitQuery {
        start: endringer::CommitQueryStart::Head,
        max_count: None,
        skip: 0,
        since: None,
        until: None,
        order: endringer::SortOrder::NewestFirst,
    }).unwrap();
    assert_eq!(queried.commits.len(), all.len(),
        "unbounded query should match list_commits length");
    assert!(!queried.truncated);
}

#[test]
fn query_commits_from_ref() {
    // Need a real fixture with a named branch.
    use std::path::PathBuf;
    let dir = tempfile::TempDir::new().unwrap();
    let p: &PathBuf = &dir.path().to_path_buf();
    git(p, &["init", "-b", "main"]);
    git(p, &["config", "user.email", "fixture@test.local"]);
    git(p, &["config", "user.name", "Fixture"]);
    std::fs::write(p.join("f.txt"), "a").unwrap();
    git(p, &["add", "."]);
    git(p, &["commit", "-m", "first"]);

    let repo = repository(p).unwrap();
    let result = repo.query_commits(endringer::CommitQuery {
        start: endringer::CommitQueryStart::Ref("main".to_owned()),
        max_count: Some(10),
        skip: 0, since: None, until: None,
        order: endringer::SortOrder::NewestFirst,
    }).unwrap();
    assert!(!result.commits.is_empty(),
        "query from Ref(main) should return commits");
}

#[test]
fn query_commits_skip_offsets_results() {
    let dir = tempfile::TempDir::new().unwrap();
    let p = dir.path();
    git(p, &["init", "-b", "main"]);
    git(p, &["config", "user.email", "fixture@test.local"]);
    git(p, &["config", "user.name", "Fixture"]);
    for i in 0..4 {
        std::fs::write(p.join(format!("f{i}.txt")), "x").unwrap();
        git(p, &["add", "."]);
        git(p, &["commit", "-m", &format!("commit {i}")]);
    }

    let repo = repository(p).unwrap();
    let all    = repo.list_commits().unwrap();
    let paged  = repo.query_commits(endringer::CommitQuery {
        start: endringer::CommitQueryStart::Head,
        max_count: None, skip: 1,
        since: None, until: None,
        order: endringer::SortOrder::NewestFirst,
    }).unwrap();
    assert_eq!(paged.commits.len(), all.len() - 1,
        "skip=1 should omit the first (newest) commit");
}
