//! Git in-progress operation detection (RFC 008).
//!
//! Reads marker files and directories under the git directory to determine
//! whether a merge, rebase, cherry-pick, revert, or bisect is in progress.
//!
//! Detection order (mirrors `git status`):
//! 1. `rebase-merge/`      → `Rebase { kind: Merge }`
//! 2. `rebase-apply/`      → `Rebase { kind: Apply }`
//! 3. `MERGE_HEAD`         → `Merge { heads }`
//! 4. `CHERRY_PICK_HEAD`   → `CherryPick { head }`
//! 5. `REVERT_HEAD`        → `Revert { head }`
//! 6. `BISECT_LOG`         → `Bisect`
//! 7. `refs/bisect/` directory with entries → `Bisect`
//! 8. (none)               → `None`

use std::path::Path;

use anyhow::Result;
use endringer_core::types::{CommitId, OperationState, RebaseKind};

pub(crate) fn operation_state(git_dir: &Path) -> Result<OperationState> {
    // 1. rebase-merge/
    if git_dir.join("rebase-merge").is_dir() {
        return Ok(OperationState::Rebase { kind: RebaseKind::Merge });
    }

    // 2. rebase-apply/
    if git_dir.join("rebase-apply").is_dir() {
        return Ok(OperationState::Rebase { kind: RebaseKind::Apply });
    }

    // 3. MERGE_HEAD
    let merge_head = git_dir.join("MERGE_HEAD");
    if merge_head.exists() {
        let heads = read_commit_ids_from_file(&merge_head)?;
        return Ok(OperationState::Merge { heads });
    }

    // 4. CHERRY_PICK_HEAD
    let cherry = git_dir.join("CHERRY_PICK_HEAD");
    if cherry.exists() {
        let head = read_single_commit_id(&cherry)?;
        return Ok(OperationState::CherryPick { head });
    }

    // 5. REVERT_HEAD
    let revert = git_dir.join("REVERT_HEAD");
    if revert.exists() {
        let head = read_single_commit_id(&revert)?;
        return Ok(OperationState::Revert { head });
    }

    // 6. BISECT_LOG
    if git_dir.join("BISECT_LOG").exists() {
        return Ok(OperationState::Bisect);
    }

    // 7. refs/bisect/ directory with any entries
    let bisect_dir = git_dir.join("refs").join("bisect");
    if bisect_dir.is_dir() {
        if let Ok(mut entries) = std::fs::read_dir(&bisect_dir) {
            if entries.next().is_some() {
                return Ok(OperationState::Bisect);
            }
        }
    }

    Ok(OperationState::None)
}

/// Reads all non-empty lines from `path` as `CommitId`s.
/// Lines that fail to parse are silently skipped (best-effort).
fn read_commit_ids_from_file(path: &Path) -> Result<Vec<CommitId>> {
    let content = std::fs::read_to_string(path)?;
    let mut ids = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(id) = CommitId::from_hex(trimmed) {
            ids.push(id);
        }
    }
    Ok(ids)
}

/// Reads the first non-empty line from `path` as a `CommitId`.
/// Returns `None` if the file is empty or unparseable.
fn read_single_commit_id(path: &Path) -> Result<Option<CommitId>> {
    let content = std::fs::read_to_string(path)?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        return Ok(CommitId::from_hex(trimmed).ok());
    }
    Ok(None)
}
