# Git CLI parity tests

endringer does not invoke `git` at runtime. However, git CLI output is the
practical reference for many behavioural contracts. This page documents why
and how git CLI comparisons are used in tests.

## Why git CLI is used in tests

Some endringer semantics are best defined by reference to git:
- ahead/behind counts match `git rev-list --left-right --count`;
- merge-base matches `git merge-base`;
- is-ancestor matches `git merge-base --is-ancestor`;
- branch and tag listings should match `git for-each-ref`.

Parity tests catch subtle divergences from gix behaviour earlier than
integration tests alone.

## Commands used

All parity tests use machine-readable, locale-independent output:

| endringer API | git command |
|---|---|
| `ahead_behind` | `git rev-list --left-right --count A...B` |
| `merge_base` | `git merge-base A B` |
| `is_ancestor` | `git merge-base --is-ancestor A B` |
| `list_tags` | `git for-each-ref --format=%(refname:short) refs/tags` |
| `local_branches` | `git for-each-ref --format=%(refname:short) refs/heads` |
| `worktree_status` | `git status --porcelain` |
| `blame` | `git blame --line-porcelain` |

## Environment isolation

Parity test commands use the same isolation as all fixture commands:
`GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL=/dev/null`, `GIT_EDITOR=true`,
`GIT_TERMINAL_PROMPT=0`, null stdin.

## Known intentional deviations

See `crates/endringer/tests/parity/KNOWN-DEVIATIONS.md` for a maintained
list. Current deviations:

- `worktree_status()` uses `ChangeKind` (Added/Modified/Deleted) and does not
  distinguish mode changes or type changes from content modifications.
- `rich_worktree_status()` extends ChangeKind but rename/copy detection is
  opt-in and may differ from `git status --find-renames`.
- The `DiffSummary` from `diff()` classifies paths as added/modified/deleted
  without rename/copy information unless `diff_entries(options)` is used.
- jj backend paths are tested only via git-store view; jj-native semantics
  are not compared against git CLI.
