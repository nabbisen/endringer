//! Shared fixture-repository helper for integration tests.
//!
//! Include in any test file with:
//! ```rust
//! #[path = "support/fixture.rs"]
//! mod fixture;
//! use fixture::Fixture;
//! ```
//!
//! Files in `tests/support/` are **not** auto-discovered as test binaries by
//! Cargo — only `.rs` files directly in `tests/` are. This file is only
//! compiled when explicitly referenced via `#[path]`.

use std::path::{Path, PathBuf};
use std::process::Command;

/// A temporary git repository with a deterministic commit history.
///
/// ```text
/// commit A  "initial commit"   — adds README.md   (tagged v0.1.0)
/// commit B  "add feature"      — adds src.rs       ← HEAD / main
/// ```
pub struct Fixture {
    /// Keeps the temp directory alive for the lifetime of the fixture.
    pub _dir: tempfile::TempDir,
    pub path: PathBuf,
}

impl Fixture {
    /// Creates the standard two-commit fixture.
    pub fn new() -> Self {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().to_path_buf();
        let f = Fixture { _dir: dir, path };
        f.git(&["init", "-b", "main"]);
        f.git(&["config", "user.email", "fixture@test.local"]);
        f.git(&["config", "user.name", "Fixture"]);
        std::fs::write(f.path.join("README.md"), "# fixture\n").unwrap();
        f.git(&["add", "."]);
        f.git(&["commit", "-m", "initial commit"]);
        f.git(&["tag", "v0.1.0"]);
        std::fs::write(f.path.join("src.rs"), "fn main() {}\n").unwrap();
        f.git(&["add", "."]);
        f.git(&["commit", "-m", "add feature"]);
        f
    }

    /// Returns the path to the fixture repository.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Runs a git command inside the fixture directory.
    pub fn git(&self, args: &[&str]) {
        let ok = Command::new("git")
            .args(args)
            .current_dir(&self.path)
            .status()
            .expect("git")
            .success();
        assert!(ok, "git {} failed", args.join(" "));
    }
}
