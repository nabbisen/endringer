//! Working-tree status: `is_dirty` and full `WorktreeStatus`.
//!
//! ## Dirty-check heuristic
//!
//! A stat mismatch (on-disk mtime or size differing from the index's cached
//! stat) is **not** by itself a verdict — it used to be ("fast path: obvious
//! mismatch"), which reported a file as `Modified` whenever its timestamp had
//! changed for any reason (a copied or restored repository, a `touch`, a
//! tool that rewrites a file with identical content), whether or not its
//! content actually differed. `git status` always compares content before
//! reporting a change and only uses the stat cache to *skip* that comparison
//! when it already matches; this module now always compares content too.
//!
//! For every tracked file: the working-tree content is converted to its
//! **git form** — the same conversion `git add` would apply (`core.autocrlf`,
//! `.gitattributes` `eol`/`text`, clean filters, via the repository's own
//! [`gix::filter::Pipeline`]) — hashed as a git blob, and compared with the
//! index entry's own object id. A stat match is no longer trusted as a
//! shortcut: doing so would miss a working-tree conversion that changes
//! content without ever touching mtime or size (an existing `core.autocrlf`
//! checkout, for one). This trades a cheap stat check for reading and
//! converting every tracked file on every call — see `performance.md` for
//! measurements, and the module's own tests for the two false-positive cases
//! this replaces (a stat-only mismatch, and a working-tree conversion the old
//! raw-byte hash ignored).
//!
//! A filter that cannot run (an external `clean` filter whose program is
//! missing, for example) is treated as a mismatch — the file is reported
//! `Modified`, the same conservative call the old code made for any read
//! failure. See `crates/endringer/tests/parity/KNOWN-DEVIATIONS.md`.
//!
//! endringer stays read-only throughout: it never writes the index, never
//! refreshes the on-disk stat cache, and never takes `index.lock` — unlike
//! `git status`, which refreshes the index as a side effect of running.
//!
//! ## Symbolic links
//!
//! An index entry with mode `120000` is read with `symlink_metadata`, never
//! `metadata` — the latter follows the link, which would hash the **target
//! file's** content instead of comparing anything about the link itself, and
//! would report a dangling link as deleted. The content compared is the
//! link's own target path (`read_link`), exactly what git stores as the
//! blob — no filter pipeline runs for a symlink; `core.autocrlf` and
//! `.gitattributes` apply to file content, not link targets. On a platform
//! or configuration where `core.symlinks=false` materialised the link as a
//! plain file holding the target path as text (the common case on Windows),
//! the entry is still mode `120000` in the index, but the path on disk is an
//! ordinary file — that file's raw bytes are hashed directly, still with no
//! filter.
//!
//! ## Submodules
//!
//! An index entry with mode `160000` (a gitlink) is never read as a file.
//! An uninitialized submodule (no `.git` inside the path) is clean — there
//! is nothing to compare. An initialized one is opened as its own
//! repository, at its own path (`gix::open`, never `gix::discover` —
//! discovery searches upward, and a broken `.git` inside the submodule must
//! never be silently resolved by opening an ancestor repository instead).
//! It is `Modified` when its checked-out HEAD differs from the gitlink's
//! recorded commit, or when its own `worktree_status` is not clean (staged,
//! unstaged or untracked) — the same two reasons `git status` marks a
//! submodule modified. Once the submodule is known to be initialized,
//! **any** failure to read it — opening its repository, reading or peeling
//! its HEAD (an unborn HEAD included: there is nothing to match the gitlink
//! against), or computing its own status — is also `Modified`, never
//! silently `None`; the same conservative call this module already makes
//! for an unreadable file, above. `submodule.<name>.ignore` and
//! `diff.ignoreSubmodules` are not honoured; see
//! `crates/endringer/tests/parity/KNOWN-DEVIATIONS.md`. The untracked walk
//! does not descend into a gitlink's own directory — its files belong to the
//! submodule's own status, read above, not the parent's untracked list.
//!
//! ## Gitignore
//!
//! Untracked files are filtered through the repository's active ignore rules
//! (`.gitignore`, `$GIT_DIR/info/exclude`, global excludes) via gix's
//! exclude-stack.  Files that match an ignore rule do not appear in
//! `WorktreeStatus::untracked`.

use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use gix::bstr::ByteSlice;
use gix::Repository;

use endringer_core::types::{ChangeKind, StatusEntry, WorktreeStatus};

use crate::object::collect_blob_oids;

// ── Public API ───────────────────────────────────────────────────────────── //

/// Quick dirty check. Returns `true` if the working tree has any uncommitted
/// changes (staged or unstaged). Bare repos always return `false`.
pub(crate) fn is_dirty(repo: &Repository) -> Result<bool> {
    if repo.workdir().is_none() {
        return Ok(false);
    }
    let status = worktree_status(repo)?;
    Ok(!status.staged.is_empty() || !status.unstaged.is_empty())
}

/// Full working-tree status (staged, unstaged, untracked).
pub(crate) fn worktree_status(repo: &Repository) -> Result<WorktreeStatus> {
    if repo.workdir().is_none() {
        return Ok(WorktreeStatus::default());
    }

    // A missing `.git/index` (a fresh `git init` before the first `git
    // add`) is not a read failure — git itself treats it as an empty
    // index, listing the working tree's files as untracked rather than
    // refusing to read it at all. `index_or_empty()` is gix's own name
    // for that same reading; `open_index()` (which errors instead) is
    // kept for every other caller in this crate that genuinely needs the
    // on-disk file to exist.
    let index = repo.index_or_empty().context("open git index")?;
    let workdir = repo.workdir().expect("checked above");

    // ── Build HEAD blob map ─────────────────────────────────────────────── //
    let head_blobs: HashMap<Vec<u8>, gix::ObjectId> =
        match repo.head().ok().and_then(|mut h| h.peel_to_commit().ok()) {
            Some(commit) => {
                let tree = commit.tree().context("HEAD tree")?;
                collect_blob_oids(repo, tree.id)?
            }
            None => HashMap::new(),
        };

    // ── Build index blob map ────────────────────────────────────────────── //
    let mut index_blobs: HashMap<Vec<u8>, gix::ObjectId> = HashMap::new();
    for entry in index.entries() {
        index_blobs.insert(entry.path(&index).to_vec(), entry.id);
    }
    let index_path_set: HashSet<Vec<u8>> = index_blobs.keys().cloned().collect();

    // ── Staged changes (index vs HEAD) ──────────────────────────────────── //
    let mut staged = Vec::new();
    for (path, &index_oid) in &index_blobs {
        match head_blobs.get(path.as_slice()) {
            None => staged.push(make_entry(path, ChangeKind::Added)),
            Some(&head_oid) if head_oid != index_oid => {
                staged.push(make_entry(path, ChangeKind::Modified))
            }
            _ => {}
        }
    }
    for path in head_blobs.keys() {
        if !index_blobs.contains_key(path.as_slice()) {
            staged.push(make_entry(path, ChangeKind::Deleted));
        }
    }
    staged.sort_by(|a, b| a.path.cmp(&b.path));

    // ── Unstaged changes (working tree vs index) ────────────────────────── //
    // A filter pipeline, built once per call (not once per file) — see this
    // module's own doc comment for why a stat match is no longer a shortcut,
    // and `performance.md` for what this costs on a large working tree.
    let attrs = repo
        .attributes_only(&*index, gix::worktree::stack::state::attributes::Source::WorktreeThenIdMapping)
        .context("build attributes cache for the filter pipeline")?;
    let mut pipeline = gix::filter::Pipeline::new(repo, attrs.detach())
        .context("build filter pipeline")?;

    // Gitlink (submodule) paths, by their own index path — checked both
    // below (to route them to `submodule_status` instead of the file path)
    // and by `collect_untracked` (to not descend into one).
    let gitlink_paths: HashSet<Vec<u8>> = index
        .entries()
        .iter()
        .filter(|e| e.mode == gix::index::entry::Mode::COMMIT)
        .map(|e| e.path(&index).to_vec())
        .collect();

    let mut unstaged = Vec::new();
    for entry in index.entries() {
        let path_bytes = entry.path(&index);
        let rela_path_os = path_bytes.to_os_str_lossy();
        let rela_path = Path::new(rela_path_os.as_ref() as &std::ffi::OsStr);
        let abs = workdir.join(rela_path);

        if entry.mode == gix::index::entry::Mode::COMMIT {
            if let Some(kind) = submodule_status(&abs, entry.id) {
                unstaged.push(make_entry(path_bytes, kind));
            }
            continue;
        }

        if entry.mode == gix::index::entry::Mode::SYMLINK {
            match symlink_blob_id(&abs) {
                None => unstaged.push(make_entry(path_bytes, ChangeKind::Deleted)),
                Some(oid) if oid != entry.id => {
                    unstaged.push(make_entry(path_bytes, ChangeKind::Modified))
                }
                Some(_) => {}
            }
            continue;
        }

        match std::fs::metadata(&abs) {
            Err(_) => unstaged.push(make_entry(path_bytes, ChangeKind::Deleted)),
            Ok(_) => {
                let disk_oid = git_form_blob_id(&mut pipeline, &abs, rela_path, &index);
                if disk_oid.map_or(true, |oid| oid != entry.id) {
                    unstaged.push(make_entry(path_bytes, ChangeKind::Modified));
                }
            }
        }
    }
    unstaged.sort_by(|a, b| a.path.cmp(&b.path));

    // ── Untracked files (with gitignore) ────────────────────────────────── //
    let mut untracked = Vec::new();
    let mut exclude_stack = repo
        .excludes(
            &*index,
            None,
            gix::worktree::stack::state::ignore::Source::WorktreeThenIdMappingIfNotSkipped,
        )
        .ok(); // If building the exclude stack fails, we fall back to no filtering.

    collect_untracked(
        workdir,
        workdir,
        &index_path_set,
        &gitlink_paths,
        &mut exclude_stack,
        &mut untracked,
    )?;
    untracked.sort();

    Ok(WorktreeStatus { staged, unstaged, untracked })
}

// ── Helpers ──────────────────────────────────────────────────────────────── //

fn make_entry(path_bytes: &[u8], kind: ChangeKind) -> StatusEntry {
    let s = String::from_utf8_lossy(path_bytes);
    StatusEntry {
        path: PathBuf::from(s.replace('/', std::path::MAIN_SEPARATOR_STR).as_str()),
        kind,
    }
}

/// Computes the git blob id a working-tree file would be stored as, were it
/// staged right now: reads `abs` from disk, runs it through the repository's
/// filter pipeline (`core.autocrlf`, `.gitattributes` `eol`/`text`, clean
/// filters — whatever applies at `rela_path`) to obtain its *git form*, and
/// hashes that: `sha1("blob " + size + "\0" + content)`.
///
/// Returns `None` on any I/O or conversion error — treated by the caller the
/// same as a mismatch (`Modified`), the same conservative call the previous
/// raw-byte hash made on a read failure.
fn git_form_blob_id(
    pipeline: &mut gix::filter::Pipeline<'_>,
    abs: &Path,
    rela_path: &Path,
    index: &gix::index::State,
) -> Option<gix::ObjectId> {
    let raw = std::fs::read(abs).ok()?;
    let mut converted = pipeline.convert_to_git(raw.as_slice(), rela_path, index).ok()?;
    let mut content = Vec::new();
    converted.read_to_end(&mut content).ok()?;
    blob_id_of(&content)
}

/// Computes the git blob id for a symlink index entry at `abs`: the link's
/// own target path, unfiltered — not the target file's content, which
/// `metadata`/`fs::read` would silently follow into. Falls back to hashing
/// the path's raw bytes directly when `core.symlinks=false` materialised it
/// as a plain file instead of a real symlink (common on Windows); either
/// way, no filter pipeline runs — `core.autocrlf`/`.gitattributes` apply to
/// file content, not link targets.
///
/// Returns `None` when nothing exists at `abs` at all (`symlink_metadata`
/// itself fails) — a dangling link (the metadata call succeeds; only the
/// target is missing) is not this case, and is hashed normally.
///
/// The target is converted to the bytes git would store exactly as git
/// itself does: on Windows, the separator is normalised (`\` -> `/`), since
/// that is what a checkout on that platform needs and what git's own
/// `convert.c` does; on Unix, the target's raw bytes are used unmodified —
/// a backslash is an ordinary character in a Unix target, and a target
/// need not be UTF-8, so a lossy string conversion can turn a clean link
/// into a false `Modified`.
fn symlink_blob_id(abs: &Path) -> Option<gix::ObjectId> {
    let meta = std::fs::symlink_metadata(abs).ok()?;
    let content: Vec<u8> = if meta.file_type().is_symlink() {
        let target = std::fs::read_link(abs).ok()?;
        target_to_git_bytes(&target)
    } else {
        std::fs::read(abs).ok()?
    };
    blob_id_of(&content)
}

/// A symlink target's bytes, exactly as git would store them in the blob.
#[cfg(unix)]
fn target_to_git_bytes(target: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    target.as_os_str().as_bytes().to_vec()
}

/// A symlink target's bytes, exactly as git would store them in the blob.
#[cfg(windows)]
fn target_to_git_bytes(target: &Path) -> Vec<u8> {
    target.to_string_lossy().replace('\\', "/").into_bytes()
}

/// `sha1("blob " + size + "\0" + content)` — the shared tail of
/// [`git_form_blob_id`] and [`symlink_blob_id`], which otherwise differ
/// entirely in what they read and how.
fn blob_id_of(content: &[u8]) -> Option<gix::ObjectId> {
    let header = format!("blob {}\0", content.len());
    let mut hasher = gix::hash::hasher(gix::hash::Kind::Sha1);
    hasher.update(header.as_bytes());
    hasher.update(content);
    hasher.try_finalize().ok()
}

/// The status of a gitlink (submodule) entry: `None` when it is clean or
/// uninitialized (nothing to report either way); `Some(Modified)` when its
/// checked-out HEAD differs from `expected_commit` (the id the parent's
/// index recorded), when its own `worktree_status` is not clean (staged,
/// unstaged or untracked — the same cases `git status` folds into "modified
/// content"/"untracked content" for a submodule), or when anything about an
/// initialized submodule cannot be read at all: opening its repository,
/// reading or peeling its HEAD (an unborn HEAD included — there is nothing
/// to match the gitlink against), or computing its own status. That is the
/// same conservative call this module's own dirty-check heuristic already
/// makes for an unreadable file — never clean, and the submodule is opened
/// at its own path (`gix::open`), never discovered by searching upward
/// (`gix::discover`) — a broken `.git` inside the submodule must fail here,
/// not silently succeed against an ancestor repository.
/// `submodule.<name>.ignore` and `diff.ignoreSubmodules` are not honoured —
/// see `crates/endringer/tests/parity/KNOWN-DEVIATIONS.md`.
fn submodule_status(abs: &Path, expected_commit: gix::ObjectId) -> Option<ChangeKind> {
    if !abs.join(".git").exists() {
        return None; // uninitialized — nothing checked out to compare
    }

    // From here on, the submodule is initialized: any failure to read it
    // is `Modified`, never `None` — opening its own repository must never
    // fall back to an ancestor one.
    let sub_repo = match gix::open(abs) {
        Ok(repo) => repo,
        Err(_) => return Some(ChangeKind::Modified),
    };
    let checked_out = match sub_repo.head().ok().and_then(|mut h| h.peel_to_commit().ok()) {
        Some(commit) => commit.id,
        None => return Some(ChangeKind::Modified), // unreadable, or unborn
    };
    if checked_out != expected_commit {
        return Some(ChangeKind::Modified);
    }
    match worktree_status(&sub_repo) {
        Ok(sub_status) => {
            if !sub_status.staged.is_empty()
                || !sub_status.unstaged.is_empty()
                || !sub_status.untracked.is_empty()
            {
                Some(ChangeKind::Modified)
            } else {
                None
            }
        }
        Err(_) => Some(ChangeKind::Modified),
    }
}

fn collect_untracked(
    workdir: &Path,
    dir: &Path,
    index_paths: &HashSet<Vec<u8>>,
    gitlink_paths: &HashSet<Vec<u8>>,
    exclude_stack: &mut Option<gix::AttributeStack<'_>>,
    result: &mut Vec<PathBuf>,
) -> Result<()> {
    for e in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let e = e?;
        let abs = e.path();
        let rel = abs.strip_prefix(workdir).unwrap_or(&abs);

        // Skip .git directory.
        if rel.components().next().map_or(false, |c| c.as_os_str() == ".git") {
            continue;
        }

        let ft = e.file_type()?;
        if ft.is_dir() {
            // A gitlink's own directory belongs to the submodule's status,
            // not the parent's untracked list — its path is tracked (as a
            // commit, not a tree), so it is never itself reported untracked
            // either way, but without this check every *file inside* it
            // would be, since none of those paths appear in `index_paths`
            // (that index has no entries for the submodule's own files at
            // all).
            let rel_key = rel.to_str().unwrap_or("").replace('\\', "/").into_bytes();
            if gitlink_paths.contains(&rel_key) {
                continue;
            }
            collect_untracked(workdir, &abs, index_paths, gitlink_paths, exclude_stack, result)?;
        } else if ft.is_file() {
            let rel_key = rel.to_str().unwrap_or("").replace('\\', "/").into_bytes();
            if !index_paths.contains(&rel_key) {
                // Check gitignore rules if the exclude stack is available.
                let ignored = exclude_stack
                    .as_mut()
                    .and_then(|stack| stack.at_path(rel, None).ok())
                    .map_or(false, |platform| platform.is_excluded());
                if !ignored {
                    result.push(rel.to_path_buf());
                }
            }
        }
    }
    Ok(())
}

// ── RFC 013: rich_worktree_status ─────────────────────────────────────────── //

pub(crate) fn rich_worktree_status(
    repo: &gix::Repository,
    options: endringer_core::types::StatusOptions,
) -> anyhow::Result<endringer_core::types::RichWorktreeStatus> {
    use endringer_core::types::{
        ConflictStatus, FileStatusKind, RichStatusEntry, RichWorktreeStatus,
    };
    use std::collections::BTreeMap;

    // Reuse the existing worktree_status to get staged/unstaged/untracked.
    let simple = worktree_status(repo)?;

    let mut entries: BTreeMap<std::path::PathBuf, RichStatusEntry> = BTreeMap::new();

    // Map staged changes.
    for se in &simple.staged {
        let kind = match se.kind {
            endringer_core::types::ChangeKind::Added    => FileStatusKind::Added,
            endringer_core::types::ChangeKind::Modified => FileStatusKind::Modified,
            endringer_core::types::ChangeKind::Deleted  => FileStatusKind::Deleted,
        };
        entries.entry(se.path.clone()).or_insert_with(|| RichStatusEntry {
            path: se.path.clone(), old_path: None,
            index: None, worktree: None, conflict: None,
        }).index = Some(kind);
    }

    // Map unstaged changes.
    for se in &simple.unstaged {
        let kind = match se.kind {
            endringer_core::types::ChangeKind::Added    => FileStatusKind::Added,
            endringer_core::types::ChangeKind::Modified => FileStatusKind::Modified,
            endringer_core::types::ChangeKind::Deleted  => FileStatusKind::Deleted,
        };
        entries.entry(se.path.clone()).or_insert_with(|| RichStatusEntry {
            path: se.path.clone(), old_path: None,
            index: None, worktree: None, conflict: None,
        }).worktree = Some(kind);
    }

    // Conflict info from index stages.
    if let Ok(Some(index)) = repo.try_index() {
        let mut conflict_map: BTreeMap<std::path::PathBuf, Vec<u8>> = BTreeMap::new();
        for entry in index.entries() {
            let stage = entry.flags.stage();
            if stage != gix::index::entry::Stage::Unconflicted {
                let stage_num = match stage {
                    gix::index::entry::Stage::Base   => 1u8,
                    gix::index::entry::Stage::Ours   => 2u8,
                    gix::index::entry::Stage::Theirs => 3u8,
                    _ => continue,
                };
                if let Ok(p) = std::str::from_utf8(entry.path(&index)) {
                    conflict_map.entry(std::path::PathBuf::from(p))
                        .or_default().push(stage_num);
                }
            }
        }
        for (path, stages) in conflict_map {
            entries.entry(path.clone()).or_insert_with(|| RichStatusEntry {
                path, old_path: None, index: None, worktree: None, conflict: None,
            }).conflict = Some(ConflictStatus { stages });
        }
    }

    // Untracked files.
    if options.include_untracked {
        for path in &simple.untracked {
            entries.entry(path.clone()).or_insert_with(|| RichStatusEntry {
                path: path.clone(), old_path: None,
                index: None, worktree: Some(FileStatusKind::Untracked), conflict: None,
            });
        }
    }

    let sorted_entries: Vec<RichStatusEntry> = entries.into_values().collect();
    Ok(RichWorktreeStatus { entries: sorted_entries })
}
