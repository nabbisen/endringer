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

## Working-tree status: a filter that cannot run

`worktree_status()`/`is_dirty()` compare a tracked file's content in its git
form (since 0.33.4) — converted through the repository's filter pipeline
(`core.autocrlf`, `.gitattributes`, clean filters). If a configured external
clean filter's program is missing, the conversion fails and the file is
reported `Modified`, the conservative call also made for any other read
failure. `git status` in the same situation reports the file unchanged if
its raw content still matches what was last staged, or may itself warn and
fall back depending on `filter.<driver>.required`. This is not compared
against git CLI behaviour in the parity suite — the scenario requires a
missing external program, not something the parity fixtures construct.

## Submodule status: `ignore` settings are not honoured

`worktree_status()`/`is_dirty()` (since 0.33.4) reports a submodule
`Modified` whenever its checked-out HEAD differs from the parent's gitlink,
or its own working tree is not clean. `git status` lets
`submodule.<name>.ignore` (in `.gitmodules` or local config) and
`diff.ignoreSubmodules` narrow or suppress this per submodule (`none` /
`untracked` / `dirty` / `all`). endringer always behaves as `ignore=none` —
it does not read either setting.

## Submodule status: a broken submodule fails `git status` outright

When a submodule's gitfile is broken (points at a `.git` directory that
does not exist), `git status` on the superproject fails outright: `fatal:
... gitfile does not point to a valid repository`. endringer (since 0.33.4)
does not fail — it reports that one submodule `Modified` and the rest of
the status normally.

## jj backend

jj repositories are not compared against git CLI except through the
git-store view. jj-native semantics (change IDs, op log) are out of scope.
