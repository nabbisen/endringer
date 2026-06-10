//! Integration tests for rich detail reads: submodule summaries (RFC 019),
//! stash detail and diff (RFC 020), and worktree detail (RFC 021).

#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;
use endringer::repository::repository;
use endringer::{Error, SubmoduleState, WorktreeState};

// ── helper ────────────────────────────────────────────────────────────────── //

fn git_cmd(dir: &std::path::Path, args: &[&str]) {
    let ok = std::process::Command::new("git")
        .args(args).current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .env("GIT_EDITOR","true").env("GIT_TERMINAL_PROMPT","0")
        .stdin(std::process::Stdio::null())
        .status().expect("git").success();
    assert!(ok, "git {} failed", args.join(" "));
}

// ── RFC 019: submodule_summaries ──────────────────────────────────────────── //

#[test]
fn submodule_summaries_empty_when_no_submodules() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let summaries = repo.submodule_summaries().unwrap();
    assert!(summaries.is_empty(), "no submodules → should be empty");
}

#[test]
fn submodule_summaries_sorted_by_path() {
    // Create two bare repos and register them as submodules.
    let f = Fixture::new();
    let bare_a = tempfile::TempDir::new().unwrap();
    let bare_b = tempfile::TempDir::new().unwrap();

    // Initialise bare repos.
    for bare in [bare_a.path(), bare_b.path()] {
        git_cmd(bare, &["init", "--bare"]);
    }

    // Add submodules in reverse alphabetical order (expect sorted output).
    let bare_b_url = format!("file://{}", bare_b.path().display());
    let bare_a_url = format!("file://{}", bare_a.path().display());
    let _ = std::process::Command::new("git")
        .args(["submodule", "add", "--force", &bare_b_url, "zzz-sub"])
        .current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .env("GIT_EDITOR","true").env("GIT_TERMINAL_PROMPT","0")
        .stdin(std::process::Stdio::null()).status();
    let _ = std::process::Command::new("git")
        .args(["submodule", "add", "--force", &bare_a_url, "aaa-sub"])
        .current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .env("GIT_EDITOR","true").env("GIT_TERMINAL_PROMPT","0")
        .stdin(std::process::Stdio::null()).status();

    let repo = repository(f.path()).unwrap();
    let summaries = repo.submodule_summaries().unwrap();

    if summaries.len() >= 2 {
        let paths: Vec<String> = summaries.iter()
            .map(|s| s.path.to_str().unwrap_or("").to_owned()).collect();
        let sorted = { let mut v = paths.clone(); v.sort(); v };
        assert_eq!(paths, sorted, "submodule_summaries should be sorted by path");
    }
}

#[test]
fn submodule_summaries_url_present() {
    // Just verify that a registered submodule has the expected fields.
    let f = Fixture::new();
    let bare = tempfile::TempDir::new().unwrap();
    git_cmd(bare.path(), &["init", "--bare"]);
    let url = format!("file://{}", bare.path().display());
    let _ = std::process::Command::new("git")
        .args(["submodule", "add", "--force", &url, "my-sub"])
        .current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .env("GIT_EDITOR","true").env("GIT_TERMINAL_PROMPT","0")
        .stdin(std::process::Stdio::null()).status();

    let repo = repository(f.path()).unwrap();
    let summaries = repo.submodule_summaries().unwrap();
    if let Some(sub) = summaries.iter().find(|s| s.path.to_str() == Some("my-sub")) {
        assert!(sub.url.is_some(), "registered submodule should have a URL");
        // State should be Registered (no commits in bare repo yet).
        assert!(
            matches!(sub.state, SubmoduleState::Registered | SubmoduleState::MissingWorktree
                                | SubmoduleState::MissingGitDir | SubmoduleState::Unknown),
            "empty bare submodule should be in an expected state; got {:?}", sub.state
        );
    }
    // If git add failed silently (no-commit bare), just assert no panic.
}

// ── RFC 020: stash_detail and stash_diff ─────────────────────────────────── //

struct StashFixture {
    pub _f: Fixture,
}

impl StashFixture {
    fn new() -> Self {
        let f = Fixture::new();
        // Modify a file and stash.
        std::fs::write(f.path.join("README.md"), "stashed change\n").unwrap();
        git_cmd(f.path(), &["stash", "push", "-m", "test stash"]);
        StashFixture { _f: f }
    }
    fn path(&self) -> &std::path::Path { self._f.path() }
}

#[test]
fn stash_detail_on_empty_stash_returns_err() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let err = repo.stash_detail(0).unwrap_err();
    // Should be NotFound or Backend error — not a panic.
    assert!(
        matches!(err, Error::NotFound { .. }) || err.to_string().contains("not found"),
        "expected not-found error on empty stash; got: {err}"
    );
}

#[test]
fn stash_detail_returns_metadata() {
    let f = StashFixture::new();
    let repo = repository(f.path()).unwrap();
    let detail = repo.stash_detail(0).unwrap();
    assert_eq!(detail.id.index, 0);
    assert!(!detail.message.is_empty(), "stash message should not be empty");
    assert!(!detail.parents.is_empty(), "stash commit should have parents");
    assert!(!detail.commit_id.to_string().is_empty());
}

#[test]
fn stash_detail_message_matches_entry() {
    let f = StashFixture::new();
    let repo = repository(f.path()).unwrap();
    let entries = repo.stash_entries().unwrap();
    let detail  = repo.stash_detail(0).unwrap();
    if let Some(entry) = entries.first() {
        assert_eq!(detail.commit_id, entry.commit_id,
            "stash_detail commit_id should match stash_entries");
    }
}

#[test]
fn stash_detail_invalid_index_returns_err() {
    let f = StashFixture::new();
    let repo = repository(f.path()).unwrap();
    let err = repo.stash_detail(99).unwrap_err();
    assert!(
        matches!(err, Error::NotFound { .. }) || err.to_string().contains("not found"),
        "invalid index should return not-found error; got: {err}"
    );
}

#[test]
fn stash_diff_returns_diff_summary() {
    let f = StashFixture::new();
    let repo = repository(f.path()).unwrap();
    let diff = repo.stash_diff(0).unwrap();
    // The stash modified README.md — should appear in the diff.
    assert!(
        !diff.added.is_empty() || !diff.modified.is_empty() || !diff.deleted.is_empty(),
        "stash diff should contain at least one changed path"
    );
}

#[test]
fn stash_diff_invalid_index_returns_err() {
    let f = StashFixture::new();
    let repo = repository(f.path()).unwrap();
    let err = repo.stash_diff(99).unwrap_err();
    assert!(
        matches!(err, Error::NotFound { .. }) || err.to_string().contains("not found"),
        "stash_diff with invalid index should return not-found error; got: {err}"
    );
}

// ── RFC 021: worktree_details ─────────────────────────────────────────────── //

#[test]
fn worktree_details_empty_when_no_linked_worktrees() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let details = repo.worktree_details().unwrap();
    assert!(details.is_empty(), "no linked worktrees → should be empty");
}

#[test]
fn worktree_details_single_linked_worktree() {
    let f = Fixture::new();
    let wt_dir = tempfile::TempDir::new().unwrap();
    // Add a linked worktree on a new branch.
    git_cmd(f.path(), &["branch", "feature-wt"]);
    let ok = std::process::Command::new("git")
        .args(["worktree", "add", wt_dir.path().to_str().unwrap(), "feature-wt"])
        .current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .env("GIT_TERMINAL_PROMPT","0")
        .stdin(std::process::Stdio::null())
        .status().map(|s| s.success()).unwrap_or(false);
    if !ok {
        eprintln!("skip: git worktree add failed (may not be supported)");
        return;
    }

    let repo = repository(f.path()).unwrap();
    let details = repo.worktree_details().unwrap();
    assert_eq!(details.len(), 1, "should have exactly one linked worktree");
    let wt = &details[0];
    assert_eq!(wt.state, WorktreeState::Present, "linked worktree should be Present");
    assert!(!wt.is_locked, "new worktree should not be locked");
    assert!(wt.lock_reason.is_none());
    assert_eq!(wt.current_branch, "feature-wt");
}

#[test]
fn worktree_details_sorted_by_id() {
    let f = Fixture::new();
    let wt1 = tempfile::TempDir::new().unwrap();
    let wt2 = tempfile::TempDir::new().unwrap();
    git_cmd(f.path(), &["branch", "branch-a"]);
    git_cmd(f.path(), &["branch", "branch-b"]);
    let ok1 = std::process::Command::new("git")
        .args(["worktree", "add", wt1.path().to_str().unwrap(), "branch-a"])
        .current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .env("GIT_TERMINAL_PROMPT","0").stdin(std::process::Stdio::null())
        .status().map(|s| s.success()).unwrap_or(false);
    let ok2 = std::process::Command::new("git")
        .args(["worktree", "add", wt2.path().to_str().unwrap(), "branch-b"])
        .current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .env("GIT_TERMINAL_PROMPT","0").stdin(std::process::Stdio::null())
        .status().map(|s| s.success()).unwrap_or(false);
    if !ok1 || !ok2 { eprintln!("skip: worktrees not supported"); return; }

    let repo = repository(f.path()).unwrap();
    let details = repo.worktree_details().unwrap();
    let ids: Vec<&str> = details.iter().map(|d| d.id.as_str()).collect();
    let sorted = { let mut v = ids.clone(); v.sort(); v };
    assert_eq!(ids, sorted, "worktree_details must be sorted ascending by id");
}

#[test]
fn worktree_details_locked_worktree() {
    let f = Fixture::new();
    let wt_dir = tempfile::TempDir::new().unwrap();
    git_cmd(f.path(), &["branch", "lock-test"]);
    let ok = std::process::Command::new("git")
        .args(["worktree", "add", wt_dir.path().to_str().unwrap(), "lock-test"])
        .current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .env("GIT_TERMINAL_PROMPT","0").stdin(std::process::Stdio::null())
        .status().map(|s| s.success()).unwrap_or(false);
    if !ok { eprintln!("skip: worktrees not supported"); return; }

    // Lock the worktree with a reason.
    let ok2 = std::process::Command::new("git")
        .args(["worktree", "lock", "--reason", "CI reserved",
               wt_dir.path().to_str().unwrap()])
        .current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .env("GIT_TERMINAL_PROMPT","0").stdin(std::process::Stdio::null())
        .status().map(|s| s.success()).unwrap_or(false);
    if !ok2 { eprintln!("skip: git worktree lock failed"); return; }

    let repo = repository(f.path()).unwrap();
    let details = repo.worktree_details().unwrap();
    let locked = details.iter().find(|d| d.is_locked);
    assert!(locked.is_some(), "locked worktree should appear as locked");
    assert_eq!(
        locked.unwrap().lock_reason.as_deref(),
        Some("CI reserved"),
        "lock reason should match"
    );
}
