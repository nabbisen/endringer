//! Jujutsu (jj) backend — drives the `jj` CLI and parses its output.
//!
//! Jujutsu uses SHA-256 change IDs (64 hex characters) rather than Git's
//! SHA-1 commit hashes.  `CommitId` transparently supports both widths.
//!
//! # Availability
//!
//! Methods that require write access (`create_tag`, `delete_tag`, …) are
//! forwarded to the underlying Git repo that jj co-locates with its own
//! metadata, and will work as long as the repository has a Git backend.
//!
//! # Errors
//!
//! Every method returns an error if the `jj` binary is not on `$PATH`.

mod parse;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use anyhow::{Context, Result, bail};

use crate::{
    backend::VcsBackend,
    types::{
        BranchInfo, CommitId, CommitInfo, DiffSummary, SortOrder, StatusDigest,
        TagInfo,
    },
};

/// Jujutsu backend.  All operations delegate to `jj` CLI invocations.
pub(crate) struct JjBackend {
    /// Root of the repository (the directory that contains `.jj/`).
    root: PathBuf,
}

impl JjBackend {
    /// Opens a Jujutsu repository at `path`.
    ///
    /// Returns an error if `jj` is not on `$PATH` or if `path` does not
    /// contain a `.jj/` directory.
    pub(crate) fn open(path: &Path) -> Result<Self> {
        // Verify jj is reachable.
        jj_version()?;

        // Resolve the actual repo root (jj root prints it).
        let root = jj_root(path)?;

        Ok(JjBackend { root })
    }
}

// ── CLI helpers ──────────────────────────────────────────────────────────── //

/// Checks that `jj` is accessible and returns its version string.
fn jj_version() -> Result<String> {
    let out = Command::new("jj")
        .args(["version"])
        .output()
        .context("failed to run 'jj version' — is jj installed and on $PATH?")?;

    if !out.status.success() {
        bail!("'jj version' exited with status {}", out.status);
    }

    Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

/// Returns the repository root by running `jj root`.
fn jj_root(path: &Path) -> Result<PathBuf> {
    let out = Command::new("jj")
        .args(["root"])
        .current_dir(path)
        .output()
        .context("failed to run 'jj root'")?;

    if !out.status.success() {
        bail!(
            "not a jj repository at {}: {}",
            path.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }

    Ok(PathBuf::from(
        String::from_utf8_lossy(&out.stdout).trim().to_owned(),
    ))
}

/// Runs `jj <args>` in the repository root and returns stdout as a String.
fn jj_run(root: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("jj")
        .args(args)
        // Disable interactive output; force machine-readable mode.
        .arg("--no-pager")
        .current_dir(root)
        .output()
        .with_context(|| format!("failed to run 'jj {}'", args.join(" ")))?;

    if !out.status.success() {
        bail!(
            "'jj {}' failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }

    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

// ── VcsBackend impl ──────────────────────────────────────────────────────── //

impl VcsBackend for JjBackend {
    fn status_digest(&self) -> Result<StatusDigest> {
        // `jj log` for the current change (@).
        // Template fields: change_id, commit_id, description, author.timestamp
        let raw = jj_run(
            &self.root,
            &[
                "log",
                "--no-graph",
                "--revsets=@",
                "--template",
                r#"change_id ++ "\t" ++ commit_id ++ "\t" ++ description.first_line() ++ "\t" ++ author.timestamp().format("%s") ++ "\n""#,
            ],
        )?;

        let repo_name = self
            .root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_owned();

        let current_branch = jj_run(&self.root, &["branch", "list", "--revsets=@"])
            .ok()
            .and_then(|s| {
                s.lines()
                    .next()
                    .and_then(|l| l.split(':').next())
                    .map(str::trim)
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| "@".to_owned());

        let info = parse::log_line(&raw.trim())?;

        Ok(StatusDigest {
            repo_name,
            current_branch,
            last_commit_id: info.commit_id,
            last_commit_summary: info.summary,
            last_commit_timestamp: info.timestamp,
        })
    }

    fn local_branches(&self) -> Result<Vec<BranchInfo>> {
        let raw = jj_run(
            &self.root,
            &[
                "branch",
                "list",
                "--template",
                r#"name ++ "\t" ++ target.commit_id() ++ "\t" ++ target.description().first_line() ++ "\t" ++ target.author().timestamp().format("%s") ++ "\n""#,
            ],
        )?;

        raw.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|line| parse::branch_line(line, "refs/heads/"))
            .collect()
    }

    fn remote_branches(&self) -> Result<Vec<BranchInfo>> {
        // jj remote branches are tracked under `refs/remotes/` in the colocated git repo.
        let raw = jj_run(
            &self.root,
            &[
                "branch",
                "list",
                "--all-remotes",
                "--template",
                r#"name ++ "\t" ++ remote ++ "\t" ++ target.commit_id() ++ "\t" ++ target.description().first_line() ++ "\t" ++ target.author().timestamp().format("%s") ++ "\n""#,
            ],
        )
        .unwrap_or_default();

        raw.lines()
            .filter(|l| !l.trim().is_empty() && l.contains('\t'))
            .map(|line| parse::remote_branch_line(line))
            .collect()
    }

    fn list_commits(&self) -> Result<Vec<CommitInfo>> {
        let raw = jj_run(
            &self.root,
            &[
                "log",
                "--no-graph",
                "--revsets=ancestors(@, all())",
                "--template",
                r#"commit_id ++ "\t" ++ author.name() ++ "\t" ++ committer.name() ++ "\t" ++ description.first_line() ++ "\t" ++ author.timestamp().format("%s") ++ "\t" ++ committer.timestamp().format("%s") ++ "\n""#,
            ],
        )?;

        raw.lines()
            .filter(|l| !l.trim().is_empty())
            .map(parse::commit_line)
            .collect()
    }

    fn list_commits_sorted(&self, order: SortOrder) -> Result<Vec<CommitInfo>> {
        let mut commits = self.list_commits()?;
        match order {
            SortOrder::NewestFirst => commits.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)),
            SortOrder::OldestFirst => commits.sort_by(|a, b| a.timestamp.cmp(&b.timestamp)),
            SortOrder::ByName => commits.sort_by(|a, b| a.summary.cmp(&b.summary)),
        }
        Ok(commits)
    }

    fn log_since(&self, since: SystemTime, until: SystemTime) -> Result<Vec<CommitInfo>> {
        let commits = self.list_commits()?;
        Ok(commits
            .into_iter()
            .filter(|c| c.timestamp >= since && c.timestamp <= until)
            .collect())
    }

    fn find_commit(&self, id: &CommitId) -> Result<CommitInfo> {
        let raw = jj_run(
            &self.root,
            &[
                "log",
                "--no-graph",
                &format!("--revsets={}", id),
                "--template",
                r#"commit_id ++ "\t" ++ author.name() ++ "\t" ++ committer.name() ++ "\t" ++ description.first_line() ++ "\t" ++ author.timestamp().format("%s") ++ "\t" ++ committer.timestamp().format("%s") ++ "\n""#,
            ],
        )?;

        let line = raw
            .lines()
            .find(|l| !l.trim().is_empty())
            .ok_or_else(|| anyhow::anyhow!("commit '{}' not found", id))?;
        parse::commit_line(line)
    }

    fn list_tags(&self) -> Result<Vec<TagInfo>> {
        let raw = jj_run(
            &self.root,
            &[
                "tag",
                "list",
                "--template",
                r#"name ++ "\t" ++ target.commit_id() ++ "\t" ++ target.description().first_line() ++ "\t" ++ target.author().timestamp().format("%s") ++ "\n""#,
            ],
        )
        .unwrap_or_default();

        raw.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|line| parse::tag_line(line))
            .collect()
    }

    fn list_tags_sorted(&self, order: SortOrder) -> Result<Vec<TagInfo>> {
        let mut tags = self.list_tags()?;
        match order {
            SortOrder::NewestFirst => tags.sort_by(|a, b| b.commit_timestamp.cmp(&a.commit_timestamp)),
            SortOrder::OldestFirst => tags.sort_by(|a, b| a.commit_timestamp.cmp(&b.commit_timestamp)),
            SortOrder::ByName => tags.sort_by(|a, b| a.name.cmp(&b.name)),
        }
        Ok(tags)
    }

    fn create_tag(&self, name: &str) -> Result<()> {
        jj_run(&self.root, &["tag", "create", name, "-r", "@"])?;
        Ok(())
    }

    fn create_annotated_tag(&self, name: &str, _message: &str) -> Result<()> {
        // jj only has lightweight tags; create as a regular tag.
        self.create_tag(name)
    }

    fn delete_tag(&self, name: &str) -> Result<()> {
        jj_run(&self.root, &["tag", "delete", name])?;
        Ok(())
    }

    fn diff(&self, from: &CommitId, to: &CommitId) -> Result<DiffSummary> {
        let raw = jj_run(
            &self.root,
            &[
                "diff",
                "--no-pager",
                "--summary",
                &format!("--from={}", from),
                &format!("--to={}", to),
            ],
        )?;

        let mut summary = DiffSummary::default();
        for line in raw.lines() {
            if let Some(rest) = line.strip_prefix("A ") {
                summary.added.push(PathBuf::from(rest.trim()));
            } else if let Some(rest) = line.strip_prefix("M ") {
                summary.modified.push(PathBuf::from(rest.trim()));
            } else if let Some(rest) = line.strip_prefix("D ") {
                summary.deleted.push(PathBuf::from(rest.trim()));
            } else if let Some(rest) = line.strip_prefix("R ") {
                // Rename: "R old-path new-path"
                let mut parts = rest.trim().splitn(2, ' ');
                if let (Some(old), Some(new)) = (parts.next(), parts.next()) {
                    summary.deleted.push(PathBuf::from(old));
                    summary.added.push(PathBuf::from(new));
                }
            }
        }

        Ok(summary)
    }

    fn remote_url(&self, name: &str) -> Option<String> {
        let raw = jj_run(&self.root, &["git", "remote", "list"]).ok()?;
        for line in raw.lines() {
            // Format: "name  url  (fetch)"
            let mut parts = line.split_whitespace();
            if parts.next()? == name {
                let url = parts.next()?;
                return Some(url.to_owned());
            }
        }
        None
    }
}
