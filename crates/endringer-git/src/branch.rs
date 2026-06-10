use std::time::SystemTime;

use anyhow::{Context, Result};
use endringer_core::types::{AheadBehind, BranchInfo, BranchTrackingInfo, CommitId, CommitInfo, SortOrder};
use gix::Repository;

use crate::util::{gix_id_to_commit_id, seconds_to_systemtime};
use crate::graph;

mod util;

pub(crate) fn local_branches(repository: &Repository) -> Result<Vec<BranchInfo>> {
    util::branches(repository, "refs/heads/")
}

pub(crate) fn remote_branches(repository: &Repository) -> Result<Vec<BranchInfo>> {
    util::branches(repository, "refs/remotes/")
}

pub(crate) fn list_commits(repository: &Repository) -> Result<Vec<CommitInfo>> {
    collect_commits(repository, |_| true)
}

pub(crate) fn list_commits_sorted(
    repository: &Repository,
    order: SortOrder,
) -> Result<Vec<CommitInfo>> {
    let mut commits = list_commits(repository)?;
    apply_commit_sort(&mut commits, order);
    Ok(commits)
}

pub(crate) fn log_since(
    repository: &Repository,
    since: SystemTime,
    until: SystemTime,
) -> Result<Vec<CommitInfo>> {
    collect_commits(repository, |ts| ts >= since && ts <= until)
}

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
        let committer = commit.committer()?;
        let author_time = author.time().context("failed to read author timestamp")?;
        let committer_time = committer.time().context("failed to read committer timestamp")?;

        let timestamp = seconds_to_systemtime(author_time.seconds);

        if !predicate(timestamp) {
            continue;
        }

        // Collect parent commit IDs from the ancestry graph entry.
        let parents: Vec<CommitId> = info
            .parent_ids
            .iter()
            .copied()
            .map(gix_id_to_commit_id)
            .collect();

        history.push(CommitInfo {
            commit_id: gix_id_to_commit_id(info.id),
            parents,
            summary: message.summary().to_string(),
            author: author.name.to_string(),
            committer: committer.name.to_string(),
            timestamp,
            committer_timestamp: seconds_to_systemtime(committer_time.seconds),
        });
    }

    Ok(history)
}

fn apply_commit_sort(commits: &mut Vec<CommitInfo>, order: SortOrder) {
    match order {
        SortOrder::NewestFirst => commits.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)),
        SortOrder::OldestFirst => commits.sort_by(|a, b| a.timestamp.cmp(&b.timestamp)),
        SortOrder::ByName => commits.sort_by(|a, b| a.summary.cmp(&b.summary)),
    }
}

/// O(1) commit lookup by [`CommitId`].
pub(crate) fn find_commit(repository: &Repository, id: &CommitId) -> Result<CommitInfo> {
    let oid = gix::ObjectId::from_hex(id.to_string().as_bytes())
        .map_err(|_| anyhow::anyhow!("invalid commit id '{}'", id))?;

    let obj = repository
        .find_object(oid)
        .map_err(|e| anyhow::anyhow!("commit '{}' not found: {e}", id.short()))?;
    let commit = obj
        .try_into_commit()
        .map_err(|_| anyhow::anyhow!("object '{}' is not a commit", id.short()))?;

    let author = commit.author()?;
    let committer = commit.committer()?;
    let author_time = author.time().context("author timestamp")?;
    let committer_time = committer.time().context("committer timestamp")?;
    let message = commit.message()?;

    let parents: Vec<CommitId> = commit
        .parent_ids()
        .map(|id| gix_id_to_commit_id(id.detach()))
        .collect();

    Ok(CommitInfo {
        commit_id: id.clone(),
        parents,
        author: author.name.to_string(),
        committer: committer.name.to_string(),
        summary: message.summary().to_string(),
        timestamp: seconds_to_systemtime(author_time.seconds),
        committer_timestamp: seconds_to_systemtime(committer_time.seconds),
    })
}

// ── RFC 005: branch tracking and sync state ──────────────────────────────── //

/// Returns tracking metadata and divergence data for a single local branch.
pub(crate) fn branch_tracking(
    repository: &Repository,
    branch: &str,
) -> Result<BranchTrackingInfo> {
    let full_name = format!("refs/heads/{branch}");
    let tip_commit_id = resolve_local_branch(repository, branch)
        .with_context(|| format!("branch not found: {branch}"))?;

    let upstream = graph::branch_upstream_ref(repository, branch)?;
    let upstream_gone = is_upstream_gone(repository, upstream.as_deref());

    let ahead_behind = if upstream.is_some() && !upstream_gone {
        let upstream_ref = upstream.as_ref().unwrap();
        match resolve_ref_commit(repository, upstream_ref) {
            Ok(upstream_tip) => {
                Some(graph::ahead_behind(repository, &tip_commit_id, &upstream_tip)?)
            }
            Err(_) => None,
        }
    } else {
        None
    };

    Ok(BranchTrackingInfo {
        branch: branch.to_owned(),
        full_name,
        tip_commit_id,
        upstream,
        upstream_gone,
        ahead_behind,
    })
}

/// Returns tracking metadata for all local branches, sorted ascending
/// by full ref name (contract from RFC 005).
pub(crate) fn local_branch_tracking(
    repository: &Repository,
) -> Result<Vec<BranchTrackingInfo>> {
    // Enumerate local branches (already sorted by full_name from util::branches).
    let branches = util::branches(repository, "refs/heads/")?;
    let mut result = Vec::with_capacity(branches.len());

    for b in &branches {
        let upstream = graph::branch_upstream_ref(repository, &b.name)?;
        let upstream_gone = is_upstream_gone(repository, upstream.as_deref());

        let ahead_behind: Option<AheadBehind> = if upstream.is_some() && !upstream_gone {
            let upstream_ref = upstream.as_ref().unwrap();
            match resolve_ref_commit(repository, upstream_ref) {
                Ok(upstream_tip) => {
                    Some(graph::ahead_behind(repository, &b.last_commit_id, &upstream_tip)?)
                }
                Err(_) => None,
            }
        } else {
            None
        };

        result.push(BranchTrackingInfo {
            branch: b.name.clone(),
            full_name: b.full_name.clone(),
            tip_commit_id: b.last_commit_id.clone(),
            upstream,
            upstream_gone,
            ahead_behind,
        });
    }

    Ok(result)
}

/// Returns `true` if `branch` has been merged into `target`.
///
/// Equivalent to `is_ancestor(branch_tip, target_tip)` but named so
/// callers cannot accidentally reverse the arguments.
pub(crate) fn is_merged_into(
    repository: &Repository,
    branch: &str,
    target: &str,
) -> Result<bool> {
    let branch_tip = resolve_local_branch(repository, branch)
        .with_context(|| format!("branch not found: {branch}"))?;
    let target_tip = resolve_local_branch(repository, target)
        .with_context(|| format!("target not found: {target}"))?;
    graph::is_ancestor(repository, &branch_tip, &target_tip)
}

// ── Helpers ──────────────────────────────────────────────────────────────── //

fn resolve_local_branch(repository: &Repository, branch: &str) -> Result<CommitId> {
    resolve_ref_commit(repository, &format!("refs/heads/{branch}"))
}

fn resolve_ref_commit(repository: &Repository, refname: &str) -> Result<CommitId> {
    let mut reference = repository
        .find_reference(refname)
        .with_context(|| format!("reference not found: {refname}"))?;
    let commit = reference.peel_to_commit()?;
    Ok(gix_id_to_commit_id(commit.id))
}

/// Returns `true` if an upstream is configured but the ref does not exist.
fn is_upstream_gone(repository: &Repository, upstream: Option<&str>) -> bool {
    match upstream {
        None => false,
        Some(r) => repository.find_reference(r).is_err(),
    }
}
