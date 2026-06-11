use std::path::PathBuf;

use anyhow::{Context, Result};
use endringer_core::types::{CommitId, DiffSummary};
use gix::bstr::ByteSlice;
use gix::Repository;

fn to_path(loc: &gix::bstr::BStr) -> PathBuf {
    PathBuf::from(loc.to_os_str_lossy().as_ref() as &std::ffi::OsStr)
}

pub(crate) fn diff(repository: &Repository, from: &CommitId, to: &CommitId) -> Result<DiffSummary> {
    let from_oid = gix::ObjectId::from_hex(from.to_string().as_bytes())
        .map_err(|_| anyhow::anyhow!("invalid commit id '{}'", from))?;
    let to_oid = gix::ObjectId::from_hex(to.to_string().as_bytes())
        .map_err(|_| anyhow::anyhow!("invalid commit id '{}'", to))?;

    let from_commit = repository
        .find_object(from_oid)
        .with_context(|| format!("commit '{}' not found", from.short()))?
        .try_into_commit()
        .map_err(|_| anyhow::anyhow!("object '{}' is not a commit", from.short()))?;

    let to_commit = repository
        .find_object(to_oid)
        .with_context(|| format!("commit '{}' not found", to.short()))?
        .try_into_commit()
        .map_err(|_| anyhow::anyhow!("object '{}' is not a commit", to.short()))?;

    let from_tree = from_commit.tree()?;
    let to_tree = to_commit.tree()?;

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
            ChangeDetached::Rewrite { source_location, location, .. } => {
                summary.deleted.push(to_path(source_location.as_ref()));
                summary.added.push(to_path(location.as_ref()));
            }
        }
    }

    // Guarantee stable, sorted output within each category.
    summary.added.sort();
    summary.modified.sort();
    summary.deleted.sort();

    Ok(summary)
}

// ── RFC 028: diff_entries ─────────────────────────────────────────────────── //

pub(crate) fn diff_entries(
    repo: &Repository,
    from: &CommitId,
    to: &CommitId,
    options: endringer_core::types::DiffOptions,
) -> Result<Vec<endringer_core::types::DiffEntry>> {
    use endringer_core::types::{DiffChangeKind, DiffEntry};

    // First version: map DiffSummary → DiffEntry without rename detection.
    // When rename detection is requested and gix exposes it, this can be
    // replaced with a richer walk.
    let summary = diff(repo, from, to)?;

    let mut entries: Vec<DiffEntry> = Vec::new();

    for path in summary.added {
        entries.push(DiffEntry { new_path: Some(path), old_path: None, kind: DiffChangeKind::Added, similarity: None });
    }
    for path in summary.modified {
        entries.push(DiffEntry { new_path: Some(path.clone()), old_path: Some(path), kind: DiffChangeKind::Modified, similarity: None });
    }
    for path in summary.deleted {
        entries.push(DiffEntry { new_path: None, old_path: Some(path), kind: DiffChangeKind::Deleted, similarity: None });
    }

    // Sort by new_path (falling back to old_path for deletions).
    entries.sort_by(|a, b| {
        let ak = a.new_path.as_ref().or(a.old_path.as_ref());
        let bk = b.new_path.as_ref().or(b.old_path.as_ref());
        ak.cmp(&bk)
    });

    // Note: detect_renames and detect_copies are accepted in options but
    // not yet implemented. They will produce the same output as detect=false
    // in this version.
    let _ = options;

    Ok(entries)
}
