# RFC 021 — Linked worktree detail and safety metadata

**Status.** Implemented (v0.30.0)  
**Priority.** P3  
**Target band.** v0.29.x  
**Breaking change.** Adds API  
**Primary area.** Repository metadata / worktrees

---

## 1. Summary

Extend linked worktree reads with details useful for repository UIs: prune state, lock reason, HEAD commit, path validity, and relationship to the main repository.

---

## 2. Motivation

Current `WorktreeInfo` reports linked worktree ID, path, current branch, and lock status. That is a good minimal surface, but real repositories can have stale/prunable worktree entries, lock reasons, detached HEADs, missing paths, and branch relationships that consumers may want to display.

Worktree repair, prune, add, move, remove, or lock/unlock are writes and remain out of scope.

---

## 3. Goals

- Keep `worktrees()` as the simple list API.
- Add richer detail without forcing cost on simple callers.
- Report missing/stale worktree entries as data, not surprise errors.
- Read lock reason where available.
- Read HEAD commit where available.

---

## 4. Non-goals

- Do not add, remove, move, prune, lock, or unlock worktrees.
- Do not mutate worktree administrative files.
- Do not recursively scan each worktree's full dirty status by default.

---

## 5. External design

### 5.1 New type

```rust
pub struct WorktreeDetail {
    pub id: String,
    pub path: PathBuf,
    pub current_branch: String,
    pub head_commit_id: Option<CommitId>,
    pub is_locked: bool,
    pub lock_reason: Option<String>,
    pub state: WorktreeState,
}

pub enum WorktreeState {
    Present,
    MissingPath,
    MissingGitFile,
    Prunable,
    Unknown,
}
```

### 5.2 New method

```rust
Repository::worktree_details() -> Result<Vec<WorktreeDetail>>
```

`worktrees()` remains the concise call.

### 5.3 Semantics

- Main worktree remains excluded unless a future RFC adds `all_worktrees()`.
- Linked worktrees are sorted by ID, matching existing behavior.
- Missing linked worktree paths are reported in `state`, not omitted.

---

## 6. Internal design

### 6.1 Git backend

Read `.git/worktrees/*` administrative directories. Reuse current worktree parser and enrich it with:

- lock file contents as `lock_reason`;
- HEAD target and commit resolution;
- path existence checks;
- gitfile presence checks.

### 6.2 Error behavior

Malformed administrative entries may be represented as `Unknown` or as typed errors depending on severity. A single malformed linked worktree should not necessarily make all worktree listing fail.

### 6.3 jj backend

Delegate through git store only where meaningful. jj worktree semantics should not be promised until RFC 007 covers them.

---

## 7. Tests and verification

- Linked worktree present.
- Locked worktree with reason.
- Detached linked worktree.
- Missing worktree path.
- Sorting by ID.
- Main worktree excluded.

---

## 8. Rollout plan

1. Add `WorktreeState` and `WorktreeDetail`.
2. Implement git backend enrichment.
3. Add fixtures for lock/missing/detached cases.
4. Document no mutation and no pruning.

---

## 9. Risks and mitigations

**Risk: overfitting to git internal files.** Use well-known worktree admin layout but keep unknown states possible.

**Risk: callers want prune/remove.** Keep write operations out of scope.

**Risk: jj ambiguity.** Bound jj support until verified.

---

## 10. Definition of done

- Rich worktree detail API exists.
- Existing `worktrees()` stays compatible.
- Tests cover locked, detached, missing, and present linked worktrees.
- Docs clarify read-only scope.
