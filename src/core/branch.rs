use anyhow::{Context, Result};
use gix::Repository;

use crate::types::BranchInfo;

pub fn local_branches(repository: &Repository) -> Result<Vec<BranchInfo>> {
    branches(&repository, "refs/heads/")
}

pub fn remote_branches(repository: &Repository) -> Result<Vec<BranchInfo>> {
    branches(&repository, "refs/remotes/")
}

fn branches(repository: &Repository, prefix: &str) -> Result<Vec<BranchInfo>> {
    let references = repository
        .references()
        .context("Failed to get references")?;
    let platform = references
        .prefixed(prefix)
        .context("Failed to filter references")?;

    let mut branches = Vec::new();

    for res in platform {
        // 個別の参照取得エラーをスキップするかハンドリングする
        let reference = res.map_err(|e| anyhow::anyhow!("Reference error: {}", e))?;

        let last_commit = reference.clone().peel_to_commit().map_err(
            |e: gix::reference::peel::to_kind::Error| {
                anyhow::anyhow!("Failed to peel reference: {}", e)
            },
        )?;

        let last_commit_id = last_commit.id;

        let last_committer = last_commit.committer()?;
        let last_commit_time = last_committer.time()?;
        let last_commit_timestamp = last_commit_time.seconds;
        let offset = last_commit_time.offset;

        branches.push(BranchInfo {
            // "main"
            name: reference.name().shorten().to_string(),
            // "refs/heads/main"
            full_name: reference.name().as_bstr().to_string(),
            // HEADが指すコミットID
            last_commit_id,
            last_commit_timestamp,
            offset,
        });
    }

    Ok(branches)
}
