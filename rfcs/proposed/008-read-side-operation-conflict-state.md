# RFC 008 — Read-side operation and conflict state

**Status.** Proposed  
**Priority.** P2  
**Target band.** v0.25.x  
**Breaking change.** Adds API; trait impact should be eased by RFC 003  
**Primary area.** Working-tree introspection

---

## 1. Summary

Expose read-only information about in-progress repository operations and unmerged/conflicted paths.

This allows a consumer UI to say:

- "merge in progress";
- "rebase in progress";
- "3 paths have conflicts";
- "bisect in progress".

It does **not** resolve conflicts or mutate the repository.

---

## 2. Motivation

`is_dirty()` and `worktree_status()` report file changes, but not the repository operation context. A dirty tree during normal editing is different from a dirty tree in the middle of a merge or rebase.

Read-side operation state is within `endringer`'s boundary because it reads marker files and index state. Resolution remains the consumer's responsibility.

---

## 3. Goals

- Add `operation_state()` for Git marker-file state.
- Add `unmerged_paths()` for paths with higher-stage index entries.
- Optionally add richer `conflict_summary()` with per-stage data.
- Keep jj support deferred until jj verification and stance decisions are complete.

---

## 4. Non-goals

- No `git merge --abort` equivalent.
- No `mark_resolved`.
- No checkout of ours/theirs/base.
- No automatic conflict parsing into hunks.
- No jj-native conflict model in this RFC.

---

## 5. External design

### 5.1 Operation state

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperationState {
    None,
    Merge { heads: Vec<CommitId> },
    Rebase { kind: RebaseKind },
    CherryPick { head: Option<CommitId> },
    Revert { head: Option<CommitId> },
    Bisect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RebaseKind {
    Merge,
    Apply,
    Unknown,
}
```

### 5.2 Conflict paths

Minimal API:

```rust
pub fn unmerged_paths(&self) -> Result<Vec<PathBuf>>;
```

Optional richer API:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictSummary {
    pub paths: Vec<ConflictPath>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictPath {
    pub path: PathBuf,
    pub stages: Vec<ConflictStage>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictStage {
    pub stage: u8, // 1 = base, 2 = ours, 3 = theirs
    pub object_id: ObjectId,
}
```

`ObjectId` is provided by **RFC 031** (foundation pass, v0.21.x), which lands before this RFC, so `ConflictStage.object_id` can use it directly. Implementations may still ship `unmerged_paths()` first and add the richer `ConflictSummary` later, but no longer because the type is missing — only as an incremental scope choice.

### 5.3 Repository methods

```rust
pub fn operation_state(&self) -> Result<OperationState>;
pub fn unmerged_paths(&self) -> Result<Vec<PathBuf>>;
pub fn conflict_summary(&self) -> Result<ConflictSummary>; // optional
```

---

## 6. Internal design

### 6.1 Git operation detection

Read marker files under the git directory.

Suggested detection order:

1. rebase:
   - `rebase-merge/` → `RebaseKind::Merge`
   - `rebase-apply/` → `RebaseKind::Apply`
2. merge:
   - `MERGE_HEAD`
3. cherry-pick:
   - `CHERRY_PICK_HEAD`
4. revert:
   - `REVERT_HEAD`
5. bisect:
   - `BISECT_LOG` or `refs/bisect/*`
6. none.

Order matters because some marker files can coexist in unusual states. Prefer the state Git itself would report as primary.

### 6.2 Reading heads

For marker files containing object IDs:

- parse each non-empty line;
- convert to `CommitId`;
- validate as commit if cheap;
- if invalid, return `CorruptRepository` after RFC 006 or a backend error before it.

### 6.3 Unmerged paths

Read the index and collect entries with stage > 0.

Return unique paths sorted ascending.

Pseudo-code:

```rust
let mut paths = BTreeSet::new();
for entry in index.entries() {
    if entry.stage() > 0 {
        paths.insert(entry.path().to_path_buf());
    }
}
Ok(paths.into_iter().collect())
```

### 6.4 jj behavior

Until jj conflict semantics are designed:

- `operation_state()` may return `OperationState::None` only if verified for the git-view path;
- otherwise return unsupported on jj;
- `unmerged_paths()` returns unsupported on jj native repositories because jj conflicts are not index-stage conflicts.

Do not overclaim jj support.

---

## 7. Test plan

Git fixture tests:

- clean repository → `OperationState::None`, no unmerged paths;
- merge conflict → `Merge`, paths present;
- cherry-pick conflict → `CherryPick`, paths present;
- revert conflict → `Revert`, paths present;
- rebase conflict using merge backend if fixture can create it;
- bisect state if fixture can create it;
- paths sorted and deduplicated;
- bare repository behavior documented and tested.

Async parity tests:

- async `operation_state()` matches sync;
- async `unmerged_paths()` matches sync.

---

## 8. Compatibility

Adds public types and methods.

If added as required trait methods, it breaks custom backends. Prefer RFC 003 first and provide unsupported defaults.

---

## 9. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Git marker-file states are subtle | Match Git's documented marker files and add fixture coverage. |
| Conflict summary requires object IDs beyond commits | Start with `unmerged_paths()` and defer rich summary if necessary. |
| jj conflicts differ fundamentally | Explicitly defer jj support. |
| Consumers ask for resolution helpers | Keep non-goals clear. |

---

## 10. Acceptance criteria

- `OperationState` and `RebaseKind` are public and documented.
- `operation_state()` and `unmerged_paths()` exist in sync and async APIs.
- Git merge/cherry-pick conflict fixtures pass.
- jj behavior is explicitly unsupported or verified; no silent overclaim.
