# RFC 017 — Performance benchmarking and large-repository profiling

**Status.** Proposed  
**Priority.** P2  
**Target band.** v0.28.x  
**Breaking change.** No  
**Primary area.** Performance / test infrastructure

---

## 1. Summary

Introduce a benchmark and profiling strategy for large repositories so endringer can detect regressions in status, branch listing, history traversal, diff, blame, and file-at-commit operations.

This RFC does not optimize specific code paths directly. It creates the measurement foundation needed before performance-sensitive changes.

---

## 2. Motivation

The library promises ergonomic owned-value reads and lock-free backend access. Those are good defaults, but some calls may still become expensive in large repositories:

- `list_commits()` can traverse a large history;
- `worktree_status()` can scan many files;
- `blame()` can be expensive on large files;
- branch and tag enumeration can scale with ref count;
- `diff()` can produce large path lists.

Without reproducible benchmarks, the project cannot tell whether a gix upgrade, status fix, or new metadata field caused a real regression.

---

## 3. Goals

- Define benchmark scenarios for small, medium, and large repositories.
- Add Criterion-based micro/operation benchmarks if acceptable for dev dependencies.
- Add non-Criterion smoke performance tests that can run in CI with loose thresholds.
- Document which operations are expected to be cheap, moderate, or potentially expensive.
- Provide a repeatable local profiling recipe.

---

## 4. Non-goals

- Do not add built-in caching.
- Do not promise strict latency budgets for all repositories.
- Do not make benchmark fixtures huge in the normal source checkout.
- Do not block CI on brittle wall-clock thresholds unless carefully isolated.

---

## 5. External design

### 5.1 Benchmark groups

Create `benches/` or an `xtask bench-fixtures` workflow with these groups:

| Group | Operation | Purpose |
|---|---|---|
| status | `status_digest`, `is_dirty`, `worktree_status` | Working tree cost |
| refs | `local_branches`, `remote_branches`, `list_tags` | Ref enumeration cost |
| history | `list_commits`, bounded query from RFC 012 | History traversal cost |
| graph | `merge_base`, `is_ancestor`, `ahead_behind` | Commit graph cost |
| object | `file_at_commit`, tree snapshot from RFC 010 | Object lookup cost |
| blame | `blame`, `blame_at` | Expensive file history cost |

### 5.2 Fixture sizes

Use generated repositories rather than checking in large binary fixtures:

- **tiny**: 5 commits, 3 files;
- **small**: 100 commits, 100 files;
- **medium**: 2,000 commits, 5,000 files;
- **large-local**: optional developer-only fixture, not run in normal CI.

The large-local fixture may be created by an ignored script or by pointing the benchmark at an existing local repository through an environment variable.

### 5.3 Performance classification

Document operations using a simple classification:

| Class | Meaning |
|---|---|
| Cheap | suitable for frequent status-widget refresh |
| Moderate | suitable for user-triggered UI refresh |
| Expensive | should be bounded, paged, or triggered intentionally |

Do not overpromise exact durations.

---

## 6. Internal design

### 6.1 Fixture builder

Add a test utility that can create deterministic repositories:

```rust
pub struct RepoFixtureSpec {
    pub commits: usize,
    pub files: usize,
    pub branches: usize,
    pub tags: usize,
    pub dirty_files: usize,
}
```

The fixture builder may use the `git` CLI in tests/benches only, consistent with the existing test strategy.

### 6.2 Benchmark isolation

Benchmarks should disable host git config as the current fixture tests do. They should also avoid network, hooks, prompts, and signing.

### 6.3 Regression reports

Add a `PERF_NOTES.md` or mdBook page summarizing baseline numbers from a maintainer machine, marked as informative rather than contractual.

---

## 7. Tests and verification

- Benchmarks compile under `cargo bench`.
- CI runs only small smoke performance tests, not heavyweight benches.
- Fixture generation is deterministic.
- A gix upgrade can be benchmarked before and after.
- `worktree_status()` is tested with same-second same-size edits to preserve current correctness behavior.

---

## 8. Rollout plan

1. Add fixture builder for benchmark repositories.
2. Add tiny/small Criterion benchmarks.
3. Add documentation for local large-repo benchmarking.
4. Optionally add CI smoke checks with wide thresholds or no thresholds.

---

## 9. Risks and mitigations

**Risk: benchmarks become flaky.** Keep CI thresholds loose or informational.

**Risk: fixture generation is slower than benchmarks.** Cache fixtures within benchmark runs.

**Risk: performance work creates pressure for internal caching.** Caching remains a non-goal unless a future RFC reopens the boundary deliberately.

---

## 10. Definition of done

- Benchmark groups exist for status, refs, history, graph, object, and blame operations.
- Fixture builder is reusable by tests.
- Performance documentation exists.
- Normal CI remains fast and reliable.
