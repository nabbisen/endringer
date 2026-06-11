//! Git CLI parity test harness (RFC 015).
//!
//! These helpers run real `git` plumbing/porcelain commands with machine-
//! readable output formats and return structured results. They exist only
//! for tests — endringer never calls `git` at runtime.
//!
//! All commands use the same environment isolation as the fixture helpers:
//! no system config, no global config, no editor, no prompts.

use std::path::Path;
use std::process::Command;

/// Runs a git command in `repo` and returns trimmed stdout.
/// Panics if the command fails.
pub fn git_output(repo: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .env("GIT_CONFIG_NOSYSTEM","1")
        .env("GIT_CONFIG_GLOBAL","/dev/null")
        .env("GIT_EDITOR","true")
        .env("GIT_TERMINAL_PROMPT","0")
        .stdin(std::process::Stdio::null())
        .output()
        .expect("git");
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

/// Runs a git command and returns non-empty trimmed output lines.
pub fn git_lines(repo: &Path, args: &[&str]) -> Vec<String> {
    git_output(repo, args)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Returns (ahead, behind) counts via `git rev-list --left-right --count`.
pub fn git_ahead_behind(repo: &Path, local: &str, upstream: &str) -> (usize, usize) {
    let spec = format!("{}...{}", local, upstream);
    let line = git_output(repo, &["rev-list","--left-right","--count",&spec]);
    let mut parts = line.split_whitespace();
    let ahead: usize = parts.next().unwrap_or("0").parse().unwrap_or(0);
    let behind: usize = parts.next().unwrap_or("0").parse().unwrap_or(0);
    (ahead, behind)
}

/// Returns the merge-base SHA between two refs.
pub fn git_merge_base(repo: &Path, a: &str, b: &str) -> Option<String> {
    let out = Command::new("git")
        .args(["merge-base", a, b])
        .current_dir(repo)
        .env("GIT_CONFIG_NOSYSTEM","1")
        .env("GIT_CONFIG_GLOBAL","/dev/null")
        .env("GIT_TERMINAL_PROMPT","0")
        .stdin(std::process::Stdio::null())
        .output()
        .ok()?;
    if out.status.success() {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        if s.is_empty() { None } else { Some(s) }
    } else {
        None // unrelated histories
    }
}

/// Returns true when `git merge-base --is-ancestor a b` exits 0.
pub fn git_is_ancestor(repo: &Path, a: &str, b: &str) -> bool {
    Command::new("git")
        .args(["merge-base","--is-ancestor", a, b])
        .current_dir(repo)
        .env("GIT_CONFIG_NOSYSTEM","1")
        .env("GIT_CONFIG_GLOBAL","/dev/null")
        .env("GIT_TERMINAL_PROMPT","0")
        .stdin(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Returns tag names via `git for-each-ref --format=%(refname:short) refs/tags`.
pub fn git_tag_names(repo: &Path) -> Vec<String> {
    git_lines(repo, &["for-each-ref","--format=%(refname:short)","refs/tags"])
}

/// Returns local branch names via `git for-each-ref refs/heads`.
pub fn git_branch_names(repo: &Path) -> Vec<String> {
    git_lines(repo, &["for-each-ref","--format=%(refname:short)","refs/heads"])
}

/// Returns porcelain status lines (v1 format XY path).
pub fn git_status_short(repo: &Path) -> Vec<String> {
    git_lines(repo, &["status","--porcelain","--no-renames","-z"])
        .into_iter()
        .filter(|l| l.len() >= 3)
        .collect()
}
