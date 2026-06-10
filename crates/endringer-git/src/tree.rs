//! Tree listing at a commit (RFC 010).
//!
//! Provides non-recursive enumeration of a git tree's direct children at a
//! given commit.  Results are sorted ascending by entry name.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use endringer_core::types::{CommitId, ObjectId, TreeEntry, TreeEntryKind};
use gix::Repository;

use crate::util::gix_id_to_object_id;

// ── Public API ───────────────────────────────────────────────────────────── //

/// Returns the root-level tree entries at `commit_id`, non-recursive.
pub(crate) fn tree_at_commit(
    repo: &Repository,
    commit_id: &CommitId,
) -> Result<Vec<TreeEntry>> {
    let tree_id = resolve_root_tree(repo, commit_id)?;
    list_tree(repo, tree_id, Path::new(""))
}

/// Returns the tree entries of the directory at `path` within `commit_id`.
pub(crate) fn tree_at_path(
    repo: &Repository,
    commit_id: &CommitId,
    path: &Path,
) -> Result<Vec<TreeEntry>> {
    let tree_id = resolve_root_tree(repo, commit_id)?;

    // Walk path components to find the target sub-tree.
    let path_str = path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("non-UTF-8 path: {}", path.display()))?;
    let normalised = path_str.replace('\\', "/");

    // Empty / root path → list the root tree directly.
    if normalised.is_empty() || normalised == "." || normalised == "/" {
        return list_tree(repo, tree_id, path);
    }

    let components: Vec<&[u8]> = normalised
        .split('/')
        .filter(|s| !s.is_empty())
        .map(str::as_bytes)
        .collect();

    let subtree_id = walk_to_subtree(repo, tree_id, &components, path)?;
    list_tree(repo, subtree_id, path)
}

// ── Internals ────────────────────────────────────────────────────────────── //

/// Resolves a commit to its root tree OID.
fn resolve_root_tree(
    repo: &Repository,
    commit_id: &CommitId,
) -> Result<gix::ObjectId> {
    let oid = gix::ObjectId::from_hex(commit_id.to_string().as_bytes())
        .map_err(|_| anyhow::anyhow!("invalid commit id '{}'", commit_id))?;
    let commit = repo
        .find_object(oid)
        .with_context(|| format!("commit '{}' not found", commit_id.short()))?
        .try_into_commit()
        .map_err(|_| anyhow::anyhow!("object '{}' is not a commit", commit_id.short()))?;
    Ok(commit.tree().context("failed to read commit tree")?.id)
}

/// Walks tree entries following path components until the target sub-tree.
fn walk_to_subtree(
    repo: &Repository,
    root: gix::ObjectId,
    components: &[&[u8]],
    original_path: &Path,
) -> Result<gix::ObjectId> {
    let mut current = root;

    for (i, component) in components.iter().enumerate() {
        let obj = repo
            .find_object(current)
            .context("tree object missing during path walk")?;
        let tree = obj
            .try_into_tree()
            .map_err(|_| anyhow::anyhow!("expected tree at component {i}"))?;

        let mut found: Option<gix::ObjectId> = None;
        for entry in tree.iter() {
            let te = entry.context("tree entry decode")?;
            if te.filename() == *component {
                if !te.mode().is_tree() {
                    anyhow::bail!(
                        "path component '{}' in '{}' is a file, not a directory",
                        String::from_utf8_lossy(component),
                        original_path.display()
                    );
                }
                found = Some(te.object_id());
                break;
            }
        }

        current = found.ok_or_else(|| {
            anyhow::anyhow!(
                "path '{}' not found in commit tree",
                original_path.display()
            )
        })?;
    }

    Ok(current)
}

/// Lists the direct children of `tree_id`, building paths relative to `dir`.
fn list_tree(
    repo: &Repository,
    tree_id: gix::ObjectId,
    dir: &Path,
) -> Result<Vec<TreeEntry>> {
    let obj = repo
        .find_object(tree_id)
        .context("tree object not found")?;
    let tree = obj
        .try_into_tree()
        .map_err(|_| anyhow::anyhow!("object is not a tree"))?;

    let mut entries: Vec<TreeEntry> = tree
        .iter()
        .map(|result| {
            let te = result.context("tree entry decode")?;
            let filename_bytes = te.filename();
            let name = String::from_utf8_lossy(filename_bytes).into_owned();
            let path: PathBuf = if dir == Path::new("") {
                PathBuf::from(&name)
            } else {
                dir.join(&name)
            };

            let mode = te.mode();
            let object_id: ObjectId = gix_id_to_object_id(te.object_id());
            let executable = mode.is_executable();

            let (kind, size) = if mode.is_tree() {
                (TreeEntryKind::Directory, None)
            } else if mode.is_link() {
                (TreeEntryKind::Symlink, None)
            } else if mode.is_commit() {
                // Submodule — commit object at the pinned SHA.
                (TreeEntryKind::Submodule, None)
            } else if mode.is_blob() {
                // Regular file — read the blob to get its size.
                let blob_size = repo
                    .find_blob(te.object_id())
                    .map(|b| b.data.len() as u64)
                    .ok();
                (TreeEntryKind::File, blob_size)
            } else {
                (TreeEntryKind::Other, None)
            };

            Ok(TreeEntry {
                path,
                name,
                kind,
                object_id,
                size,
                executable,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // Sort ascending by name (deterministic output).
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}
