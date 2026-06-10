//! Integration tests for `ahead_behind` and `branch_ahead_behind`.
//!
//! RFC 004 §7.1 — git graph tests
//! RFC 004 §7.2 — branch upstream tests

#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;
use endringer::repository::repository;

// ── Helpers ──────────────────────────────────────────────────────────────── //

/// Returns the output of `git rev-list --left-right --count A...B`.
/// Used as the ground-truth oracle for ahead/behind counts.
fn git_left_right_count(f: &Fixture, local: &str, upstream: &str) -> (usize, usize) {
    let out = std::process::Command::new("git")
        .args(["rev-list", "--left-right", "--count",
               &format!("{local}...{upstream}")])
        .current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .expect("git");
    let s = String::from_utf8(out.stdout).unwrap();
    let parts: Vec<&str> = s.trim().split_whitespace().collect();
    (parts[0].parse().unwrap(), parts[1].parse().unwrap())
}

fn rev_parse(f: &Fixture, rev: &str) -> String {
    let out = std::process::Command::new("git")
        .args(["rev-parse", rev])
        .current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .expect("git");
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

// ── Case 1: identical tips ────────────────────────────────────────────────── //

#[test]
fn ahead_behind_identical_tips() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();

    let head_hex = rev_parse(&f, "HEAD");
    let id = endringer::CommitId::from_hex(&head_hex).unwrap();

    let ab = repo.ahead_behind(&id, &id).unwrap();
    assert_eq!(ab.ahead, 0, "identical: ahead should be 0");
    assert_eq!(ab.behind, 0, "identical: behind should be 0");
    assert_eq!(ab.merge_base.as_ref(), Some(&id), "identical: merge_base = self");
}

// ── Case 2: local one commit ahead ────────────────────────────────────────── //

#[test]
fn ahead_behind_local_one_ahead() {
    let f = Fixture::new();
    // HEAD is one commit ahead of the initial commit (tagged v0.1.0).
    let repo = repository(f.path()).unwrap();

    let head_hex = rev_parse(&f, "HEAD");
    let tag_hex  = rev_parse(&f, "v0.1.0");
    let head = endringer::CommitId::from_hex(&head_hex).unwrap();
    let tag  = endringer::CommitId::from_hex(&tag_hex).unwrap();

    let ab = repo.ahead_behind(&head, &tag).unwrap();
    let (expected_ahead, expected_behind) = git_left_right_count(&f, "HEAD", "v0.1.0");

    assert_eq!(ab.ahead,  expected_ahead,  "ahead mismatch");
    assert_eq!(ab.behind, expected_behind, "behind mismatch");
    assert_eq!(ab.merge_base.as_ref(), Some(&tag), "merge_base should be the tag commit");
}

// ── Case 3: local two commits behind ─────────────────────────────────────── //

#[test]
fn ahead_behind_local_two_behind() {
    let f = Fixture::new();
    // Add two more commits to "upstream", keep local at HEAD^.
    std::fs::write(f.path.join("c.rs"), "// c\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "commit C"]);
    std::fs::write(f.path.join("d.rs"), "// d\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "commit D"]);

    let repo = repository(f.path()).unwrap();
    let local_hex    = rev_parse(&f, "HEAD~2"); // two behind
    let upstream_hex = rev_parse(&f, "HEAD");
    let local    = endringer::CommitId::from_hex(&local_hex).unwrap();
    let upstream = endringer::CommitId::from_hex(&upstream_hex).unwrap();

    let ab = repo.ahead_behind(&local, &upstream).unwrap();
    let (exp_a, exp_b) = git_left_right_count(&f, &local_hex, "HEAD");

    assert_eq!(ab.ahead,  exp_a, "ahead mismatch");
    assert_eq!(ab.behind, exp_b, "behind should be 2");
    assert_eq!(ab.behind, 2);
}

// ── Case 4: both diverged ────────────────────────────────────────────────── //

#[test]
fn ahead_behind_both_diverged() {
    // Create:
    //   A (initial)
    //   ├── B (main: "add feature") ← local
    //   └── C (branch-x)           ← upstream
    let f = Fixture::new();
    let base_hex = rev_parse(&f, "HEAD^"); // A

    f.git(&["checkout", "-b", "branch-x", &base_hex]);
    std::fs::write(f.path.join("x.rs"), "// x\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "commit X"]);
    let upstream_hex = rev_parse(&f, "HEAD");

    f.git(&["checkout", "main"]);
    let local_hex = rev_parse(&f, "HEAD");

    let repo = repository(f.path()).unwrap();
    let local    = endringer::CommitId::from_hex(&local_hex).unwrap();
    let upstream = endringer::CommitId::from_hex(&upstream_hex).unwrap();
    let base     = endringer::CommitId::from_hex(&base_hex).unwrap();

    let ab = repo.ahead_behind(&local, &upstream).unwrap();
    let (exp_a, exp_b) = git_left_right_count(&f, "main", "branch-x");

    assert_eq!(ab.ahead,  exp_a, "ahead mismatch (diverged)");
    assert_eq!(ab.behind, exp_b, "behind mismatch (diverged)");
    assert!(ab.ahead  > 0, "both sides should have commits");
    assert!(ab.behind > 0, "both sides should have commits");
    // Merge base should be the fork point A.
    assert_eq!(ab.merge_base.as_ref(), Some(&base));
}

// ── Case 5: unrelated histories ──────────────────────────────────────────── //

#[test]
fn ahead_behind_unrelated_histories() {
    let f = Fixture::new();
    // Create an orphan branch — no shared ancestor.
    f.git(&["checkout", "--orphan", "orphan"]);
    std::fs::write(f.path.join("orphan.rs"), "// orphan\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "orphan commit"]);
    let orphan_hex = rev_parse(&f, "HEAD");

    f.git(&["checkout", "main"]);
    let main_hex = rev_parse(&f, "HEAD");

    let repo = repository(f.path()).unwrap();
    let main_id   = endringer::CommitId::from_hex(&main_hex).unwrap();
    let orphan_id = endringer::CommitId::from_hex(&orphan_hex).unwrap();

    let ab = repo.ahead_behind(&main_id, &orphan_id).unwrap();
    assert_eq!(ab.merge_base, None, "unrelated: no merge base");
    // Both sides have all their own commits.
    assert!(ab.ahead  > 0, "main should have commits");
    assert!(ab.behind > 0, "orphan should have commits");
}

// ── Case 6: merge commit in history ──────────────────────────────────────── //

#[test]
fn ahead_behind_with_merge_commit() {
    //   A ──── B (main)
    //    \    / (merge)
    //     C  (branch-m) merged into main → D
    let f = Fixture::new();
    let a_hex = rev_parse(&f, "HEAD^");

    f.git(&["checkout", "-b", "branch-m", &a_hex]);
    std::fs::write(f.path.join("m.rs"), "// m\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "branch-m commit"]);

    f.git(&["checkout", "main"]);
    f.git(&["merge", "--no-ff", "branch-m", "-m", "merge branch-m"]);

    let local_hex    = rev_parse(&f, "HEAD");   // D (merge commit)
    let upstream_hex = rev_parse(&f, "HEAD^2"); // C (branch tip)

    let repo = repository(f.path()).unwrap();
    let local    = endringer::CommitId::from_hex(&local_hex).unwrap();
    let upstream = endringer::CommitId::from_hex(&upstream_hex).unwrap();

    let ab = repo.ahead_behind(&local, &upstream).unwrap();
    let (exp_a, exp_b) = git_left_right_count(&f, "HEAD", "HEAD^2");

    assert_eq!(ab.ahead,  exp_a, "ahead mismatch (merge commit)");
    assert_eq!(ab.behind, exp_b, "behind mismatch (merge commit)");
}

// ── Case 7: missing commit ID ─────────────────────────────────────────────── //

#[test]
fn ahead_behind_missing_commit_returns_error() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let real_hex = rev_parse(&f, "HEAD");
    let real = endringer::CommitId::from_hex(&real_hex).unwrap();
    let fake = endringer::CommitId::from_hex(
        "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef").unwrap();

    assert!(repo.ahead_behind(&real, &fake).is_err(), "missing upstream should error");
    assert!(repo.ahead_behind(&fake, &real).is_err(), "missing local should error");
}

// ── RFC 004 §7.2: branch upstream tests ──────────────────────────────────── //

#[test]
fn branch_ahead_behind_no_upstream() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    // "main" has no upstream configured in this fixture.
    let result = repo.branch_ahead_behind("main").unwrap();
    assert_eq!(result, None, "no upstream configured → Ok(None)");
}

#[test]
fn branch_ahead_behind_with_upstream() {
    // Set up a "remote" by cloning the fixture bare, then configure tracking.
    let f = Fixture::new();

    let bare_dir = tempfile::TempDir::new().unwrap();
    f.git(&["clone", "--bare", f.path().to_str().unwrap(),
            bare_dir.path().to_str().unwrap()]);

    // Clone from the bare repo into a new worktree — this sets up tracking.
    let work_dir = tempfile::TempDir::new().unwrap();
    let ok = std::process::Command::new("git")
        .args(["clone", bare_dir.path().to_str().unwrap(),
               work_dir.path().to_str().unwrap()])
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_EDITOR", "true")
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(std::process::Stdio::null())
        .status().unwrap().success();
    assert!(ok, "clone failed");

    let repo = repository(work_dir.path()).unwrap();
    // "main" should track "origin/main" after a normal clone.
    let result = repo.branch_ahead_behind("main").unwrap();
    assert!(result.is_some(), "cloned repo should have upstream");
    let ab = result.unwrap();
    assert_eq!(ab.ahead,  0, "fresh clone: 0 ahead");
    assert_eq!(ab.behind, 0, "fresh clone: 0 behind");
}

#[test]
fn branch_ahead_behind_local_upstream_dot() {
    // branch.X.remote = "." means a local branch tracks another local branch.
    let f = Fixture::new();
    f.git(&["checkout", "-b", "feature"]);
    std::fs::write(f.path.join("feat.rs"), "// feat\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "feature commit"]);
    // Configure feature to track main locally.
    f.git(&["config", "branch.feature.remote", "."]);
    f.git(&["config", "branch.feature.merge", "refs/heads/main"]);

    let repo = repository(f.path()).unwrap();
    let ab = repo.branch_ahead_behind("feature").unwrap();
    assert!(ab.is_some(), "local upstream should resolve");
    let ab = ab.unwrap();
    assert_eq!(ab.ahead,  1, "feature is 1 ahead of main");
    assert_eq!(ab.behind, 0, "feature is not behind main");
}
