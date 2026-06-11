# Known deviations from Git CLI behaviour

This file records intentional differences between endringer's output and
the corresponding `git` CLI output. Deviations must be listed here and
referenced in the affected test.

## Status: ChangeKind simplification

`worktree_status()` returns `ChangeKind::Added | Modified | Deleted`.
Git also reports `R` (rename), `C` (copy), `T` (type change), `U` (unmerged).
These are collapsed:
- `R` → appears as `Deleted` (old path) + `Added` (new path) in simple status
- `T` → appears as `Modified`
- `U` → does not appear in simple status; use `unmerged_paths()` or `conflict_summary()`

The richer `rich_worktree_status()` exposes `FileStatusKind::Renamed`, but
rename detection is opt-in and may differ from `--find-renames` thresholds.

## Tag listing order

`list_tags()` returns tags sorted by name when `SortOrder::ByName` is used.
`git for-each-ref refs/tags` also sorts by refname (which includes
`refs/tags/` prefix). The resulting order matches after stripping the prefix.

## Ahead/behind: unrelated histories

When branches have no common ancestor, `ahead_behind()` returns an error.
`git rev-list --left-right --count` also fails in this case.
Behaviour matches.

## jj backend

jj repositories are not compared against git CLI except through the
git-store view. jj-native semantics (change IDs, op log) are out of scope.
