//! Integration tests for branch tracking and sync state (RFC 005).

#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;
use endringer::repository::repository;

fn clone_fixture(f: &Fixture) -> tempfile::TempDir {
    let work_dir = tempfile::TempDir::new().unwrap();
    let ok = std::process::Command::new("git")
        .args(["clone", f.path().to_str().unwrap(),
               work_dir.path().to_str().unwrap()])
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_EDITOR", "true")
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(std::process::Stdio::null())
        .status().unwrap().success();
    assert!(ok, "clone failed");
    work_dir
}

// ── branch without upstream ───────────────────────────────────────────────── //

#[test]
fn branch_tracking_no_upstream() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let info = repo.branch_tracking("main").unwrap();

    assert_eq!(info.branch, "main");
    assert_eq!(info.full_name, "refs/heads/main");
    assert!(info.upstream.is_none(), "no upstream configured");
    assert!(!info.upstream_gone);
    assert!(info.ahead_behind.is_none());
}

// ── branch with origin upstream (fresh clone = 0 ahead, 0 behind) ────────── //

#[test]
fn branch_tracking_with_upstream_clean() {
    let f = Fixture::new();
    let cloned = clone_fixture(&f);
    let repo = repository(cloned.path()).unwrap();
    let info = repo.branch_tracking("main").unwrap();

    assert_eq!(info.upstream.as_deref(), Some("refs/remotes/origin/main"));
    assert!(!info.upstream_gone);
    let ab = info.ahead_behind.expect("ahead_behind should be computed");
    assert_eq!(ab.ahead,  0, "fresh clone: ahead=0");
    assert_eq!(ab.behind, 0, "fresh clone: behind=0");
}

// ── branch one commit ahead of upstream ──────────────────────────────────── //

#[test]
fn branch_tracking_ahead() {
    let f = Fixture::new();
    let cloned = clone_fixture(&f);

    // Add a local commit in the clone.
    std::fs::write(cloned.path().join("extra.rs"), "// extra\n").unwrap();
    let git = |args: &[&str]| {
        let ok = std::process::Command::new("git").args(args).current_dir(cloned.path())
            .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
            .env("GIT_EDITOR","true").env("GIT_TERMINAL_PROMPT","0")
            .stdin(std::process::Stdio::null()).status().unwrap().success();
        assert!(ok, "git {} failed", args.join(" "));
    };
    git(&["config", "user.email", "fixture@test.local"]);
    git(&["config", "user.name",  "Fixture"]);
    git(&["add", "."]);
    git(&["commit", "-m", "local extra commit"]);

    let repo = repository(cloned.path()).unwrap();
    let info = repo.branch_tracking("main").unwrap();

    let ab = info.ahead_behind.expect("ahead_behind should be computed");
    assert_eq!(ab.ahead,  1, "one commit ahead");
    assert_eq!(ab.behind, 0, "not behind");
}

// ── upstream gone ─────────────────────────────────────────────────────────── //

#[test]
fn branch_tracking_upstream_gone() {
    let f = Fixture::new();
    let cloned = clone_fixture(&f);

    // Manually configure an upstream that doesn't exist.
    let git = |args: &[&str]| {
        std::process::Command::new("git").args(args).current_dir(cloned.path())
            .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
            .env("GIT_TERMINAL_PROMPT","0").status().unwrap();
    };
    git(&["config", "branch.main.remote", "origin"]);
    git(&["config", "branch.main.merge",  "refs/heads/deleted-branch"]);

    let repo = repository(cloned.path()).unwrap();
    let info = repo.branch_tracking("main").unwrap();

    assert!(info.upstream.is_some(), "upstream key present");
    assert!(info.upstream_gone, "upstream ref should be gone");
    assert!(info.ahead_behind.is_none(), "no ahead_behind when upstream is gone");
}

// ── local upstream (remote = ".") ────────────────────────────────────────── //

#[test]
fn branch_tracking_local_upstream() {
    let f = Fixture::new();
    f.git(&["checkout", "-b", "feature"]);
    std::fs::write(f.path.join("feat.rs"), "// feat\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "feature commit"]);
    f.git(&["config", "branch.feature.remote", "."]);
    f.git(&["config", "branch.feature.merge", "refs/heads/main"]);

    let repo = repository(f.path()).unwrap();
    let info = repo.branch_tracking("feature").unwrap();

    assert_eq!(info.upstream.as_deref(), Some("refs/heads/main"));
    assert!(!info.upstream_gone);
    let ab = info.ahead_behind.unwrap();
    assert_eq!(ab.ahead,  1);
    assert_eq!(ab.behind, 0);
}

// ── local_branch_tracking is sorted ascending by full_name ───────────────── //

#[test]
fn local_branch_tracking_sorted() {
    let f = Fixture::new();
    // Create branches in non-alphabetical creation order.
    f.git(&["checkout", "-b", "zzz-last"]);
    f.git(&["checkout", "main"]);
    f.git(&["checkout", "-b", "aaa-first"]);
    f.git(&["checkout", "main"]);

    let repo = repository(f.path()).unwrap();
    let list = repo.local_branch_tracking().unwrap();
    let names: Vec<&str> = list.iter().map(|b| b.full_name.as_str()).collect();

    // Must be ascending.
    let sorted = {
        let mut v = names.clone();
        v.sort();
        v
    };
    assert_eq!(names, sorted, "local_branch_tracking must be sorted by full_name");
    // All three branches should appear.
    assert!(list.iter().any(|b| b.branch == "aaa-first"));
    assert!(list.iter().any(|b| b.branch == "main"));
    assert!(list.iter().any(|b| b.branch == "zzz-last"));
}

// ── local_branches is also sorted (RFC 005 branch-listing sort contract) ──── //

#[test]
fn local_branches_sorted() {
    let f = Fixture::new();
    f.git(&["checkout", "-b", "zzz-branch"]);
    f.git(&["checkout", "main"]);
    f.git(&["checkout", "-b", "aaa-branch"]);
    f.git(&["checkout", "main"]);

    let repo = repository(f.path()).unwrap();
    let branches = repo.local_branches().unwrap();
    let full_names: Vec<&str> = branches.iter().map(|b| b.full_name.as_str()).collect();
    let sorted = {
        let mut v = full_names.clone();
        v.sort();
        v
    };
    assert_eq!(full_names, sorted, "local_branches must be sorted by full_name");
}

// ── is_merged_into ────────────────────────────────────────────────────────── //

#[test]
fn is_merged_into_positive() {
    let f = Fixture::new();
    // initial commit is the base; main has 2 commits.
    // Create feature branched from the initial commit and merge it in.
    let base_hex = {
        let out = std::process::Command::new("git")
            .args(["rev-parse", "HEAD^"]).current_dir(f.path())
            .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
            .output().unwrap();
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    };
    f.git(&["checkout", "-b", "merged-feature", &base_hex]);
    std::fs::write(f.path.join("mf.rs"), "// mf\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "merged feature commit"]);
    f.git(&["checkout", "main"]);
    f.git(&["merge", "--no-ff", "merged-feature", "-m", "merge merged-feature"]);

    let repo = repository(f.path()).unwrap();
    assert!(repo.is_merged_into("merged-feature", "main").unwrap(),
            "merged-feature should be merged into main");
}

#[test]
fn is_merged_into_negative() {
    let f = Fixture::new();
    // Create a branch that has NOT been merged.
    f.git(&["checkout", "-b", "unmerged"]);
    std::fs::write(f.path.join("um.rs"), "// um\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "unmerged commit"]);

    let repo = repository(f.path()).unwrap();
    // Restore HEAD to main before calling (branch still exists).
    f.git(&["checkout", "main"]);
    drop(repo); // re-open after branch switch
    let repo = repository(f.path()).unwrap();
    assert!(!repo.is_merged_into("unmerged", "main").unwrap(),
            "unmerged should NOT be merged into main");
}
