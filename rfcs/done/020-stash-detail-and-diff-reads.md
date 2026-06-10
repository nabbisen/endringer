# RFC 020 — Stash detail and stash diff reads

**Status.** Implemented (v0.30.0)  
**Priority.** P3  
**Target band.** v0.29.x  
**Breaking change.** Adds API  
**Primary area.** Repository metadata / stash

---

## 1. Summary

Extend stash support from listing entries to reading stash details and diffs, while keeping all stash mutation out of scope.

---

## 2. Motivation

Current `stash_entries()` returns index, commit ID, and message. This is useful for an inventory list but insufficient for UIs that want to show what a stash contains before a user decides what to do with it.

Applying, dropping, popping, or creating a stash are writes and remain out of scope. Reading the files changed by a stash belongs in endringer.

---

## 3. Goals

- Add a stable way to identify a stash entry.
- Add stash detail metadata.
- Add a read-only stash diff summary.
- Preserve newest-first ordering for stash lists.
- Avoid parsing human-readable git CLI output.

---

## 4. Non-goals

- Do not implement stash create, apply, pop, branch, or drop.
- Do not require the git CLI at runtime.
- Do not expose internal gix types.
- Do not promise perfect representation of every stash shape in the first version.

---

## 5. External design

### 5.1 New types

```rust
pub struct StashId {
    pub index: usize,
}

pub struct StashDetail {
    pub id: StashId,
    pub commit_id: CommitId,
    pub message: String,
    pub author: Option<String>,
    pub timestamp: Option<SystemTime>,
    pub parents: Vec<CommitId>,
}
```

`StashId` may start as a wrapper over `usize`. A future version can add reflog selector text if needed.

### 5.2 New methods

```rust
Repository::stash_detail(index: usize) -> Result<StashDetail>
Repository::stash_diff(index: usize) -> Result<DiffSummary>
```

`stash_diff(index)` should represent the files changed by the stash relative to its base parent, documented precisely.

### 5.3 Semantics

A git stash commit commonly has multiple parents. The first implementation should define the diff as:

- default: stash commit tree vs first parent tree;
- optional future: separate staged/unstaged/untracked components if the stash shape allows it.

If the stash shape is unknown, return `UnsupportedBackendFeature` or a documented conservative diff.

---

## 6. Internal design

### 6.1 Git backend

Read `refs/stash` reflog as today for list order. For details, locate the stash commit by reflog index and inspect the commit object.

For diff, reuse existing `diff(from, to)` internals with the first parent and stash commit.

### 6.2 Error behavior

- Missing stash ref: `stash_entries()` returns empty list.
- Missing index: `NotFound { kind: ... }` after RFC 006.
- Malformed stash commit: `CorruptRepository` or `Backend`.

### 6.3 jj backend

Delegate if a git-style stash ref exists. Otherwise return empty or unsupported, based on verified behavior from RFC 007.

---

## 7. Tests and verification

- Repository with no stash returns empty list.
- Repository with one stash returns detail and diff.
- Multiple stashes preserve index ordering.
- Invalid index returns typed not-found error after RFC 006.
- No CLI runtime use.

---

## 8. Rollout plan

1. Add `StashId` and `StashDetail`.
2. Implement `stash_detail`.
3. Implement `stash_diff` using existing diff machinery.
4. Document stash diff semantics clearly.

---

## 9. Risks and mitigations

**Risk: stash commit shapes vary.** Start with first-parent semantics and document it.

**Risk: callers mistake this for stash management.** Keep method names read-only and docs explicit.

**Risk: jj behavior is unclear.** Gate jj claims behind RFC 007 verification.

---

## 10. Definition of done

- Stash detail and diff APIs exist.
- Existing `stash_entries()` remains unchanged.
- Tests cover no-stash, one-stash, multi-stash, and invalid-index cases.
- Documentation states exact diff semantics.
