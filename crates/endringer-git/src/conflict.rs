//! Git index conflict detection (RFC 008).
//!
//! Reads the index and collects entries with stage > 0 (conflicted entries).
//!
//! Stage numbers:
//! - 1 = base (common ancestor)
//! - 2 = ours
//! - 3 = theirs

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Result;
use endringer_core::types::{ConflictPath, ConflictStage, ConflictSummary, ObjectId};
use gix::Repository;

use crate::util::gix_id_to_object_id;

/// Returns paths with unmerged index entries, sorted ascending.
pub(crate) fn unmerged_paths(repo: &Repository) -> Result<Vec<PathBuf>> {
    let index = match repo.try_index()? {
        Some(idx) => idx,
        None => return Ok(vec![]),
    };

    let mut paths: std::collections::BTreeSet<PathBuf> = std::collections::BTreeSet::new();
    for entry in index.entries() {
        if entry.flags.stage() != gix::index::entry::Stage::Unconflicted {
            let path_bytes = entry.path(&index);
            if let Ok(s) = std::str::from_utf8(path_bytes) {
                paths.insert(PathBuf::from(s));
            }
        }
    }
    Ok(paths.into_iter().collect())
}

/// Returns a structured conflict summary with per-stage object IDs.
pub(crate) fn conflict_summary(repo: &Repository) -> Result<ConflictSummary> {
    let index = match repo.try_index()? {
        Some(idx) => idx,
        None => return Ok(ConflictSummary { paths: vec![] }),
    };

    // Collect per-path, per-stage entries.
    // Key = path string, Value = vec of ConflictStage.
    let mut map: BTreeMap<String, Vec<ConflictStage>> = BTreeMap::new();

    for entry in index.entries() {
        let stage = entry.flags.stage();
        if stage == gix::index::entry::Stage::Unconflicted {
            continue;
        }
        let stage_num: u8 = match stage {
            gix::index::entry::Stage::Base   => 1,
            gix::index::entry::Stage::Ours   => 2,
            gix::index::entry::Stage::Theirs => 3,
            _ => continue, // Unconflicted already handled above
        };

        let path_bytes = entry.path(&index);
        let path_str = match std::str::from_utf8(path_bytes) {
            Ok(s) => s.to_owned(),
            Err(_) => continue, // skip non-UTF-8 paths
        };

        let object_id: ObjectId = gix_id_to_object_id(entry.id);

        map.entry(path_str).or_default().push(ConflictStage {
            stage: stage_num,
            object_id,
        });
    }

    let paths: Vec<ConflictPath> = map
        .into_iter()
        .map(|(path_str, mut stages)| {
            stages.sort_by_key(|s| s.stage);
            ConflictPath {
                path: PathBuf::from(path_str),
                stages,
            }
        })
        .collect();

    Ok(ConflictSummary { paths })
}
