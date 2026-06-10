//! Commit-graph operations: merge base, ancestry, and ahead/behind.

use std::collections::{HashMap, VecDeque};

use anyhow::{Context, Result};
use endringer_core::types::{AheadBehind, CommitId};
use gix::Repository;

use crate::util::gix_id_to_commit_id;

fn parse_oid(id: &CommitId) -> Result<gix::ObjectId> {
    gix::ObjectId::from_hex(id.to_string().as_bytes())
        .map_err(|_| anyhow::anyhow!("invalid commit id '{}'", id))
}

// ── merge_base ───────────────────────────────────────────────────────────── //

/// Returns the best common ancestor of `a` and `b`, or `None` if there is
/// none (unrelated histories).
pub(crate) fn merge_base(
    repo: &Repository,
    a: &CommitId,
    b: &CommitId,
) -> Result<Option<CommitId>> {
    let a_oid = parse_oid(a)?;
    let b_oid = parse_oid(b)?;

    match repo.merge_base(a_oid, b_oid) {
        Ok(id) => Ok(Some(gix_id_to_commit_id(id.detach()))),
        Err(e) => {
            let msg = e.to_string();
            // gix returns various forms depending on version:
            // "not found", "NotFound", "Could not find a merge-base"
            if msg.contains("not found")
                || msg.contains("NotFound")
                || msg.contains("Could not find")
                || msg.contains("merge-base")
            {
                Ok(None)
            } else {
                Err(anyhow::anyhow!("merge_base failed: {e}"))
            }
        }
    }
}

// ── is_ancestor ──────────────────────────────────────────────────────────── //

/// Returns `true` if `candidate` is a direct or transitive ancestor of
/// `descendant`. A commit is considered its own ancestor.
pub(crate) fn is_ancestor(
    repo: &Repository,
    candidate: &CommitId,
    descendant: &CommitId,
) -> Result<bool> {
    if candidate == descendant {
        return Ok(true);
    }
    match merge_base(repo, candidate, descendant)? {
        Some(base) => Ok(base == *candidate),
        None => Ok(false),
    }
}

// ── ahead_behind ─────────────────────────────────────────────────────────── //

/// Flags for the symmetric-difference traversal.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct SideFlags(u8);

impl SideFlags {
    const LEFT:  SideFlags = SideFlags(0b01); // reachable from `local`
    const RIGHT: SideFlags = SideFlags(0b10); // reachable from `upstream`
    #[allow(dead_code)]
    const BOTH:  SideFlags = SideFlags(0b11);

    fn is_both(self) -> bool { self.0 == 0b11 }
    fn has_left(self) -> bool { self.0 & 0b01 != 0 }
    fn has_right(self) -> bool { self.0 & 0b10 != 0 }
    fn union(self, other: SideFlags) -> SideFlags { SideFlags(self.0 | other.0) }
    fn adds_to(self, existing: SideFlags) -> bool { (self.0 & !existing.0) != 0 }
}

/// Returns ahead/behind counts between `local` and `upstream` commit tips.
///
/// Equivalent to `git rev-list --left-right --count local...upstream`.
/// Cost is O(commits strictly between the merge base and the two tips).
pub(crate) fn ahead_behind(
    repo: &Repository,
    local: &CommitId,
    upstream: &CommitId,
) -> Result<AheadBehind> {
    // Trivial case.
    if local == upstream {
        return Ok(AheadBehind { ahead: 0, behind: 0, merge_base: Some(local.clone()) });
    }

    let local_oid = parse_oid(local)?;
    let upstream_oid = parse_oid(upstream)?;

    // Validate both are commits.
    repo.find_object(local_oid)
        .with_context(|| format!("local commit not found: {local}"))?
        .try_into_commit()
        .map_err(|_| anyhow::anyhow!("local is not a commit: {local}"))?;
    repo.find_object(upstream_oid)
        .with_context(|| format!("upstream commit not found: {upstream}"))?
        .try_into_commit()
        .map_err(|_| anyhow::anyhow!("upstream is not a commit: {upstream}"))?;

    // Two-pass symmetric difference.
    //
    // Pass 1: walk from both tips, propagating LEFT / RIGHT flags to every
    // reachable ancestor. When a commit accumulates both LEFT and RIGHT it
    // is common history (BOTH). Parents of a BOTH commit are always BOTH.
    //
    // A commit's flags can only grow (never shrink). We enqueue (commit,
    // propagated_flags) tuples; a dequeue is a no-op if the flags in the
    // queue entry add nothing new to the current map value.
    let mb = merge_base(repo, local, upstream)?;

    let mut flags: HashMap<gix::ObjectId, SideFlags> = HashMap::new();
    let mut work: VecDeque<(gix::ObjectId, SideFlags)> = VecDeque::new();

    let push = |oid: gix::ObjectId,
                propagate: SideFlags,
                flags: &mut HashMap<gix::ObjectId, SideFlags>,
                work: &mut VecDeque<(gix::ObjectId, SideFlags)>| {
        let entry = flags.entry(oid).or_default();
        if propagate.adds_to(*entry) {
            *entry = entry.union(propagate);
            work.push_back((oid, *entry));
        }
    };

    push(local_oid,    SideFlags::LEFT,  &mut flags, &mut work);
    push(upstream_oid, SideFlags::RIGHT, &mut flags, &mut work);

    while let Some((oid, propagate)) = work.pop_front() {
        // The flags in the map may have grown since enqueue; use the latest.
        let current = flags.get(&oid).copied().unwrap_or_default();
        // The propagate to parents is the current map value (not the stale
        // enqueued value), so parents inherit all flags accumulated so far.
        for parent_oid in commit_parent_ids(repo, oid)? {
            let parent = flags.entry(parent_oid).or_default();
            if current.adds_to(*parent) {
                *parent = parent.union(current);
                work.push_back((parent_oid, *parent));
            }
        }
        // Suppress unused variable warning for propagate (it acts as the
        // enqueue guard above).
        let _ = propagate;
    }

    // Pass 2: count.
    let mut ahead = 0usize;
    let mut behind = 0usize;
    let mut found_base: Option<CommitId> = mb.clone();

    for (oid, f) in &flags {
        if f.is_both() {
            if found_base.is_none() {
                found_base = Some(gix_id_to_commit_id(*oid));
            }
        } else if f.has_left() {
            ahead += 1;
        } else if f.has_right() {
            behind += 1;
        }
    }

    Ok(AheadBehind {
        ahead,
        behind,
        merge_base: found_base.or(mb),
    })
}

/// Returns the parent IDs of the commit at `oid`.
fn commit_parent_ids(repo: &Repository, oid: gix::ObjectId) -> Result<Vec<gix::ObjectId>> {
    let commit_obj = repo.find_object(oid)
        .with_context(|| format!("object not found: {oid}"))?
        .try_into_commit()
        .map_err(|_| anyhow::anyhow!("not a commit: {oid}"))?;
    let decoded = commit_obj.decode()?;
    decoded.parents.iter()
        .map(|p| {
            gix::ObjectId::from_hex(&**p)
                .map_err(|e| anyhow::anyhow!("invalid parent id in commit {oid}: {e}"))
        })
        .collect()
}

// ── branch upstream resolution ───────────────────────────────────────────── //

/// Resolves the configured upstream ref for a local branch.
///
/// Returns `None` when no upstream is configured.
/// Returns `Err` when the configured upstream ref no longer exists locally.
pub(crate) fn branch_ahead_behind(
    repo: &Repository,
    branch: &str,
) -> Result<Option<AheadBehind>> {
    let config = repo.config_snapshot();

    // gix config: string_by(section, subsection, value_name)
    // For `branch.main.remote`: section="branch", subsection=Some("main"), value="remote"
    let branch_bstr = gix::bstr::BStr::new(branch.as_bytes());

    let remote = config.string_by("branch", Some(branch_bstr), "remote")
        .map(|s| s.to_string());
    let merge = config.string_by("branch", Some(branch_bstr), "merge")
        .map(|s| s.to_string());

    let upstream_ref = match (remote, merge) {
        (None, _) | (_, None) => return Ok(None),
        (Some(remote), Some(merge)) => {
            if remote == "." {
                merge
            } else {
                let short = merge
                    .strip_prefix("refs/heads/")
                    .unwrap_or(&merge);
                format!("refs/remotes/{remote}/{short}")
            }
        }
    };

    let local_ref = format!("refs/heads/{branch}");
    let local_commit = resolve_ref_to_commit(repo, &local_ref)
        .with_context(|| format!("branch not found: {branch}"))?;
    let upstream_commit = resolve_ref_to_commit(repo, &upstream_ref)
        .with_context(|| format!("upstream ref not found: {upstream_ref}"))?;

    Ok(Some(ahead_behind(repo, &local_commit, &upstream_commit)?))
}

fn resolve_ref_to_commit(repo: &Repository, refname: &str) -> Result<CommitId> {
    let mut reference = repo
        .find_reference(refname)
        .with_context(|| format!("reference not found: {refname}"))?;
    let commit = reference.peel_to_commit()?;
    Ok(gix_id_to_commit_id(commit.id))
}
