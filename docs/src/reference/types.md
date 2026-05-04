# Types

All types live in `endringer` (re-exported from `endringer-core`).

## `CommitId`

Opaque commit identifier (SHA-1 or SHA-256 bytes).

```rust
id.short()          // → "a1b2c3d" (7 chars)
id.to_string()      // → full 40 or 64 char hex
id.as_bytes()       // → &[u8]
CommitId::from_hex(hex)   // → Result<CommitId, CommitIdFromHexError>
CommitId::from_bytes(vec) // → CommitId
```

Implements `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`,
`Display`.

## `CommitInfo`

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

## `BranchInfo`

```rust
pub struct BranchInfo {
    pub name:                   String,    // e.g. "main"
    pub full_name:              String,    // e.g. "refs/heads/main"
    pub last_commit_id:         CommitId,
    pub last_commit_summary:    String,
    pub last_commit_timestamp:  SystemTime,
}
```

## `StatusDigest`

```rust
pub struct StatusDigest {
    pub repo_name:             String,
    pub current_branch:        String,   // "(detached)" for detached HEAD
    pub last_commit_id:        CommitId,
    pub last_commit_summary:   String,
    pub last_commit_timestamp: SystemTime,
}
```

## `TagInfo`

```rust
pub struct TagInfo {
    pub name:             String,
    pub full_name:        String,
    pub commit_id:        CommitId,     // peeled to commit
    pub commit_summary:   String,
    pub commit_timestamp: SystemTime,
    pub annotation:       Option<TagAnnotation>,  // None for lightweight tags
}
```

## `TagAnnotation`

```rust
pub struct TagAnnotation {
    pub message:           String,
    pub tagger_name:       Option<String>,
    pub tagger_timestamp:  Option<SystemTime>,
}
```

## `DiffSummary`

```rust
pub struct DiffSummary {
    pub added:    Vec<PathBuf>,   // sorted ascending
    pub modified: Vec<PathBuf>,
    pub deleted:  Vec<PathBuf>,
}
```

## `BlameEntry`

```rust
pub struct BlameEntry {
    pub commit_id:     CommitId,
    pub start_line:    u32,       // 1-indexed, inclusive
    pub end_line:      u32,       // 1-indexed, inclusive
    pub original_path: Option<PathBuf>,  // set when file was renamed
}
```

## `WorktreeStatus`

```rust
pub struct WorktreeStatus {
    pub staged:    Vec<StatusEntry>,   // index differs from HEAD
    pub unstaged:  Vec<StatusEntry>,   // working tree differs from index
    pub untracked: Vec<PathBuf>,       // not tracked; gitignore applied
}

pub struct StatusEntry {
    pub path: PathBuf,
    pub kind: ChangeKind,              // Added | Modified | Deleted
}
```

## Other types

`SubmoduleInfo`, `StashEntry`, `WorktreeInfo`, `SortOrder`, `BackendKind`,
`CommitIdFromHexError` — see the crate API docs on docs.rs.
