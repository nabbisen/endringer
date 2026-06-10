//! Rich submodule status summary (RFC 019).
//!
//! Reads `.gitmodules`, resolves expected commits from the index gitlink
//! entries, and opens each initialized submodule to read its HEAD commit.
//! Dirty detection is deferred (`is_dirty: None`) in the first version.

use anyhow::Result;
use endringer_core::types::{CommitId, SubmoduleState, SubmoduleSummary};
use gix::Repository;

pub(crate) fn submodule_summaries(repo: &Repository) -> Result<Vec<SubmoduleSummary>> {
    // Read the .gitmodules list (reuses existing submodule module logic).
    let modules = crate::submodule::submodules(repo)?;
    let workdir = repo.workdir();

    let mut results: Vec<SubmoduleSummary> = modules
        .into_iter()
        .map(|info| {
            let path = info.path.clone();
            let expected = read_gitlink(repo, &path);
            let (state, checked_out) = inspect_submodule(workdir, &path);

            SubmoduleSummary {
                name: info.name,
                path,
                url: info.url,
                expected_commit_id: expected,
                checked_out_commit_id: checked_out,
                state,
                is_dirty: None, // conservative first version
            }
        })
        .collect();

    results.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(results)
}

/// Read the gitlink OID from the index for the given submodule path.
fn read_gitlink(repo: &Repository, submodule_path: &std::path::Path) -> Option<CommitId> {
    let index = repo.try_index().ok()??;
    let path_str = submodule_path.to_str()?;

    for entry in index.entries() {
        let entry_path = std::str::from_utf8(entry.path(&index)).unwrap_or("");
        if entry_path == path_str && entry.mode == gix::index::entry::Mode::COMMIT {
            // Convert gix ObjectId → CommitId via hex representation.
            if let Ok(id) = CommitId::from_hex(&entry.id.to_hex().to_string()) {
                return Some(id);
            }
        }
    }
    None
}

/// Determine the submodule state and checked-out commit from the filesystem.
fn inspect_submodule(
    workdir: Option<&std::path::Path>,
    submodule_path: &std::path::Path,
) -> (SubmoduleState, Option<CommitId>) {
    let base = match workdir {
        Some(d) => d,
        None => return (SubmoduleState::Unknown, None),
    };

    let full_path = base.join(submodule_path);

    if !full_path.exists() {
        return (SubmoduleState::MissingWorktree, None);
    }

    // Check for .git file or directory inside the submodule path.
    let git_marker = full_path.join(".git");
    if !git_marker.exists() {
        return (SubmoduleState::MissingGitDir, None);
    }

    // Try to open the nested repository and read its HEAD.
    match gix::discover(&full_path) {
        Ok(sub_repo) => {
            let head_commit = sub_repo.head().ok()
                .and_then(|mut h| h.peel_to_commit().ok())
                .map(|c| {
                    CommitId::from_hex(&c.id.to_hex().to_string()).ok()
                })
                .flatten();

            let state = if sub_repo.head().ok().map_or(false, |h| h.is_detached()) {
                SubmoduleState::Detached
            } else {
                SubmoduleState::Initialized
            };
            (state, head_commit)
        }
        Err(_) => (SubmoduleState::MissingGitDir, None),
    }
}
