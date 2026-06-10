//! Stash detail and diff reads (RFC 020).
//!
//! Extends stash support from simple entry listing to commit-object detail
//! and a diff summary of the stash vs its first parent.

use anyhow::{Context, Result};
use endringer_core::types::{DiffSummary, StashDetail, StashId};
use gix::Repository;

use crate::util::{gix_id_to_commit_id, seconds_to_systemtime};

/// Returns detailed metadata for `stash@{index}`.
pub(crate) fn stash_detail(repo: &Repository, index: usize) -> Result<StashDetail> {
    let entries = crate::stash::stash_entries(repo)?;
    let entry = entries.into_iter().find(|e| e.index == index)
        .ok_or_else(|| anyhow::anyhow!("stash@{{{index}}} not found"))?;

    // Resolve the stash commit object to get full author/parents.
    let oid = gix::ObjectId::from_hex(entry.commit_id.to_string().as_bytes())
        .map_err(|_| anyhow::anyhow!("invalid stash commit id"))?;
    let obj = repo.find_object(oid).context("stash commit object not found")?;
    let commit = obj.try_into_commit()
        .map_err(|_| anyhow::anyhow!("stash ref does not point to a commit"))?;

    let author = commit.author().ok().map(|a| a.name.to_string());
    let timestamp = commit.author().ok()
        .and_then(|a| a.time().ok())
        .map(|t| seconds_to_systemtime(t.seconds));
    let parents: Vec<_> = commit
        .parent_ids()
        .map(|id| gix_id_to_commit_id(id.detach()))
        .collect();

    Ok(StashDetail {
        id: StashId { index },
        commit_id: entry.commit_id,
        message: entry.message,
        author,
        timestamp,
        parents,
    })
}

/// Returns a diff summary for `stash@{index}` vs its first parent.
///
/// The diff is: stash commit tree vs first-parent tree. This shows the
/// working-tree changes captured by the stash.
pub(crate) fn stash_diff(repo: &Repository, index: usize) -> Result<DiffSummary> {
    let detail = stash_detail(repo, index)?;
    let stash_id = detail.commit_id.clone();
    let parent_id = detail.parents.into_iter().next()
        .ok_or_else(|| anyhow::anyhow!("stash@{{{index}}} has no parent commit"))?;
    crate::diff::diff(repo, &parent_id, &stash_id)
}
