# RFC 013 — Rich status model

**Status.** Proposed  
**Priority.** P3  
**Target band.** v0.27.x+  
**Breaking change.** No if added as v2 API; replacing current `WorktreeStatus` would be breaking and is deferred  
**Primary area.** Working-tree status fidelity

---

## 1. Summary

Design a richer working-tree status model that can represent more Git states than `Added | Modified | Deleted` while preserving the current simple status API.

---

## 2. Motivation

The current status model is intentionally simple. It is enough for clean/dirty indicators and simple file lists, but serious VCS UIs eventually need to represent:

- renamed files;
- copied files;
- file mode changes;
- type changes;
- conflicts;
- submodule state changes;
- ignored files if requested;
- skip-worktree / assume-unchanged edge cases if exposed later.

Adding this carefully avoids stretching `ChangeKind` until it becomes ambiguous.

---

## 3. Goals

- Add a richer status API without breaking `worktree_status()`.
- Preserve simple API for status widgets.
- Define rename/mode/type/conflict representation.
- Keep ignored files opt-in if included.

---

## 4. Non-goals

- No status porcelain text output.
- No staging/unstaging operations.
- No conflict resolution.
- No file watching.
- No persistent cache.

---

## 5. External design

### 5.1 New status type

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RichWorktreeStatus {
    pub entries: Vec<RichStatusEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RichStatusEntry {
    pub path: PathBuf,
    pub old_path: Option<PathBuf>,
    pub index: Option<FileStatusKind>,
    pub worktree: Option<FileStatusKind>,
    pub conflict: Option<ConflictStatus>,
}
```

### 5.2 Change kind

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileStatusKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    TypeChanged,
    ModeChanged,
    Untracked,
    Ignored,
    SubmoduleChanged,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictStatus {
    pub stages: Vec<u8>,
}
```

### 5.3 Options

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusOptions {
    pub include_untracked: bool,
    pub include_ignored: bool,
    pub detect_renames: bool,
    pub detect_copies: bool,
}
```

Default:

```rust
impl Default for StatusOptions {
    fn default() -> Self {
        Self {
            include_untracked: true,
            include_ignored: false,
            detect_renames: false,
            detect_copies: false,
        }
    }
}
```

### 5.4 API

```rust
pub fn rich_worktree_status(&self, options: StatusOptions) -> Result<RichWorktreeStatus>;
```

The existing `worktree_status()` remains and may be implemented in terms of the richer API later.

---

## 6. Internal design

### 6.1 Initial implementation strategy

Start with statuses gix exposes cheaply and reliably:

- added;
- modified;
- deleted;
- untracked;
- ignored when requested;
- unmerged/conflict via index stages after RFC 008.

Rename/copy detection may be expensive and should stay opt-in.

### 6.2 Avoid breaking simple status

Do not change `ChangeKind` yet. Map rich status into existing simple categories for `worktree_status()`:

- `Renamed` and `Copied` can appear as `Modified` or as add/delete pairs depending on existing behavior;
- document that simple status is intentionally lossy.

### 6.3 Sorting

Sort by path ascending, then by old path if present.

### 6.4 jj behavior

Use git-view behavior only where verified. jj conflicts are deferred.

---

## 7. Test plan

- staged add/modify/delete;
- unstaged modify/delete;
- untracked with gitignore applied;
- ignored included only when option set;
- executable bit change on Unix;
- symlink/file type change where platform supports;
- conflict entries after RFC 008;
- rename detection if implemented;
- simple status still passes existing tests.

---

## 8. Compatibility

No breaking change if this is a new API.

Replacing `WorktreeStatus` is explicitly deferred to a later migration decision.

---

## 9. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Rich status becomes too complex | Add only states with tests and clear consumer value. |
| Rename detection is expensive | Make it opt-in. |
| Simple and rich status disagree | Document simple status as lossy and test common mapping. |

---

## 10. Acceptance criteria

- `RichWorktreeStatus`, `RichStatusEntry`, `FileStatusKind`, and `StatusOptions` are public.
- Existing `worktree_status()` behavior remains unchanged.
- Rich status covers at least current simple states plus one additional tested state.
- Expensive detection is opt-in.
