# RFC 010 — Point-in-time reads and tree snapshots

**Status.** Proposed  
**Priority.** P2  
**Target band.** v0.26.x  
**Breaking change.** Adds API; trait impact should be eased by RFC 003  
**Primary area.** History browsing

---

## 1. Summary

Extend history-oriented reads so consumers can inspect repository state at a specific commit.

Initial additions:

- `blame_at(path, commit_id)`;
- `tree_at_commit(commit_id)`;
- opaque object IDs for tree/blob entries if needed.

---

## 2. Motivation

`file_at_commit(path, commit_id)` already supports historical file content. `blame(path)` is currently HEAD-only. A code-review UI or history browser needs a consistent way to inspect a snapshot without checking it out.

Point-in-time reads are pure reads and fit the library boundary.

---

## 3. Goals

- Add blame at an arbitrary commit.
- Add tree listing at a commit.
- Avoid misusing `CommitId` for blobs and trees.
- Keep results owned and simple.

---

## 4. Non-goals

- No checkout.
- No working tree mutation.
- No streaming tree iterator in the first version.
- No full diff hunk model.
- No semantic language-aware browsing.

---

## 5. External design

### 5.1 Object ID type

`CommitId` currently means commit identifier. Tree entries may point to blobs, trees, commits for submodules, or tags in unusual cases. A generic object ID type is required.

This type — `ObjectId` — is **owned by RFC 031** (Object identity foundation), which lands in the v0.21.x foundation pass *before* this RFC and before RFC 006. RFC 031 defines it as an opaque byte-backed newtype mirroring `CommitId` (`from_hex`/`from_bytes`/`as_bytes`/`short`/`Display`, SHA-1 or SHA-256, `gix::ObjectId` hidden), with explicit lossless `CommitId`↔`ObjectId` conversions:

```rust
// from RFC 031:
pub struct ObjectId(/* private */);
impl ObjectId {
    pub fn from_hex(hex: &str) -> Result<Self, ObjectIdFromHexError>;
    pub fn from_bytes(bytes: Vec<u8>) -> Self;
    pub fn as_bytes(&self) -> &[u8];
    pub fn short(&self) -> String;
}
```

This RFC therefore **consumes** `ObjectId` rather than introducing it. (RFC 031 also notes that a future internal refactor may make `CommitId` a newtype over `ObjectId`; this RFC does not require it.)

### 5.2 Tree entries

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeEntry {
    pub path: PathBuf,
    pub name: String,
    pub kind: TreeEntryKind,
    pub object_id: ObjectId,
    pub size: Option<u64>,
    pub executable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeEntryKind {
    File,
    Directory,
    Symlink,
    Submodule,
    Other,
}
```

### 5.3 New API

```rust
pub fn blame_at(&self, path: &Path, commit_id: &CommitId) -> Result<Vec<BlameEntry>>;

pub fn tree_at_commit(&self, commit_id: &CommitId) -> Result<Vec<TreeEntry>>;

pub fn tree_at_path(&self, commit_id: &CommitId, path: &Path) -> Result<Vec<TreeEntry>>;
```

`tree_at_commit` lists the root tree recursively or non-recursively? This RFC chooses **non-recursive root listing** for the first API and adds `tree_at_path` for directory browsing.

If recursive listing is needed, add later:

```rust
pub fn tree_at_commit_recursive(&self, commit_id: &CommitId) -> Result<Vec<TreeEntry>>;
```

### 5.4 Sorting

Tree entries are sorted by path/name ascending for deterministic output.

---

## 6. Internal design

### 6.1 Git tree traversal

Implementation steps:

1. resolve commit object;
2. peel to tree;
3. if `tree_at_path`, walk path components to a subtree;
4. enumerate entries;
5. map Git file modes to `TreeEntryKind` and `executable`;
6. optionally read blob sizes where cheap.

Mode mapping:

| Git mode | Kind | executable |
|---|---|---|
| `100644` | File | false |
| `100755` | File | true |
| `120000` | Symlink | false |
| `040000` | Directory | false |
| `160000` | Submodule | false |

### 6.2 Blame at commit

Use gix blame support if it accepts a commit/revision parameter. If not, implement by opening the commit's tree and passing appropriate input to blame APIs, or defer until gix API is suitable.

Do not simulate checkout.

### 6.3 jj implementation

Delegate to Git backend for git-view semantics after RFC 007 verifies relevant behavior.

---

## 7. Test plan

Tree tests:

- root with files and directories;
- nested directory listing;
- executable file on Unix where supported;
- symlink on platforms where supported;
- submodule entry;
- missing path;
- path points to file, not directory;
- deterministic sorting.

Blame tests:

- file changed across commits;
- blame at old commit differs from blame at HEAD;
- renamed file behavior documented;
- binary file behavior documented.

Async parity tests for all new methods.

---

## 8. Compatibility

Adds public types and methods.

If `ObjectId` overlaps with `CommitId`, keep both public meanings clear.

---

## 9. Risks and mitigations

| Risk | Mitigation |
|---|---|
| `ObjectId` complicates API | Introduce only because tree entries need non-commit IDs. |
| Recursive listing can be expensive | Start with non-recursive listing. |
| gix blame-at support may be difficult | Split `tree_at_*` first; implement `blame_at` when feasible. |

---

## 10. Acceptance criteria

- `ObjectId`, `TreeEntry`, and `TreeEntryKind` are public if tree APIs land.
- `tree_at_commit` and `tree_at_path` work on Git fixtures.
- `blame_at` works or is split into a follow-up RFC if gix limitations are significant.
- No checkout or working-tree mutation occurs.
