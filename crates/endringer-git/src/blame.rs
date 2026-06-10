//! Per-line commit attribution via `gix::Repository::blame_file`.

use std::path::Path;

use anyhow::{Context, Result};
use endringer_core::types::{BlameEntry, CommitId};
use gix::bstr::ByteSlice;
use gix::Repository;

pub(crate) fn blame(repo: &Repository, path: &Path) -> Result<Vec<BlameEntry>> {
    if repo.workdir().is_none() {
        anyhow::bail!("blame is not supported on bare repositories");
    }

    // Determine the HEAD commit to start blame from.
    let head_id = repo
        .head()?
        .id()
        .ok_or_else(|| anyhow::anyhow!("no HEAD commit — repository is empty"))?
        .detach();

    // Convert the path to the byte-string format gix expects.
    // Paths are always relative to the repository root and use forward slashes.
    let path_str = path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("path is not valid UTF-8: {}", path.display()))?;
    // Normalise OS path separators to the forward-slash convention git uses.
    let normalised = path_str.replace('\\', "/");
    let path_bstr = gix::bstr::BStr::new(normalised.as_bytes());

    let outcome = repo
        .blame_file(path_bstr, head_id, Default::default())
        .with_context(|| format!("blame failed for '{}'", path.display()))?;

    let entries = outcome
        .entries
        .iter()
        .map(|e| {
            // gix uses 0-based line indices; we expose 1-based inclusive ranges.
            let start_line = e.start_in_blamed_file + 1;
            let end_line = e.start_in_blamed_file + e.len.get(); // inclusive

            let original_path = e.source_file_name.as_ref().map(|s| {
                std::path::PathBuf::from(
                    s.to_os_str_lossy().as_ref() as &std::ffi::OsStr,
                )
            });

            BlameEntry {
                commit_id: CommitId::from_bytes(e.commit_id.as_slice().to_vec()),
                start_line,
                end_line,
                original_path,
            }
        })
        .collect();

    Ok(entries)
}

/// Returns per-line commit attribution for `path` as it exists at `commit_id`.
///
/// Identical to [`blame`] except the blame starts from `commit_id` rather
/// than the current HEAD.
pub(crate) fn blame_at(
    repo: &Repository,
    path: &Path,
    commit_id: &CommitId,
) -> Result<Vec<BlameEntry>> {
    if repo.workdir().is_none() {
        anyhow::bail!("blame_at is not supported on bare repositories");
    }

    let oid = gix::ObjectId::from_hex(commit_id.to_string().as_bytes())
        .map_err(|_| anyhow::anyhow!("invalid commit id '{}'", commit_id))?;

    let path_str = path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("path is not valid UTF-8: {}", path.display()))?;
    let normalised = path_str.replace('\\', "/");
    let path_bstr = gix::bstr::BStr::new(normalised.as_bytes());

    let outcome = repo
        .blame_file(path_bstr, oid, Default::default())
        .with_context(|| format!("blame_at failed for '{}' at '{}'", path.display(), commit_id.short()))?;

    let entries = outcome
        .entries
        .iter()
        .map(|e| {
            let start_line = e.start_in_blamed_file + 1;
            let end_line   = e.start_in_blamed_file + e.len.get();
            let original_path = e.source_file_name.as_ref().map(|s| {
                std::path::PathBuf::from(s.to_os_str_lossy().as_ref() as &std::ffi::OsStr)
            });
            BlameEntry {
                commit_id: CommitId::from_bytes(e.commit_id.as_slice().to_vec()),
                start_line,
                end_line,
                original_path,
            }
        })
        .collect();

    Ok(entries)
}
