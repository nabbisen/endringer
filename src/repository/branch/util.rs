use anyhow::{Context, Result};
use gix::Repository;

use crate::{
    types::{BranchInfo, CommitId},
    util::seconds_to_systemtime,
};

/// Iterates references under `prefix` and converts them to [`BranchInfo`].
pub(super) fn branches(repository: &Repository, prefix: &str) -> Result<Vec<BranchInfo>> {
    let references = repository
        .references()
        .context("failed to get references")?;
    let platform = references
        .prefixed(prefix)
        .context("failed to filter references by prefix")?;

    let mut branches = Vec::new();

    for res in platform {
        let reference = res.map_err(|e| anyhow::anyhow!("reference error: {}", e))?;

        let last_commit = reference
            .clone()
            .peel_to_commit()
            .map_err(|e| anyhow::anyhow!("failed to peel reference to commit: {}", e))?;

        let last_commit_id = CommitId(last_commit.id);

        let last_commit_summary = last_commit
            .message()
            .context("failed to read branch tip commit message")?
            .summary()
            .to_string();

        let last_committer = last_commit.committer()?;
        let last_commit_time = last_committer.time()?;
        let last_commit_timestamp = seconds_to_systemtime(last_commit_time.seconds);

        branches.push(BranchInfo {
            name: reference.name().shorten().to_string(),
            full_name: reference.name().as_bstr().to_string(),
            last_commit_id,
            last_commit_summary,
            last_commit_timestamp,
        });
    }

    Ok(branches)
}
