//! Rich linked worktree detail (RFC 021).
//!
//! Enriches the existing worktree listing with HEAD commit, lock reason,
//! and filesystem state. Missing paths are reported as data, not errors.

use std::path::PathBuf;

use anyhow::Result;
use endringer_core::types::{CommitId, WorktreeDetail, WorktreeState};
use gix::Repository;

pub(crate) fn worktree_details(repo: &Repository) -> Result<Vec<WorktreeDetail>> {
    // Locate the .git/worktrees directory.
    let git_dir = repo.git_dir();
    let worktrees_dir = git_dir.join("worktrees");

    if !worktrees_dir.is_dir() {
        return Ok(vec![]);
    }

    let mut details: Vec<WorktreeDetail> = std::fs::read_dir(&worktrees_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| read_worktree_detail(&e.path()))
        .collect();

    details.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(details)
}

fn read_worktree_detail(admin_dir: &std::path::Path) -> WorktreeDetail {
    let id = admin_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_owned();

    // Read the gitdir file to find the worktree's working directory.
    let path = read_worktree_path(admin_dir)
        .unwrap_or_else(|| PathBuf::from("(unknown)"));

    // Check filesystem state.
    let state = if !path.exists() {
        WorktreeState::MissingPath
    } else if !path.join(".git").exists() {
        WorktreeState::MissingGitFile
    } else {
        WorktreeState::Present
    };

    // Read branch from HEAD file inside admin dir.
    let current_branch = read_head_branch(admin_dir)
        .unwrap_or_else(|| "(detached)".to_owned());

    // Read HEAD commit.
    let head_commit_id = read_head_commit(admin_dir);

    // Lock state.
    let lock_path = admin_dir.join("locked");
    let is_locked = lock_path.exists();
    let lock_reason = if is_locked {
        std::fs::read_to_string(&lock_path).ok()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
    } else {
        None
    };

    WorktreeDetail {
        id,
        path,
        current_branch,
        head_commit_id,
        is_locked,
        lock_reason,
        state,
    }
}

/// Read the working directory path from `.git/worktrees/<id>/gitdir`.
fn read_worktree_path(admin_dir: &std::path::Path) -> Option<PathBuf> {
    // The `gitdir` file points to the .git file inside the worktree, e.g.
    // `/path/to/worktree/.git`. The worktree root is its parent.
    let gitdir_content = std::fs::read_to_string(admin_dir.join("gitdir")).ok()?;
    let gitdir_path = std::path::Path::new(gitdir_content.trim());
    // Resolve relative path against admin_dir.
    let resolved = if gitdir_path.is_absolute() {
        gitdir_path.to_path_buf()
    } else {
        admin_dir.join(gitdir_path)
    };
    resolved.canonicalize().ok().and_then(|p| p.parent().map(|par| par.to_path_buf()))
}

/// Read the branch name from `.git/worktrees/<id>/HEAD`.
fn read_head_branch(admin_dir: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(admin_dir.join("HEAD")).ok()?;
    let trimmed = content.trim();
    if let Some(refname) = trimmed.strip_prefix("ref: refs/heads/") {
        Some(refname.to_owned())
    } else {
        None // detached
    }
}

/// Read the HEAD commit OID from `.git/worktrees/<id>/HEAD`.
fn read_head_commit(admin_dir: &std::path::Path) -> Option<CommitId> {
    let content = std::fs::read_to_string(admin_dir.join("HEAD")).ok()?;
    let trimmed = content.trim();
    // Direct OID (detached HEAD).
    if !trimmed.starts_with("ref: ") && trimmed.len() >= 40 {
        return CommitId::from_hex(trimmed).ok();
    }
    None
}
