# RFC 028 — Rename and copy detection for diff, blame, and status surfaces

**Status.** Implemented (v0.32.0)  
**Priority.** P3  
**Target band.** v0.30.x or later  
**Breaking change.** May add API; may break if replacing enums  
**Primary area.** Diff/status fidelity

---

## 1. Summary

Design how endringer should expose rename and copy information across diff, blame, and status APIs without making expensive detection mandatory for simple callers.

---

## 2. Motivation

Current `ChangeKind` is minimal: added, modified, deleted. `BlameEntry` has `original_path`, so the library already acknowledges path history. Serious VCS UIs often need rename/copy information in diffs and status views.

Rename detection can be expensive and heuristic. It should be opt-in or clearly documented rather than forced into every cheap status call.

---

## 3. Goals

- Add a richer diff/status representation for renames and copies.
- Keep cheap existing calls cheap.
- Define when rename detection is exact versus heuristic.
- Align `DiffSummary`, `WorktreeStatus`, and `BlameEntry` terminology.

---

## 4. Non-goals

- Do not make rename detection mandatory for `is_dirty()`.
- Do not promise perfect copy detection.
- Do not implement working-tree mutation or path repair.
- Do not expose gix internals.

---

## 5. External design

### 5.1 Rich change kind

Option A: extend `ChangeKind` in a breaking band:

```rust
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed { from: PathBuf },
    Copied { from: PathBuf },
    TypeChanged,
    ModeChanged,
}
```

Option B: keep `ChangeKind` simple and add richer APIs:

```rust
pub struct DiffEntry {
    pub old_path: Option<PathBuf>,
    pub new_path: Option<PathBuf>,
    pub kind: DiffChangeKind,
    pub similarity: Option<u8>,
}

pub struct DiffOptions {
    pub detect_renames: bool,
    pub detect_copies: bool,
    pub rename_threshold: Option<u8>,
}

Repository::diff_entries(from: CommitId, to: CommitId, options: DiffOptions) -> Result<Vec<DiffEntry>>
```

Recommended: Option B first. Keep `DiffSummary` as the simple cheap summary.

### 5.2 Status surface

A future `worktree_status_rich(options)` may be added later. Do not change `worktree_status()` until the cost model is understood.

### 5.3 Blame consistency

Document `BlameEntry.original_path` as rename/copy-path ancestry metadata, not necessarily proof of copy detection.

---

## 6. Internal design

### 6.1 Git backend

Use gix diff capabilities where available. If gix rename detection is limited or expensive, start with commit-to-commit diffs only and defer worktree rename detection.

### 6.2 Options type

Options should be owned, simple, and stable:

```rust
impl Default for DiffOptions {
    fn default() -> Self {
        Self { detect_renames: false, detect_copies: false, rename_threshold: None }
    }
}
```

### 6.3 Performance

Add benchmarks under RFC 017 before enabling detection in high-level calls.

---

## 7. Tests and verification

- Commit diff with a rename.
- Commit diff with delete/add that should not be treated as rename when detection is off.
- Detection threshold behavior if implemented.
- Existing `diff()` behavior unchanged.
- Performance benchmark for rename detection on generated repo.

---

## 8. Rollout plan

1. Add `DiffEntry` and `DiffOptions` as an additive API.
2. Implement commit-to-commit rename detection only.
3. Benchmark.
4. Consider status/worktree rename detection later.

---

## 9. Risks and mitigations

**Risk: heuristic results surprise consumers.** Make detection opt-in and include similarity scores.

**Risk: performance regression.** Keep simple APIs unchanged.

**Risk: enum expansion breaks users.** Prefer additive rich APIs first.

---

## 10. Definition of done

- Additive rich diff API exists.
- Existing `DiffSummary` remains stable and cheap.
- Rename tests exist.
- Docs state heuristic and cost behavior.
