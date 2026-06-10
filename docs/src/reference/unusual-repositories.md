# Unusual repository states

endringer is designed to handle real-world repositories that are not always
clean, branch-attached working trees. This page documents the exact behaviour
for states that deviate from the default.

## State definitions

| State | Condition |
|---|---|
| **Empty / unborn** | `git init` with no commits; HEAD points to a branch that doesn't exist yet |
| **Detached HEAD** | HEAD points directly to a commit OID, not to a branch |
| **Bare** | Repository without a working tree (`.git/` is the root) |

## Method behaviour matrix

| Method | Empty / unborn | Detached HEAD | Bare |
|---|---|---|---|
| `status_digest()` | `Err` — no commit to read | `Ok` — `current_branch = "(detached)"` | `Ok` — `current_branch = "(detached)"` or branch name |
| `repository_info()` | `Ok` — `head = HeadState::Unborn` | `Ok` — `head = HeadState::Detached { commit_id }` | `Ok` — `capabilities.working_tree = false` |
| `list_commits()` | `Ok(vec![])` or `Err` (HEAD unborn) | `Ok` — walks from detached commit | `Ok` — walks from HEAD |
| `query_commits()` | `Err` when start is `Head` (unborn) | `Ok` | `Ok` |
| `local_branches()` | `Ok(vec![])` | `Ok` | `Ok` |
| `list_tags()` | `Ok(vec![])` | `Ok` | `Ok` |
| `worktree_status()` | `Ok` with empty lists | `Ok` | `Ok(empty)` or `Err` — no working tree |
| `is_dirty()` | `Ok(false)` | `Ok` | `Ok(false)` |
| `references()` | `Ok` — only `HEAD` (Unborn) | `Ok` | `Ok` |
| `remotes()` | `Ok(vec![])` | `Ok` | `Ok` |
| `file_at_commit()` | `Err` (no commit specified) | `Ok` | `Ok` |

## `HeadState` and typed head information

`repository_info().head` returns a typed [`HeadState`](../types.md) value:

- `HeadState::Attached { branch, full_name, commit_id }` — normal attached HEAD
- `HeadState::Detached { commit_id }` — detached HEAD
- `HeadState::Unborn { branch }` — HEAD points to a branch with no commits yet
- `HeadState::Missing` — HEAD file is absent or unreadable (unusual but handled)

For compatibility, `StatusDigest::current_branch` still uses the string
`"(detached)"` for detached HEAD. New code should prefer `repository_info().head`.

## Bare repositories

Bare repositories support all read operations that do not require a working
tree. `worktree_status()` returns an empty `WorktreeStatus` or an error;
`is_dirty()` returns `false`. All other reads (commits, refs, tags, blame,
tree snapshots) work normally.

`RepositoryCapabilities::working_tree` is `false` for bare repositories.

## Empty / unborn repositories

An unborn repository (`git init` with no commits) returns:

- `Ok(vec![])` from `local_branches()` and `list_tags()`
- `Err` from `status_digest()`, `list_commits()` (when start is `Head`),
  and any method that requires a commit to be present
- `repository_info().head = HeadState::Unborn { branch: Some("main") }`

Callers that may encounter unborn repositories should check
`repository_info().head` before calling commit-dependent methods.
