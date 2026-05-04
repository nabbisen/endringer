use anyhow::{Context, Result};
use gix::Repository;

use crate::{
    types::{CommitId, StatusDigest},
    util::seconds_to_systemtime,
};

/// Builds a [`StatusDigest`] from the repository's current HEAD.
pub(crate) fn status_digest(repository: &Repository) -> Result<StatusDigest> {
    // Derive repo name from working directory path.
    // gix may return a relative path (e.g. ".") when the repository was opened
    // from a relative path, so we canonicalize to an absolute path first.
    let repo_name = repository
        .workdir()
        .and_then(|p| p.canonicalize().ok())
        .as_deref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_owned();

    // Resolve the current branch name from HEAD.
    // `referent_name()` returns the full ref (e.g. `refs/heads/main`);
    // `.shorten()` yields the conventional short form (e.g. `main`).
    let mut head = repository.head()?;
    let current_branch = if head.is_detached() {
        "(detached)".to_owned()
    } else {
        head.referent_name()
            .context("failed to resolve HEAD branch name")?
            .shorten()
            .to_string()
    };

    // Peel HEAD to its commit.
    let commit = head.peel_to_commit()?;
    let last_commit_id = CommitId(commit.id);

    let last_commit_summary = commit
        .message()
        .context("failed to read HEAD commit message")?
        .summary()
        .to_string();

    let last_commit_timestamp = seconds_to_systemtime(
        commit
            .time()
            .context("failed to read HEAD commit timestamp")?
            .seconds,
    );

    Ok(StatusDigest {
        repo_name,
        current_branch,
        last_commit_id,
        last_commit_summary,
        last_commit_timestamp,
    })
}
