use std::time::SystemTime;

use anyhow::{Context, Result};
use gix::Repository;

use crate::{
    types::{BranchInfo, CommitId, CommitInfo},
    util::seconds_to_systemtime,
};

mod util;

/// Returns all local branches (`refs/heads/`).
pub(crate) fn local_branches(repository: &Repository) -> Result<Vec<BranchInfo>> {
    util::branches(repository, "refs/heads/")
}

/// Returns all remote-tracking branches (`refs/remotes/`).
pub(crate) fn remote_branches(repository: &Repository) -> Result<Vec<BranchInfo>> {
    util::branches(repository, "refs/remotes/")
}

/// Returns the full commit history reachable from HEAD, newest first.
pub(crate) fn list_commits(repository: &Repository) -> Result<Vec<CommitInfo>> {
    collect_commits(repository, |_| true)
}

/// Returns commits reachable from HEAD whose **author** timestamp falls within
/// `[since, until]` (inclusive on both ends).
///
/// Note: Git history is a DAG and commit timestamps are author-controlled, so
/// this function inspects every ancestor rather than short-circuiting.  For
/// very large repositories consider using a narrow time window.
pub(crate) fn log_since(
    repository: &Repository,
    since: SystemTime,
    until: SystemTime,
) -> Result<Vec<CommitInfo>> {
    collect_commits(repository, |ts| ts >= since && ts <= until)
}

// ------------------------------------------------------------------ //
// Internal helpers
// ------------------------------------------------------------------ //

/// Walks every commit reachable from HEAD and collects those for which
/// `predicate(timestamp)` returns `true`.
fn collect_commits(
    repository: &Repository,
    predicate: impl Fn(SystemTime) -> bool,
) -> Result<Vec<CommitInfo>> {
    let head = repository.head()?;
    let head_id = head
        .id()
        .ok_or_else(|| anyhow::anyhow!("HEAD is not pointing to a commit"))?;

    let mut history = Vec::new();

    for info in head_id.ancestors().all()? {
        let info = info?;
        let commit = info.object()?;

        let message = commit.message()?;
        let author = commit.author()?;
        let author_time = author.time().context("failed to read author timestamp")?;

        let timestamp = seconds_to_systemtime(author_time.seconds);

        if !predicate(timestamp) {
            continue;
        }

        history.push(CommitInfo {
            commit_id: CommitId(info.id),
            summary: message.summary().to_string(),
            author: author.name.to_string(),
            timestamp,
        });
    }

    Ok(history)
}

/// Returns the full commit history reachable from HEAD, sorted by `order`.
pub(crate) fn list_commits_sorted(
    repository: &Repository,
    order: crate::types::SortOrder,
) -> Result<Vec<CommitInfo>> {
    let mut commits = list_commits(repository)?;
    apply_commit_sort(&mut commits, order);
    Ok(commits)
}

fn apply_commit_sort(commits: &mut Vec<CommitInfo>, order: crate::types::SortOrder) {
    use crate::types::SortOrder::*;
    match order {
        NewestFirst => commits.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)),
        OldestFirst => commits.sort_by(|a, b| a.timestamp.cmp(&b.timestamp)),
        ByName => commits.sort_by(|a, b| a.summary.cmp(&b.summary)),
    }
}
