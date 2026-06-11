# Architecture

## Workspace layout

```
endringer/                        (workspace root)
  Cargo.toml                      workspace manifest
  crates/
    endringer-core/               types + VcsBackend trait + error model
    endringer-git/                git backend (gix)
    endringer-jj/                 jj backend (delegates to git)
    endringer/                    public façade
    endringer-async/              async wrapper
  docs/                           this documentation (mdbook)
  rfcs/                           RFC directory (done/ + proposed/ + archive/)
  scripts/                        release and contract-check scripts
```

### Dependency graph

```
endringer-core
    ↑
endringer-git  ────────────────┐
endringer-jj   → endringer-git
endringer      → endringer-core, endringer-git, endringer-jj
endringer-async → endringer, tokio
```

## endringer-core

Contains `endringer_core::types` (all public types), `endringer_core::backend::VcsBackend`
(the trait all backends implement), and `endringer_core::error` (the `Error`
enum and `Result<T>` alias introduced in v0.23.0).

`VcsBackend` is `pub` to allow downstream custom backends, but is not yet
considered stable API. Methods with default implementations return
`UnsupportedBackendFeature`; custom backends need only override what they
support.

## endringer-git

Implements `VcsBackend` for `GitBackend`. Internal modules by concern:

| Module | Responsibility |
|---|---|
| `backend.rs` | `GitBackend` struct; `VcsBackend` dispatch via `repo!` macro |
| `blame.rs` | `blame_file` wrapper; `blame_at` |
| `branch.rs` | branch listing, commit traversal, `log_since`, `find_commit`, `query_commits` |
| `commit.rs` | `status_digest` |
| `conflict.rs` | index stage reading for `unmerged_paths`, `conflict_summary` |
| `diff.rs` | tree-to-tree file diff; `diff_entries` |
| `graph.rs` | `merge_base`, `is_ancestor`, `ahead_behind` |
| `info.rs` | `repository_info`, `RepositoryCapabilities` |
| `object.rs` | `file_at_commit`, recursive tree traversal |
| `operation.rs` | marker-file detection for `operation_state` |
| `refs.rs` | `references`, `references_by_kind`, `remotes` |
| `stash.rs` | reflog-based stash listing |
| `stash_detail.rs` | stash commit metadata; `stash_diff` |
| `status.rs` | `is_dirty`, `worktree_status`, `rich_worktree_status` |
| `submodule.rs` | `.gitmodules` parsing |
| `submodule_summary.rs` | rich submodule state via `gix::discover` |
| `tag.rs` | tag listing/creation/deletion; annotation extraction |
| `tree.rs` | `tree_at_commit`, `tree_at_path` |
| `util.rs` | gix conversion helpers |
| `worktree.rs` | linked worktree listing |
| `worktree_detail.rs` | rich worktree detail from `.git/worktrees/` |

Each module exposes `pub(crate)` functions called from `backend.rs` via the
`repo!` macro which calls `inner.to_thread_local()`.

## endringer-jj

`JjBackend` holds a `GitBackend` pointing at jj's git object store and
delegates every `VcsBackend` method to it. The only override is `status_digest`,
which corrects `repo_name` to show the project root rather than the store directory.

The only intentional divergence from `GitBackend` is `create_annotated_tag`,
which returns `UnsupportedBackendFeature` because jj has no annotated tag
concept.

## endringer

The public façade. `Repository` holds a `Box<dyn VcsBackend>` and forwards
calls. Provides:
- `repository(path)` and `jj_repository(path)` constructors
- `Repository::with_backend(backend, kind)` for custom backends
- Re-exports of all public types from `endringer-core`

## endringer-async

Wraps `Arc<Repository>` and re-exposes every method as `async fn` using
`tokio::task::spawn_blocking`. Provides `AsyncRepository::open` and
`AsyncRepository::open_jj` constructors.

Every sync method added in a release also gets an async mirror in the same
release. See [async semantics](async-semantics.md) for the cancellation
contract.

## Test structure

```
crates/endringer/
  src/repository/tests.rs         unit tests (use workspace git repo)
  tests/
    support/
      fixture.rs                  shared Fixture helper (#[path] inclusion)
      git_cli.rs                  git CLI parity helpers
      jj_fixture.rs               JjFixture helper for jj integration tests
    git_blame.rs                  per-line blame
    git_branches.rs               branch listing, tracking, ahead/behind
    git_cli_parity.rs             git CLI parity tests (RFC 015)
    git_commits.rs                commit history, graph helpers
    git_core.rs                   constructors, status, SHA-256 validation
    git_detail_reads.rs           submodule summaries, stash detail, worktree detail
    git_diff.rs                   diff between commits
    git_dirty.rs                  is_dirty scenarios
    git_error_model.rs            Error enum, typed error matching
    git_graph.rs                  merge_base, is_ancestor
    git_hardening.rs              security/robustness edge cases
    git_operation_state.rs        operation state, conflict detection
    git_platform.rs               path/platform edge cases
    git_refs.rs                   references, references_by_kind, remotes
    git_repository_info.rs        RepositoryInfo, RepositoryCapabilities
    git_rich_status.rs            RichWorktreeStatus, StatusOptions
    git_snapshot_diff.rs          snapshot batch reads, diff_entries
    git_status.rs                 WorktreeStatus, file_at_commit, gitignore
    git_submodule_stash.rs        submodule listing, stash entries
    git_tags.rs                   tag CRUD + annotation
    git_tree.rs                   tree_at_commit, tree_at_path, blame_at
    git_unusual_repos.rs          unborn, detached HEAD, bare repositories
    git_worktree.rs               linked worktrees
    jj.rs                         jj backend error paths
    jj_real.rs                    jj real-repository integration tests
    parity/
      KNOWN-DEVIATIONS.md         documented CLI parity deviations
    vcsbackend_defaults.rs        VcsBackend default method behaviour

crates/endringer-async/
  tests/async_tests.rs            async integration tests (#[tokio::test])

crates/endringer/
  benches/
    repository_reads.rs           Criterion benchmarks (status/refs/history/object)
```
