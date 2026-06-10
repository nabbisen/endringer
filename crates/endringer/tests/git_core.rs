//! Constructor and status tests.
#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;

use endringer::repository::{jj_repository, repository};

#[test]
fn opens_valid_repo() {
    let f = Fixture::new();
    assert!(repository(f.path()).is_ok());
}

#[test]
fn rejects_missing_path() {
    assert!(repository(std::path::Path::new("/no/such/repo")).is_err());
}

#[test]
fn backend_kind_is_git() {
    use endringer::BackendKind;
    let f = Fixture::new();
    assert_eq!(repository(f.path()).unwrap().backend_kind(), BackendKind::Git);
}

#[test]
fn status_digest_fields() {
    let f = Fixture::new();
    let d = repository(f.path()).unwrap().status_digest().unwrap();
    assert_eq!(d.current_branch, "main");
    assert_eq!(d.last_commit_summary, "add feature");
    assert_eq!(d.last_commit_id.short().len(), 7);
}

#[test]
fn remote_url_none_when_no_remote() {
    let f = Fixture::new();
    assert!(repository(f.path()).unwrap().remote_url("origin").unwrap().is_none());
}

#[test]
fn jj_repository_rejects_plain_git() {
    let f = Fixture::new();
    assert!(jj_repository(f.path()).is_err());
}

// ── RFC 023: SHA-256 object format tests ─────────────────────────────────── //

/// Returns true if the system git supports --object-format=sha256.
fn git_supports_sha256() -> bool {
    std::process::Command::new("git")
        .args(["init", "--object-format=sha256", "/dev/null"])
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn commit_id_from_hex_rejects_wrong_lengths() {
    use endringer::CommitId;
    // 39 chars (too short for SHA-1)
    assert!(CommitId::from_hex("a".repeat(39).as_str()).is_err());
    // 41 chars (too long for SHA-1, too short for SHA-256)
    assert!(CommitId::from_hex("a".repeat(41).as_str()).is_err());
    // 63 chars (too short for SHA-256)
    assert!(CommitId::from_hex("a".repeat(63).as_str()).is_err());
    // 65 chars (too long for SHA-256)
    assert!(CommitId::from_hex("a".repeat(65).as_str()).is_err());
}

#[test]
fn commit_id_from_hex_accepts_sha1_and_sha256() {
    use endringer::CommitId;
    let sha1 = CommitId::from_hex(&"a".repeat(40)).unwrap();
    let sha256 = CommitId::from_hex(&"b".repeat(64)).unwrap();
    // They are never equal regardless of byte content.
    assert_ne!(sha1, sha256, "SHA-1 and SHA-256 CommitIds are never equal");
    assert_eq!(sha1.to_string().len(), 40);
    assert_eq!(sha256.to_string().len(), 64);
}

#[test]
fn object_format_sha256_repo_opens() {
    use endringer::{repository, ObjectFormat};

    if !git_supports_sha256() {
        eprintln!("skip: git --object-format=sha256 not supported by installed git");
        return;
    }

    let dir = tempfile::TempDir::new().unwrap();
    let ok = std::process::Command::new("git")
        .args(["init", "--object-format=sha256", "-b", "main"])
        .current_dir(dir.path())
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .status().map(|s| s.success()).unwrap_or(false);
    assert!(ok, "git init --object-format=sha256 failed");

    // Configure identity and make an initial commit.
    for cfg in [["config","user.email","test@local"],["config","user.name","Test"]] {
        std::process::Command::new("git").args(cfg).current_dir(dir.path())
            .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
            .status().unwrap();
    }
    std::fs::write(dir.path().join("f.txt"), "hi").unwrap();
    std::process::Command::new("git").args(["add","."]).current_dir(dir.path())
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .status().unwrap();
    std::process::Command::new("git").args(["commit","-m","init"]).current_dir(dir.path())
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .env("GIT_EDITOR","true").env("GIT_TERMINAL_PROMPT","0")
        .stdin(std::process::Stdio::null()).status().unwrap();

    let repo = repository(dir.path()).expect("SHA-256 repo should open");
    let info = repo.repository_info().unwrap();
    assert_eq!(info.object_format, ObjectFormat::Sha256,
        "SHA-256 repo should report Sha256 object format");

    // HEAD commit ID should be 64 hex chars.
    let digest = repo.status_digest().unwrap();
    assert_eq!(digest.last_commit_id.to_string().len(), 64,
        "SHA-256 commit ID should be 64 hex chars");
}
