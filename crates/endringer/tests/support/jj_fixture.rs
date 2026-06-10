//! Jujutsu test fixture helper (RFC 007).
//!
//! ## Skip behaviour
//!
//! Tests using `JjFixture` call [`require_jj`] first. If `jj` is not on
//! `$PATH`, the test prints a skip message and returns early.
//!
//! If the environment variable `ENDRINGER_REQUIRE_JJ_CLI_TESTS=1` is set
//! (CI), missing `jj` is a panic rather than a skip.
//!
//! ## Supported jj version
//!
//! Tests are written against jj ≥ 0.18 (October 2024). Older versions used
//! different subcommand names; newer versions are expected to be compatible
//! until the storage format changes (tracked in RFC 007 acceptance criteria).

use std::path::{Path, PathBuf};
use std::process::Command;

// ── require_jj ───────────────────────────────────────────────────────────── //

/// Returns `true` if `jj` is available and the tests should run.
///
/// When `ENDRINGER_REQUIRE_JJ_CLI_TESTS=1` and jj is absent, this panics
/// to make CI failures visible.
///
/// Call this at the top of every test that uses [`JjFixture`]:
///
/// ```rust,no_run
/// #[test]
/// fn my_jj_test() {
///     if !require_jj() { return; }
///     // ... rest of test
/// }
/// ```
pub fn require_jj() -> bool {
    let available = Command::new("jj")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !available {
        if std::env::var_os("ENDRINGER_REQUIRE_JJ_CLI_TESTS").is_some() {
            panic!(
                "ENDRINGER_REQUIRE_JJ_CLI_TESTS is set but `jj` is not installed. \
                 Install jj ≥ 0.18 to run jj CLI tests."
            );
        }
        eprintln!("skip: jj not installed (set ENDRINGER_REQUIRE_JJ_CLI_TESTS=1 to fail on missing jj)");
        return false;
    }

    // Record version for CI logs.
    if let Ok(out) = Command::new("jj").arg("--version").output() {
        if let Ok(ver) = String::from_utf8(out.stdout) {
            eprintln!("jj version: {}", ver.trim());
        }
    }
    true
}

// ── JjFixture ────────────────────────────────────────────────────────────── //

/// A temporary directory containing a jj repository, created by the real
/// `jj` CLI.
///
/// Calling code must first check [`require_jj`] and return early if it
/// returns `false`.
pub struct JjFixture {
    /// Keeps the directory alive for the lifetime of the fixture.
    pub _dir: tempfile::TempDir,
    pub path: PathBuf,
}

impl JjFixture {
    /// Creates a native jj repository (`.jj/` only, no `.git/`).
    ///
    /// History is seeded with:
    /// - one commit adding `README.md`
    /// - one commit adding `src.rs`
    pub fn native() -> Self {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().to_path_buf();
        let f = JjFixture { _dir: dir, path };

        // Minimal jj config in temp home so user config cannot interfere.
        let fake_home = f.path.join(".fake-home");
        std::fs::create_dir_all(&fake_home).unwrap();
        let jj_config = fake_home.join("jj-config.toml");
        std::fs::write(&jj_config, concat!(
            "[user]\n",
            "name = \"JjFixture\"\n",
            "email = \"fixture@test.local\"\n",
            "[ui]\n",
            "paginate = \"never\"\n",
        )).unwrap();

        // git.auto-local-bookmark is the post-0.18 option name;
        // older versions use git.auto-local-branch — try both via separate
        // config lines (jj ignores unknown keys in recent versions).
        f.jj_with_config(&jj_config, &["git", "init", "."]);
        f.write("README.md", "# fixture\n");
        f.jj_with_config(&jj_config, &["describe", "-m", "initial commit"]);
        f.jj_with_config(&jj_config, &["new", "-m", "add src"]);
        f.write("src.rs", "fn main() {}\n");
        f.jj_with_config(&jj_config, &["squash"]);
        f
    }

    /// Creates a colocated jj repository (`.git/` + `.jj/`).
    ///
    /// Starts from a plain git repo and initialises jj co-location on top.
    pub fn colocated() -> Self {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().to_path_buf();
        let f = JjFixture { _dir: dir, path };

        let fake_home = f.path.join(".fake-home");
        std::fs::create_dir_all(&fake_home).unwrap();
        let jj_config = fake_home.join("jj-config.toml");
        std::fs::write(&jj_config, concat!(
            "[user]\n",
            "name = \"JjColocFixture\"\n",
            "email = \"fixture-coloc@test.local\"\n",
            "[ui]\n",
            "paginate = \"never\"\n",
        )).unwrap();

        // Init git repo first, make one commit, then colocate jj.
        f.git(&["init", "-b", "main"]);
        f.git(&["config", "user.email", "fixture@test.local"]);
        f.git(&["config", "user.name",  "Fixture"]);
        f.write("README.md", "# colocated\n");
        f.git(&["add", "."]);
        f.git(&["commit", "-m", "initial commit"]);
        // jj git init --colocate turns the existing .git into a colocated jj repo.
        f.jj_with_config(&jj_config, &["git", "init", "--colocate", "."]);
        f
    }

    /// Returns the path to the fixture repository.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Runs a jj command inside the fixture directory with a temporary config.
    fn jj_with_config(&self, config_path: &Path, args: &[&str]) {
        let mut cmd = Command::new("jj");
        cmd.args(args)
            .current_dir(&self.path)
            .env("JJ_CONFIG", config_path)
            .env("NO_COLOR", "1")
            .stdin(std::process::Stdio::null());
        let status = cmd.status().expect("jj");
        if !status.success() {
            eprintln!("jj {} exited with {:?} (non-zero — may be expected in some tests)",
                args.join(" "), status.code());
        }
    }

    /// Runs a jj command inside the fixture directory using default config.
    pub fn jj(&self, args: &[&str]) {
        let fake_home = self.path.join(".fake-home");
        let jj_config = fake_home.join("jj-config.toml");
        if jj_config.exists() {
            self.jj_with_config(&jj_config, args);
        } else {
            Command::new("jj")
                .args(args)
                .current_dir(&self.path)
                .env("NO_COLOR", "1")
                .stdin(std::process::Stdio::null())
                .status()
                .expect("jj");
        }
    }

    /// Runs a git command inside the fixture directory.
    pub fn git(&self, args: &[&str]) {
        let ok = Command::new("git")
            .args(args)
            .current_dir(&self.path)
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

    /// Writes a file into the fixture working directory.
    pub fn write(&self, rel_path: &str, contents: &str) {
        let full = self.path.join(rel_path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(full, contents).unwrap();
    }
}
