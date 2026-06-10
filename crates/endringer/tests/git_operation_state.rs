//! Integration tests for operation state and conflict detection (RFC 008).
//!
//! Fixtures create repositories in various in-progress states by running
//! git commands that set the marker files checked by `operation_state()`.

#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;
use endringer::repository::repository;
use endringer::{OperationState, RebaseKind};

// ── Clean repository ──────────────────────────────────────────────────────── //

#[test]
fn clean_repo_operation_state_is_none() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    assert_eq!(repo.operation_state().unwrap(), OperationState::None);
}

#[test]
fn clean_repo_unmerged_paths_is_empty() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    assert!(repo.unmerged_paths().unwrap().is_empty());
}

#[test]
fn clean_repo_conflict_summary_is_empty() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let cs = repo.conflict_summary().unwrap();
    assert!(cs.is_empty());
    assert_eq!(cs.len(), 0);
}

// ── Merge conflict fixture ────────────────────────────────────────────────── //

/// Creates a fixture with a merge conflict on `conflict.txt`.
///
/// Branch `main` and `branch-b` both modify `conflict.txt` in different ways,
/// then the test merges `branch-b` into `main` which conflicts.
struct MergeConflictFixture {
    _f: Fixture,
}

impl MergeConflictFixture {
    fn new() -> Self {
        let f = Fixture::new();
        // Create conflict.txt on main
        std::fs::write(f.path.join("conflict.txt"), "main line\n").unwrap();
        f.git(&["add", "."]);
        f.git(&["commit", "-m", "add conflict.txt on main"]);

        // Create branch-b from parent of that commit, modify conflict.txt
        let head_hex = {
            let out = std::process::Command::new("git")
                .args(["rev-parse", "HEAD^"])
                .current_dir(f.path())
                .env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null")
                .output().unwrap();
            String::from_utf8(out.stdout).unwrap().trim().to_string()
        };
        f.git(&["checkout", "-b", "branch-b", &head_hex]);
        std::fs::write(f.path.join("conflict.txt"), "branch-b line\n").unwrap();
        f.git(&["add", "."]);
        f.git(&["commit", "-m", "add conflict.txt on branch-b"]);

        // Switch back to main and attempt merge — this will conflict
        f.git(&["checkout", "main"]);
        let _ = std::process::Command::new("git")
            .args(["merge", "--no-ff", "branch-b"])
            .current_dir(f.path())
            .env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_EDITOR", "true").env("GIT_TERMINAL_PROMPT", "0")
            .stdin(std::process::Stdio::null())
            .status(); // expect failure (conflict) — ignore exit code

        MergeConflictFixture { _f: f }
    }

    fn path(&self) -> &std::path::Path {
        self._f.path()
    }
}

#[test]
fn merge_conflict_operation_state_is_merge() {
    let f = MergeConflictFixture::new();
    // Verify MERGE_HEAD exists (fixture created the conflict).
    if !f.path().join(".git").join("MERGE_HEAD").exists() {
        eprintln!("skip: merge conflict fixture did not produce MERGE_HEAD (no conflict?)");
        return;
    }
    let repo = repository(f.path()).unwrap();
    match repo.operation_state().unwrap() {
        OperationState::Merge { heads } => {
            assert!(!heads.is_empty(), "MERGE_HEAD should contain at least one OID");
        }
        other => panic!("expected Merge, got {other:?}"),
    }
}

#[test]
fn merge_conflict_unmerged_paths_present() {
    let f = MergeConflictFixture::new();
    if !f.path().join(".git").join("MERGE_HEAD").exists() {
        eprintln!("skip: no MERGE_HEAD");
        return;
    }
    let repo = repository(f.path()).unwrap();
    let paths = repo.unmerged_paths().unwrap();
    assert!(!paths.is_empty(), "unmerged_paths should be non-empty during merge conflict");
    assert!(
        paths.iter().any(|p| p.to_str().unwrap_or("").contains("conflict")),
        "conflict.txt should appear in unmerged paths; got: {paths:?}"
    );
}

#[test]
fn merge_conflict_paths_are_sorted() {
    let f = MergeConflictFixture::new();
    if !f.path().join(".git").join("MERGE_HEAD").exists() {
        eprintln!("skip: no MERGE_HEAD");
        return;
    }
    let repo = repository(f.path()).unwrap();
    let paths = repo.unmerged_paths().unwrap();
    let sorted = {
        let mut v = paths.clone();
        v.sort();
        v
    };
    assert_eq!(paths, sorted, "unmerged_paths should be sorted ascending");
}

#[test]
fn merge_conflict_summary_has_stages() {
    let f = MergeConflictFixture::new();
    if !f.path().join(".git").join("MERGE_HEAD").exists() {
        eprintln!("skip: no MERGE_HEAD");
        return;
    }
    let repo = repository(f.path()).unwrap();
    let cs = repo.conflict_summary().unwrap();
    assert!(!cs.is_empty(), "conflict_summary should be non-empty");
    // Each conflicted path should have stages.
    for cp in &cs.paths {
        assert!(!cp.stages.is_empty(), "path {:?} should have stages", cp.path);
        // Stages are 1, 2, or 3.
        for s in &cp.stages {
            assert!(s.stage >= 1 && s.stage <= 3, "stage should be 1-3, got {}", s.stage);
        }
    }
}

// ── Cherry-pick conflict ───────────────────────────────────────────────────── //

#[test]
fn cherry_pick_conflict_operation_state() {
    let f = Fixture::new();
    // Create a file on branch-c
    f.git(&["checkout", "-b", "branch-c"]);
    std::fs::write(f.path.join("shared.txt"), "branch-c version\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "add shared.txt on branch-c"]);

    // Switch to main, create a conflicting file
    f.git(&["checkout", "main"]);
    std::fs::write(f.path.join("shared.txt"), "main version\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "add shared.txt on main"]);

    // Cherry-pick the branch-c commit onto main (will conflict on shared.txt)
    let cherry_hex = {
        let out = std::process::Command::new("git")
            .args(["rev-parse", "branch-c"])
            .current_dir(f.path())
            .env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null")
            .output().unwrap();
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    };
    let _ = std::process::Command::new("git")
        .args(["cherry-pick", &cherry_hex])
        .current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_EDITOR", "true").env("GIT_TERMINAL_PROMPT", "0")
        .stdin(std::process::Stdio::null())
        .status();

    if !f.path().join(".git").join("CHERRY_PICK_HEAD").exists() {
        eprintln!("skip: cherry-pick did not produce CHERRY_PICK_HEAD");
        return;
    }

    let repo = repository(f.path()).unwrap();
    assert!(
        matches!(repo.operation_state().unwrap(), OperationState::CherryPick { .. }),
        "expected CherryPick state"
    );
}

// ── Revert conflict ───────────────────────────────────────────────────────── //

#[test]
fn revert_conflict_operation_state() {
    let f = Fixture::new();
    // Modify a file so reverting the commit will conflict with a later change.
    std::fs::write(f.path.join("README.md"), "version B\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "version B"]);

    let revert_target = {
        let out = std::process::Command::new("git")
            .args(["rev-parse", "HEAD^"])
            .current_dir(f.path())
            .env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null")
            .output().unwrap();
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    };

    // Modify again so revert of HEAD^ conflicts
    std::fs::write(f.path.join("README.md"), "version C\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "version C"]);

    // Revert the HEAD^ commit — this should conflict with version C
    let _ = std::process::Command::new("git")
        .args(["revert", "--no-edit", &revert_target])
        .current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_EDITOR", "true").env("GIT_TERMINAL_PROMPT", "0")
        .stdin(std::process::Stdio::null())
        .status();

    if !f.path().join(".git").join("REVERT_HEAD").exists() {
        eprintln!("skip: revert did not produce REVERT_HEAD");
        return;
    }

    let repo = repository(f.path()).unwrap();
    assert!(
        matches!(repo.operation_state().unwrap(), OperationState::Revert { .. }),
        "expected Revert state"
    );
}

// ── Rebase (merge backend) ────────────────────────────────────────────────── //

#[test]
fn rebase_merge_operation_state() {
    let f = Fixture::new();
    let base = {
        let out = std::process::Command::new("git")
            .args(["rev-parse", "HEAD^"])
            .current_dir(f.path())
            .env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null")
            .output().unwrap();
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    };

    // Create a branch that will conflict on rebase
    f.git(&["checkout", "-b", "rebase-branch", &base]);
    std::fs::write(f.path.join("src.rs"), "// rebase branch version\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "rebase branch commit"]);

    // Rebase onto main — src.rs was added by main too, should conflict
    let _ = std::process::Command::new("git")
        .args(["rebase", "--merge", "main"])
        .current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_EDITOR", "true").env("GIT_TERMINAL_PROMPT", "0")
        .stdin(std::process::Stdio::null())
        .status();

    if !f.path().join(".git").join("rebase-merge").is_dir() {
        eprintln!("skip: rebase-merge directory not produced (no conflict or different backend)");
        return;
    }

    let repo = repository(f.path()).unwrap();
    assert!(
        matches!(
            repo.operation_state().unwrap(),
            OperationState::Rebase { kind: RebaseKind::Merge }
        ),
        "expected Rebase(Merge) state"
    );
}

// ── Async parity ──────────────────────────────────────────────────────────── //
// (In async_tests.rs rather than here to keep the tokio runtime in one place)
