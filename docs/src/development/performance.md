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
| `worktree_status()` | Moderate | Iterates index; content hash fallback per entry |
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

Run `cargo bench -p endringer` to generate your own numbers.
