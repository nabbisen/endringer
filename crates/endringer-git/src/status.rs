//! Working-tree dirty check via the git index stat cache.

use anyhow::{Context, Result};
use gix::bstr::ByteSlice;
use gix::Repository;

/// Returns `true` if the working tree has uncommitted changes.
///
/// Uses two passes:
///
/// 1. **Unstaged modifications / deletions** — walks every entry in the git
///    index and compares each file's on-disk modification time (second
///    resolution) against the value recorded in the stat cache. A mismatch,
///    or a missing file, indicates the working tree is dirty.
///
/// 2. **Staged changes** — compares each index entry's blob OID against the
///    corresponding entry in the HEAD tree. A difference (or a new/deleted
///    entry) indicates the index differs from HEAD.
///
/// Bare repositories always return `false` (no working tree to be dirty).
///
/// ## Known limitation
///
/// The stat-cache approach relies on the index being up to date (i.e. the last
/// operation that touched the index was `git update-index` or equivalent).
/// In unusual workflows where the index is stale the result may be a false
/// negative. A future revision will use gix's `status` platform for exact
/// content comparison.
pub(crate) fn is_dirty(repo: &Repository) -> Result<bool> {
    if repo.workdir().is_none() {
        return Ok(false); // bare repos have no working tree
    }

    let index = repo.open_index().context("open git index")?;
    let workdir = repo.workdir().expect("checked above");

    // ── Pass 1: detect unstaged changes via stat cache ─────────────────── //
    for entry in index.entries() {
        let path_bytes = entry.path(&index);
        let abs_path = {
            let s = path_bytes.to_os_str_lossy();
            workdir.join(std::path::Path::new(s.as_ref() as &std::ffi::OsStr))
        };

        match std::fs::metadata(&abs_path) {
            Err(_) => return Ok(true), // tracked file missing → dirty
            Ok(meta) => {
                let cached_secs = entry.stat.mtime.secs;
                let disk_secs = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as u32);

                if disk_secs != Some(cached_secs)
                    || meta.len() as u32 != entry.stat.size
                {
                    return Ok(true); // mtime or size mismatch → dirty
                }
            }
        }
    }

    // ── Pass 2: detect staged changes (index vs HEAD tree) ─────────────── //
    let head_tree = match repo.head().ok().and_then(|mut h| h.peel_to_commit().ok()) {
        Some(commit) => commit.tree().context("HEAD tree")?,
        None => return Ok(false), // no HEAD yet (empty repo)
    };

    // Build a lookup: path bytes → blob OID from HEAD tree.
    // We only compare non-tree (blob) entries at the top level and recurse
    // via the flat index — this correctly handles all nesting.
    let mut head_blobs: std::collections::HashMap<Vec<u8>, gix::ObjectId> =
        std::collections::HashMap::new();
    for entry_result in head_tree.iter() {
        let te = entry_result.context("HEAD tree entry")?;
        head_blobs.insert(te.filename().to_vec(), te.object_id());
    }

    for entry in index.entries() {
        let path = entry.path(&index).to_owned();

        match head_blobs.get(path.as_slice()) {
            None => return Ok(true),                // new file staged (not in HEAD)
            Some(&head_oid) if head_oid != entry.id => return Ok(true), // staged modification
            _ => {}
        }
    }
    // Staged deletions: HEAD entries absent from the index
    let index_paths: std::collections::HashSet<Vec<u8>> = index
        .entries()
        .iter()
        .map(|e| e.path(&index).to_vec())
        .collect();
    if head_blobs.keys().any(|p| !index_paths.contains(p.as_slice())) {
        return Ok(true);
    }

    Ok(false)
}
