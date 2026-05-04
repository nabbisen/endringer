# Architecture

## Workspace layout

```
endringer/                        (workspace root)
  Cargo.toml                      workspace manifest
  crates/
    endringer-core/               types + VcsBackend trait
    endringer-git/                git backend (gix)
    endringer-jj/                 jj backend (delegates to git)
    endringer/                    public façade
    endringer-async/              async wrapper
  docs/                           this documentation (mdbook)
```

### Dependency graph

```
endringer-core
    ↑
endringer-git  ──────────────┐
endringer-jj   → endringer-git
endringer      → endringer-core, endringer-git, endringer-jj
endringer-async → endringer, tokio
```

## endringer-core

Contains `endringer_core::types` (all public types) and `endringer_core::backend::VcsBackend`
(the trait all backends implement).

`VcsBackend` is `pub` to allow downstream custom backends, but is not yet
considered stable API.

## endringer-git

Implements `VcsBackend` for `GitBackend`. Internal modules by concern:

| Module | Responsibility |
|---|---|
| `backend.rs` | `GitBackend` struct; `VcsBackend` dispatch |
| `branch.rs` | branch listing, commit traversal, `log_since`, `find_commit` |
| `commit.rs` | `status_digest` |
| `tag.rs` | tag listing/creation/deletion; annotation extraction |
| `diff.rs` | tree-to-tree file diff |
| `blame.rs` | `blame_file` wrapper |
| `graph.rs` | `merge_base`, `is_ancestor` |
| `status.rs` | `is_dirty`, `worktree_status` |
| `object.rs` | `file_at_commit`, recursive tree traversal |
| `submodule.rs` | `.gitmodules` parsing |
| `stash.rs` | reflog-based stash listing |
| `worktree.rs` | linked worktree listing |
| `util.rs` | gix conversion helpers |

Each module has a corresponding `pub(crate)` function called from `backend.rs`
via the `repo!` macro which calls `inner.to_thread_local()`.

## endringer-jj

`JjBackend` holds a `GitBackend` pointing at jj's git object store and
delegates every `VcsBackend` method to it. The only override is `status_digest`,
which corrects `repo_name` to show the project root rather than the store directory.

## endringer

The public façade. `Repository` holds a `Box<dyn VcsBackend>` and forwards
calls. Provides:
- `repository(path)` and `jj_repository(path)` constructors
- `Repository::with_backend(backend, kind)` for custom backends
- Re-exports of all public types from `endringer-core`

## endringer-async

Wraps `Arc<Repository>` and re-exposes every method as `async fn` using
`tokio::task::spawn_blocking`. Provides `AsyncRepository::open` and
`open_jj` constructors.

## Test structure

```
crates/endringer/
  src/repository/tests.rs       unit tests (use workspace git repo)
  tests/
    support/fixture.rs          shared Fixture helper (#[path] inclusion)
    git_core.rs                 constructors, status, remote URL
    git_branches.rs             branch listing
    git_commits.rs              commit history, graph helpers
    git_tags.rs                 tag CRUD + annotation
    git_diff.rs                 diff between commits
    git_dirty.rs                is_dirty scenarios
    git_blame.rs                per-line blame
    git_status.rs               WorktreeStatus, file_at_commit, gitignore
    git_submodule_stash.rs      submodule listing, stash entries
    git_worktree.rs             linked worktrees
    jj.rs                       jj backend error paths

crates/endringer-async/
  tests/async_tests.rs          #[tokio::test] async integration tests
```
