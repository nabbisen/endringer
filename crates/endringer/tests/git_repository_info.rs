//! Integration tests for repository information and capability discovery
//! (RFC 009).

#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;
use endringer::repository::repository;
use endringer::{BackendKind, HeadState, ObjectFormat};

// ── normal Git repository ─────────────────────────────────────────────────── //

#[test]
fn repository_info_normal_git() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let info = repo.repository_info().unwrap();

    assert_eq!(info.backend, BackendKind::Git);
    assert!(!info.is_bare, "fixture is not bare");
    assert!(info.workdir.is_some(), "should have a working directory");
    assert_eq!(info.object_format, ObjectFormat::Sha1);
    // HEAD is attached on main.
    match &info.head {
        HeadState::Attached { branch, .. } => assert_eq!(branch, "main"),
        other => panic!("expected Attached, got {other:?}"),
    }
    // Git capabilities.
    assert!(info.capabilities.working_tree);
    assert!(info.capabilities.tag_create_lightweight);
    assert!(info.capabilities.tag_create_annotated);
    assert!(info.capabilities.tag_delete);
    assert!(info.capabilities.branch_tracking);
    assert!(!info.capabilities.operation_state); // RFC 008 not yet
    assert!(!info.capabilities.jj_native_state);
}

// ── bare repository ───────────────────────────────────────────────────────── //

#[test]
fn repository_info_bare() {
    let f = Fixture::new();
    let bare_dir = tempfile::TempDir::new().unwrap();
    std::process::Command::new("git")
        .args(["clone", "--bare", f.path().to_str().unwrap(),
               bare_dir.path().to_str().unwrap()])
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_EDITOR", "true")
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(std::process::Stdio::null())
        .status().unwrap();

    let repo = repository(bare_dir.path()).unwrap();
    let info = repo.repository_info().unwrap();

    assert!(info.is_bare, "bare clone must be bare");
    assert!(info.workdir.is_none(), "bare repo has no working dir");
    assert!(!info.capabilities.working_tree);
}

// ── detached HEAD ─────────────────────────────────────────────────────────── //

#[test]
fn repository_info_detached_head() {
    let f = Fixture::new();
    // Checkout the initial commit by hash to detach HEAD.
    let initial_hex = {
        let out = std::process::Command::new("git")
            .args(["rev-parse", "HEAD^"]).current_dir(f.path())
            .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
            .output().unwrap();
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    };
    f.git(&["checkout", "--detach", &initial_hex]);

    let repo = repository(f.path()).unwrap();
    let info = repo.repository_info().unwrap();

    match &info.head {
        HeadState::Detached { commit_id } => {
            assert_eq!(commit_id.to_string(), initial_hex,
                "detached commit_id must match checked-out hash");
        }
        other => panic!("expected Detached, got {other:?}"),
    }
}

// ── unborn / empty repository ─────────────────────────────────────────────── //

#[test]
fn repository_info_unborn_head() {
    let dir = tempfile::TempDir::new().unwrap();
    std::process::Command::new("git")
        .args(["init", "-b", "main", dir.path().to_str().unwrap()])
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .status().unwrap();

    let repo = repository(dir.path()).unwrap();
    let info = repo.repository_info().unwrap();

    match &info.head {
        HeadState::Unborn { branch } => {
            // gix may or may not expose the branch name for an unborn head,
            // but should not panic.
            let _ = branch;
        }
        // Some gix versions return Missing for a completely empty repo
        HeadState::Missing => {}
        other => panic!("expected Unborn or Missing for empty repo, got {other:?}"),
    }
}

// ── repo_name is the directory name ──────────────────────────────────────── //

#[test]
fn repository_info_repo_name() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let info = repo.repository_info().unwrap();
    // The temp dir has a random name; we just verify it's non-empty.
    assert!(!info.repo_name.is_empty(), "repo_name should be non-empty");
}

// ── async parity ─────────────────────────────────────────────────────────── //

#[test]
fn repository_info_vcs_dir_exists() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let info = repo.repository_info().unwrap();
    assert!(info.vcs_dir.exists(),
        "vcs_dir {:?} should exist", info.vcs_dir);
    assert!(info.vcs_dir.file_name().map_or(false, |n| n == ".git"),
        "vcs_dir should be .git for a standard repo");
}
