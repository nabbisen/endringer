# Types reference

All types live in `endringer` (re-exported from `endringer-core`).

## Identity

### `CommitId`

Opaque commit identifier (SHA-1: 20 bytes / 40 hex; SHA-256: 32 bytes / 64 hex).

```rust
id.short()                 // → 7-char hex abbreviation
id.to_string()             // → full 40 or 64 hex string
id.as_bytes()              // → &[u8]
CommitId::from_hex(hex)    // → Result<CommitId, CommitIdFromHexError>; 40 or 64 chars
CommitId::from_bytes(vec)  // → CommitId
```

Implements `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Display`.

### `ObjectId`

Like `CommitId` but for any git object (blob, tree, tag, commit). Same API.

## Commits

### `CommitInfo`

```rust
pub struct CommitInfo {
    pub commit_id:           CommitId,
    pub parents:             Vec<CommitId>,   // empty for initial commit
    pub author:              String,
    pub committer:           String,
    pub summary:             String,          // first line of commit message
    pub timestamp:           SystemTime,      // author timestamp
    pub committer_timestamp: SystemTime,
}
```

### `CommitQuery` / `CommitQueryResult`

```rust
pub struct CommitQuery {
    pub start:     CommitQueryStart,   // Head | Commit(id) | Ref(name)
    pub max_count: Option<usize>,
    pub skip:      usize,
    pub since:     Option<SystemTime>,
    pub until:     Option<SystemTime>,
    pub order:     SortOrder,
}
// CommitQuery::head_page(n) — first n commits from HEAD, newest first

pub struct CommitQueryResult {
    pub commits:   Vec<CommitInfo>,
    pub truncated: bool,   // true when max_count was reached
}
```

## Branches

### `BranchInfo`

```rust
pub struct BranchInfo {
    pub name:                   String,    // e.g. "main"
    pub full_name:              String,    // e.g. "refs/heads/main"
    pub last_commit_id:         CommitId,
    pub last_commit_summary:    String,
    pub last_commit_timestamp:  SystemTime,
}
```

### `BranchTrackingInfo`

```rust
pub struct BranchTrackingInfo {
    pub tracking:     BranchInfo,
    pub upstream:     Option<String>,     // configured remote-tracking ref
    pub upstream_gone: bool,              // upstream configured but no longer exists
    pub ahead_behind: Option<AheadBehind>,
}
```

### `AheadBehind`

```rust
pub struct AheadBehind {
    pub ahead:  usize,
    pub behind: usize,
}
```

## Status

### `StatusDigest`

```rust
pub struct StatusDigest {
    pub repo_name:             String,
    pub current_branch:        String,   // "(detached)" for detached HEAD
    pub last_commit_id:        CommitId,
    pub last_commit_summary:   String,
    pub last_commit_timestamp: SystemTime,
}
```

### `WorktreeStatus`

```rust
pub struct WorktreeStatus {
    pub staged:    Vec<StatusEntry>,   // index differs from HEAD
    pub unstaged:  Vec<StatusEntry>,   // working tree differs from index
    pub untracked: Vec<PathBuf>,       // not tracked; gitignore applied
}

pub struct StatusEntry {
    pub path: PathBuf,
    pub kind: ChangeKind,   // Added | Modified | Deleted
}
```

### `RichWorktreeStatus`

```rust
pub struct RichWorktreeStatus {
    pub entries: Vec<RichStatusEntry>,   // sorted ascending by path
}

pub struct RichStatusEntry {
    pub path:     PathBuf,
    pub old_path: Option<PathBuf>,               // for renames
    pub index:    Option<FileStatusKind>,
    pub worktree: Option<FileStatusKind>,
    pub conflict: Option<ConflictStatus>,
}

pub enum FileStatusKind {
    Added, Modified, Deleted, Renamed, Copied,
    TypeChanged, ModeChanged, Untracked, Ignored, SubmoduleChanged,
}

pub struct StatusOptions {
    pub include_untracked: bool,   // default: true
    pub include_ignored:   bool,   // default: false
}
```

### `OperationState`

```rust
pub enum OperationState {
    None,
    Merge { heads: Vec<CommitId> },
    Rebase { kind: RebaseKind },         // Merge | Apply | Unknown
    CherryPick { head: Option<CommitId> },
    Revert { head: Option<CommitId> },
    Bisect,
}
```

### `ConflictSummary`

```rust
pub struct ConflictSummary {
    pub paths: Vec<ConflictPath>,   // sorted ascending
}

pub struct ConflictPath {
    pub path:   PathBuf,
    pub stages: Vec<ConflictStage>,
}

pub struct ConflictStage {
    pub stage:     u8,       // 1 = base, 2 = ours, 3 = theirs
    pub object_id: ObjectId,
}
```

## Tags

### `TagInfo`

```rust
pub struct TagInfo {
    pub name:             String,
    pub full_name:        String,
    pub commit_id:        CommitId,              // peeled to commit
    pub commit_summary:   String,
    pub commit_timestamp: SystemTime,
    pub annotation:       Option<TagAnnotation>, // None for lightweight tags
}
```

### `TagAnnotation`

```rust
pub struct TagAnnotation {
    pub message:           String,
    pub tagger_name:       Option<String>,
    pub tagger_email:      Option<String>,
    pub tagger_timestamp:  Option<SystemTime>,
}
```

## Diff and blame

### `DiffSummary`

```rust
pub struct DiffSummary {
    pub added:    Vec<PathBuf>,   // sorted ascending
    pub modified: Vec<PathBuf>,
    pub deleted:  Vec<PathBuf>,
}
```

### `DiffEntry` / `DiffOptions`

```rust
pub struct DiffEntry {
    pub new_path:   Option<PathBuf>,
    pub old_path:   Option<PathBuf>,
    pub kind:       DiffChangeKind,
    pub similarity: Option<u8>,   // 0–100 for renames/copies
}

pub enum DiffChangeKind {
    Added, Modified, Deleted, Renamed, Copied, TypeChanged, ModeChanged,
}

pub struct DiffOptions {
    pub detect_renames:    bool,          // default: false (opt-in)
    pub detect_copies:     bool,
    pub rename_threshold:  Option<u8>,
}
```

### `BlameEntry`

```rust
pub struct BlameEntry {
    pub commit_id:     CommitId,
    pub start_line:    u32,               // 1-indexed, inclusive
    pub end_line:      u32,               // 1-indexed, inclusive
    pub original_path: Option<PathBuf>,   // set when file was renamed
}
```

## Tree entries

### `TreeEntry`

```rust
pub struct TreeEntry {
    pub path:       PathBuf,
    pub name:       String,
    pub kind:       TreeEntryKind,         // File | Directory | Symlink | Submodule | Other
    pub object_id:  ObjectId,
    pub size:       Option<u64>,           // bytes; None for directories/submodules
    pub executable: bool,
}
```

## References and remotes

### `RefInfo`

```rust
pub struct RefInfo {
    pub name:   String,      // e.g. "refs/heads/main", "HEAD"
    pub kind:   RefKind,     // LocalBranch | RemoteBranch | Tag | Head | Other
    pub target: RefTarget,   // Direct(ObjectId) | Symbolic(String) | Unborn
}
```

### `RemoteInfo`

```rust
pub struct RemoteInfo {
    pub name:       String,
    pub fetch_urls: Vec<String>,
    pub push_urls:  Vec<String>,   // empty when no explicit pushurl configured
}
```

## Repository info

### `RepositoryInfo`

```rust
pub struct RepositoryInfo {
    pub repo_name:    String,
    pub path:         PathBuf,
    pub vcs_dir:      PathBuf,
    pub backend:      BackendKind,
    pub head:         HeadState,
    pub object_format: ObjectFormat,
    pub capabilities: RepositoryCapabilities,
}

pub enum HeadState {
    Attached  { branch, full_name, commit_id },
    Detached  { commit_id },
    Unborn    { branch: Option<String> },
    Missing,
}

pub enum ObjectFormat {
    Sha1,
    Sha256,
    Unknown(String),
}
```

## Rich detail types

### `SubmoduleSummary`

```rust
pub struct SubmoduleSummary {
    pub name, path, url: ...,
    pub expected_commit_id:    Option<CommitId>,   // from superproject index
    pub checked_out_commit_id: Option<CommitId>,   // from nested repo HEAD
    pub state:    SubmoduleState,   // Registered | Initialized | MissingWorktree | ...
    pub is_dirty: Option<bool>,
}
```

### `StashDetail`

```rust
pub struct StashDetail {
    pub id:        StashId,
    pub commit_id: CommitId,
    pub message:   String,
    pub author:    Option<String>,
    pub timestamp: Option<SystemTime>,
    pub parents:   Vec<CommitId>,
}
```

### `WorktreeDetail`

```rust
pub struct WorktreeDetail {
    pub id, path, current_branch: ...,
    pub head_commit_id: Option<CommitId>,
    pub is_locked:      bool,
    pub lock_reason:    Option<String>,
    pub state:          WorktreeState,   // Present | MissingPath | MissingGitFile | ...
}
```

## Repository metadata

### `SubmoduleInfo`

```rust
pub struct SubmoduleInfo {
    pub name: String,
    pub path: PathBuf,
    pub url:  Option<String>,
}
```

### `StashEntry`

```rust
pub struct StashEntry {
    pub index:     usize,      // 0 = newest (stash@{0})
    pub commit_id: CommitId,
    pub message:   String,
}
```

### `WorktreeInfo`

```rust
pub struct WorktreeInfo {
    pub id:             String,
    pub path:           PathBuf,
    pub current_branch: String,   // "(detached)" for detached HEAD
    pub is_locked:      bool,
}
```

## Snapshot

### `SnapshotRequest`

```rust
pub struct SnapshotRequest {
    pub include_status_digest:   bool,   // default: true
    pub include_operation_state: bool,   // default: true
    pub include_local_branches:  bool,   // default: false
    pub include_tags:            bool,   // default: false
}
// SnapshotRequest::default() — status + operation state only
```

### `RepositorySnapshot`

```rust
pub struct RepositorySnapshot {
    pub info:            RepositoryInfo,
    pub status_digest:   Option<StatusDigest>,
    pub operation_state: Option<OperationState>,
    pub local_branches:  Option<Vec<BranchInfo>>,
    pub tags:            Option<Vec<TagInfo>>,
}
```

## Enums

`SortOrder` — `NewestFirst | OldestFirst | ByName`.
`BackendKind` — `Git | Jj`.
`CommitIdFromHexError`, `ObjectIdFromHexError` — parse error types.
`RefKind` — `LocalBranch | RemoteBranch | Tag | Head | Other`.
`RefTarget` — `Direct(ObjectId) | Symbolic(String) | Unborn`.
`RebaseKind` — `Merge | Apply | Unknown`.
`SubmoduleState` — `Registered | Initialized | MissingWorktree | MissingGitDir | Detached | Unknown`.
`WorktreeState` — `Present | MissingPath | MissingGitFile | Prunable | Unknown`.
`TreeEntryKind` — `File | Directory | Symlink | Submodule | Other`.
`FileStatusKind` — `Added | Modified | Deleted | Renamed | Copied | TypeChanged | ModeChanged | Untracked | Ignored | SubmoduleChanged`.
`DiffChangeKind` — `Added | Modified | Deleted | Renamed | Copied | TypeChanged | ModeChanged`.
