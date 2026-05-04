//! Working-tree status: `is_dirty` and full `WorktreeStatus`.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use gix::bstr::ByteSlice;
use gix::Repository;

use endringer_core::types::{ChangeKind, StatusEntry, WorktreeStatus};

use crate::object::collect_blob_oids;

// ── Public API ───────────────────────────────────────────────────────────── //

/// Quick dirty check. Returns `true` if the working tree has any uncommitted
/// changes (staged or unstaged). Bare repos always return `false`.
///
/// Uses the same mtime + file-size heuristic as [`worktree_status`]. Files
/// that were modified **without** changing the size and within the same
/// second as the cached mtime will not be detected. This matches the
/// behaviour of `git update-index --refresh` without content hashing.
pub(crate) fn is_dirty(repo: &Repository) -> Result<bool> {
    if repo.workdir().is_none() {
        return Ok(false);
    }
    let status = worktree_status(repo)?;
    Ok(!status.staged.is_empty()
        || !status.unstaged.is_empty())
}

/// Full working-tree status (staged, unstaged, untracked).
pub(crate) fn worktree_status(repo: &Repository) -> Result<WorktreeStatus> {
    if repo.workdir().is_none() {
        return Ok(WorktreeStatus::default());
    }

    let index = repo.open_index().context("open git index")?;
    let workdir = repo.workdir().expect("checked above");

    // ── Build HEAD blob map (path → oid) ───────────────────────────────── //
    // Maps every blob in HEAD's tree (recursively) by its forward-slash path.
    let head_blobs: HashMap<Vec<u8>, gix::ObjectId> = match repo
        .head()
        .ok()
        .and_then(|mut h| h.peel_to_commit().ok())
    {
        Some(commit) => {
            let tree = commit.tree().context("HEAD tree")?;
            collect_blob_oids(repo, tree.id)?
        }
        None => HashMap::new(), // empty repository
    };

    // ── Build index path set and blob map ─────────────────────────────── //
    // Maps every index entry's forward-slash path → blob oid.
    let mut index_blobs: HashMap<Vec<u8>, gix::ObjectId> = HashMap::new();
    for entry in index.entries() {
        let path = entry.path(&index).to_vec();
        index_blobs.insert(path, entry.id);
    }
    let index_path_set: HashSet<Vec<u8>> = index_blobs.keys().cloned().collect();

    // ── Staged changes (index vs HEAD) ─────────────────────────────────── //
    let mut staged = Vec::new();

    // Staged additions and modifications.
    for (path, &index_oid) in &index_blobs {
        match head_blobs.get(path.as_slice()) {
            None => staged.push(entry(path, ChangeKind::Added)),
            Some(&head_oid) if head_oid != index_oid => {
                staged.push(entry(path, ChangeKind::Modified))
            }
            _ => {}
        }
    }
    // Staged deletions.
    for path in head_blobs.keys() {
        if !index_blobs.contains_key(path.as_slice()) {
            staged.push(entry(path, ChangeKind::Deleted));
        }
    }
    staged.sort_by(|a, b| a.path.cmp(&b.path));

    // ── Unstaged changes (working tree vs index) ───────────────────────── //
    let mut unstaged = Vec::new();
    for entry_ie in index.entries() {
        let path_bytes = entry_ie.path(&index);
        let abs = workdir
            .join(Path::new(path_bytes.to_os_str_lossy().as_ref() as &std::ffi::OsStr));

        match std::fs::metadata(&abs) {
            Err(_) => unstaged.push(entry(path_bytes, ChangeKind::Deleted)),
            Ok(meta) => {
                let cached_secs = entry_ie.stat.mtime.secs;
                let disk_secs = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as u32);
                if disk_secs != Some(cached_secs) || meta.len() as u32 != entry_ie.stat.size {
                    unstaged.push(entry(path_bytes, ChangeKind::Modified));
                }
            }
        }
    }
    unstaged.sort_by(|a, b| a.path.cmp(&b.path));

    // ── Untracked files ────────────────────────────────────────────────── //
    let mut untracked = Vec::new();
    collect_untracked(workdir, workdir, &index_path_set, &mut untracked)?;
    untracked.sort();

    Ok(WorktreeStatus { staged, unstaged, untracked })
}

// ── Helpers ──────────────────────────────────────────────────────────────── //

fn entry(path_bytes: &[u8], kind: ChangeKind) -> StatusEntry {
    let s = String::from_utf8_lossy(path_bytes);
    StatusEntry {
        path: PathBuf::from(s.replace('/', std::path::MAIN_SEPARATOR_STR).as_str()),
        kind,
    }
}

fn collect_untracked(
    workdir: &Path,
    dir: &Path,
    index_paths: &HashSet<Vec<u8>>,
    result: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let e = entry?;
        let abs = e.path();
        let rel = abs.strip_prefix(workdir).unwrap_or(&abs);

        // Always skip the .git directory.
        if rel.components().next().map_or(false, |c| {
            c.as_os_str() == ".git"
        }) {
            continue;
        }

        let ft = e.file_type()?;
        if ft.is_dir() {
            collect_untracked(workdir, &abs, index_paths, result)?;
        } else if ft.is_file() {
            let rel_key = rel
                .to_str()
                .unwrap_or("")
                .replace('\\', "/")
                .into_bytes();
            if !index_paths.contains(&rel_key) {
                result.push(rel.to_path_buf());
            }
        }
    }
    Ok(())
}
