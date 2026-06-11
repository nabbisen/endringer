# Platform and path robustness matrix

This page documents how endringer behaves across platforms and path edge
cases. It is derived from the integration test suite and updated as new
cases are verified.

## Platform support

| Platform | Status | Notes |
|---|---|---|
| Linux (x86-64) | Supported, CI-tested | Primary development target |
| macOS (arm64/x86-64) | Supported | Symlinks, executable bits tested |
| Windows | Best-effort | Path separators normalised internally; non-UTF-8 limited |

## Path edge case matrix

| Case | Expected behaviour | Test coverage |
|---|---|---|
| ASCII paths | `PathBuf` preserved | All integration tests |
| Paths with spaces | `PathBuf` preserved | `git_platform.rs` |
| Unicode (UTF-8) paths | `PathBuf` preserved | `git_platform.rs` |
| Non-UTF-8 paths (Unix) | `PathBuf` returned without panic; may not round-trip as `str` | `git_platform.rs` (Unix only) |
| Symlinks in tree | `TreeEntryKind::Symlink` reported; target bytes not loaded | `git_platform.rs` (Unix only) |
| Executable bit | `TreeEntry::executable = true` on Unix; false on Windows | `git_platform.rs` |
| Bare repository | All ref/commit reads work; worktree reads empty or error | `git_unusual_repos.rs` |
| Linked worktrees | Listed by `worktrees()` / `worktree_details()` | `git_worktree.rs` |
| Submodules | Listed by `submodules()` / `submodule_summaries()` | `git_submodule_stash.rs` |
| Nested git repositories | Inner repo not traversed automatically; inner `.git/` is an untracked dir from outer's perspective | `git_platform.rs` |

## Path format contract

endringer returns `PathBuf` values that are:

- **Repository-root-relative** for most reads (`StatusEntry::path`, `TreeEntry::path`, `DiffEntry::new_path`, etc.).
- **Absolute** for repository and worktree locations (`RepositoryInfo::path`, `WorktreeDetail::path`).
- **Forward-slash-normalised** internally for file lookup (`file_at_commit`, `tree_at_path`). Callers pass `src/lib.rs`, not `src\lib.rs` on Windows.

Path values returned to callers are `PathBuf` and use the OS separator.

## Non-UTF-8 paths

On Linux (and some macOS configurations), file paths may not be valid UTF-8.
endringer stores paths as `PathBuf` and does not panic on non-UTF-8 path
bytes. APIs that need to pass paths to gix convert via `OsStr::as_encoded_bytes()`
where supported.

Non-UTF-8 paths in `file_at_commit` and `tree_at_path` are not currently
supported — those APIs accept `&Path` and require a convertible path. Use
`tree_at_commit` to enumerate entries and inspect `TreeEntry::path` first.

## Case sensitivity

endringer makes no assumptions about filesystem case sensitivity. Tests
avoid assertions that depend on case-folding. Repositories checked out on
case-insensitive filesystems (Windows, macOS with default HFS+) may behave
differently from Linux; endringer delegates to the underlying filesystem.

## Windows path separators

Internally, endringer normalises forward slashes for object-store lookups.
Returned `PathBuf` values use the OS separator (`\` on Windows). Consumers
that store paths for later lookup must either store `PathBuf` directly or
normalise to forward slashes before passing back to endringer APIs.
