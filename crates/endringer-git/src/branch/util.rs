use anyhow::{Context, Result};
use endringer_core::types::BranchInfo;
use gix::Repository;

use crate::util::{gix_id_to_commit_id, seconds_to_systemtime};

/// Returns branches whose ref names start with `prefix`,
/// sorted ascending by full ref name.
///
/// The sort is enforced at the backend level so consumers can rely on a
/// stable, deterministic order (ascending by `full_name`).
pub(super) fn branches(repository: &Repository, prefix: &str) -> Result<Vec<BranchInfo>> {
    let mut result = Vec::new();

    for reference in repository.references()?.prefixed(prefix)? {
        let mut reference = reference.map_err(|e| anyhow::anyhow!("{e}"))?;
        let full_name = reference.name().as_bstr().to_string();
        let name = full_name
            .strip_prefix(prefix)
            .unwrap_or(&full_name)
            .to_owned();

        let commit = reference.peel_to_commit()?;
        let last_commit_id = gix_id_to_commit_id(commit.id);
        let last_commit_summary = commit
            .message()
            .context("failed to read commit message")?
            .summary()
            .to_string();
        let last_commit_timestamp =
            seconds_to_systemtime(commit.time().context("failed to read commit timestamp")?.seconds);

        result.push(BranchInfo {
            name,
            full_name,
            last_commit_id,
            last_commit_summary,
            last_commit_timestamp,
        });
    }

    // Contract: ascending by full ref name.
    result.sort_by(|a, b| a.full_name.cmp(&b.full_name));
    Ok(result)
}
