use std::path::PathBuf;

use anyhow::{Context, Result};
use gix::bstr::ByteSlice;
use gix::Repository;

use crate::types::{CommitId, DiffSummary};

/// Converts a gix `BStr` location to a `PathBuf` (UTF-8 lossy).
fn to_path(loc: &gix::bstr::BStr) -> PathBuf {
    let os_str = loc.to_os_str_lossy();
    PathBuf::from(os_str.as_ref() as &std::ffi::OsStr)
}

/// Returns a file-level diff summary between two commits.
///
/// Compares the tree of `from` against the tree of `to` and classifies each
/// changed path as added, modified, or deleted.  Patch text is not included.
///
/// Renames are reported as a deletion of the old path plus an addition of the
/// new path (consistent with `git diff --no-renames`).
pub(crate) fn diff(
    repository: &Repository,
    from: &CommitId,
    to: &CommitId,
) -> Result<DiffSummary> {
    // Resolve commit objects from their IDs.
    let from_commit = repository
        .find_object(from.0)
        .with_context(|| format!("commit '{}' not found", from.short()))?
        .try_into_commit()
        .map_err(|_| anyhow::anyhow!("object '{}' is not a commit", from.short()))?;

    let to_commit = repository
        .find_object(to.0)
        .with_context(|| format!("commit '{}' not found", to.short()))?
        .try_into_commit()
        .map_err(|_| anyhow::anyhow!("object '{}' is not a commit", to.short()))?;

    // Peel each commit to its root tree.
    let from_tree = from_commit
        .tree()
        .with_context(|| format!("failed to read tree for commit '{}'", from.short()))?;
    let to_tree = to_commit
        .tree()
        .with_context(|| format!("failed to read tree for commit '{}'", to.short()))?;

    // Use gix's high-level tree-to-tree diff.
    let changes = repository
        .diff_tree_to_tree(Some(&from_tree), Some(&to_tree), None)
        .context("failed to compute tree diff")?;

    let mut summary = DiffSummary::default();

    for change in &changes {
        use gix::object::tree::diff::ChangeDetached;
        match change {
            ChangeDetached::Addition { location, .. } => {
                summary.added.push(to_path(location.as_ref()));
            }
            ChangeDetached::Deletion { location, .. } => {
                summary.deleted.push(to_path(location.as_ref()));
            }
            ChangeDetached::Modification { location, .. } => {
                summary.modified.push(to_path(location.as_ref()));
            }
            ChangeDetached::Rewrite {
                source_location,
                location,
                ..
            } => {
                // Report rewrites as delete + add (no-renames convention).
                summary.deleted.push(to_path(source_location.as_ref()));
                summary.added.push(to_path(location.as_ref()));
            }
        }
    }

    Ok(summary)
}
