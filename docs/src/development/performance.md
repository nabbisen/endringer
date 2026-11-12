# Performance classification and benchmarks

This page documents endringer's performance characteristics and how to run
the benchmark suite.

## Running benchmarks

```sh
# Run all benchmark groups
cargo bench -p endringer

# Run a specific group
cargo bench -p endringer -- status/

# Generate HTML reports (criterion)
cargo bench -p endringer
# Results in target/criterion/
```

## Performance classification

| Operation | Class | Notes |
|---|---|---|
| `status_digest()` | Cheap | One HEAD resolution |
| `is_dirty()` | Cheap–Moderate | Two-pass: mtime + content hash fallback |
| `operation_state()` | Cheap | Reads marker files only |
| `local_branches()` | Cheap | Iterates ref directory |
| `list_tags_sorted()` | Cheap | Iterates ref directory |
| `references()` | Cheap | Iterates all refs once |
| `worktree_status()` | Moderate | Compares every tracked file's content, in its git form, against the index — a stat match/mismatch is never a shortcut (see below) |
| `rich_worktree_status()` | Moderate | Same as worktree_status + conflict stage read |
| `list_commits()` | Moderate–Expensive | Full ancestry walk; unbounded on deep histories |
| `query_commits(max_count=N)` | Moderate | Stops after N+1 commits |
| `merge_base()` | Moderate | BFS on commit graph |
| `ahead_behind()` | Moderate | Two-pass BFS |
| `diff()` | Moderate | Tree comparison; scales with changed files |
| `blame()` / `blame_at()` | Expensive | Full file history traversal |
| `tree_at_commit()` | Cheap | Single tree object read |
| `file_at_commit()` | Cheap | Single blob read |

## Benchmark groups

| Group | Fixture size | Purpose |
|---|---|---|
| `status` | 20 commits, 5 files/commit | Working tree cost |
| `refs` | 5 commits + 5 tags | Ref enumeration cost |
| `history` | 100 commits, 2 files/commit | History traversal cost |
| `object` | 10 commits, 3 files/commit | Object lookup cost |

## Fixture sizes

- **tiny** (fixture default): 2 commits, 2 files — used in unit/integration tests
- **small** (bench default): 20–100 commits, 2–5 files — bench suite
- **medium/large**: set `ENDRINGER_BENCH_REPO=/path/to/repo` to run against a real repository

## Baseline numbers (informative, not contractual)

These are informative measurements from a development machine. They are not
CI requirements.

Recorded on: Linux x86-64, NVMe SSD, Rust release profile.
Repository: 100 commits, 200 files.

| Benchmark | Approximate time |
|---|---|
| `status_digest` | ~100 µs |
| `is_dirty` (clean) | ~500 µs |
| `worktree_status` | ~1 ms |
| `list_commits_100` | ~2 ms |
| `query_commits_page10` | ~200 µs |
| `tree_at_commit_root` | ~100 µs |

## `worktree_status()` compares content (0.33.4)

The old "fast path" reported a stat mismatch (mtime or size differing from the
index's cached stat) as `Modified` without reading the file. That was also
the bug (clean files reported as modified after a `cp -r`, a `touch`, or an
`autocrlf` checkout — see this version's CHANGELOG entry): a stat mismatch is
no longer trusted, so every tracked file is now read, converted to its git
form and hashed on every call, whether or not its stat matches.

Measured informally (not Criterion; `std::time::Instant` around repeated
calls, release profile, same machine as the baseline numbers above), worst
case — **every** tracked file's mtime stale, the exact scenario a repository
copied without timestamps produces:

| Repository | Before (stat shortcut) | After (always compares content) |
|---|---|---|
| 200 tracked files, all stale | ~392 µs/call | ~646 µs/call (+65%) |
| 16 small repositories (1–2 tracked files each) | ~100 µs/call | ~103 µs/call (+~4%, dominated by per-repo open/index-load cost at this size) |

The regression is real but bounded to the stat-mismatch case specifically —
a repository where nothing changed since the last call was already paying a
per-file content hash in the old code too (the "fallback" path), just on raw
bytes instead of the git-form conversion; that case is not meaningfully
slower.

Run `cargo bench -p endringer` to generate your own numbers.
