# Backends: Git and Jujutsu

## Git backend

`repository(path)` opens a Git repository via `gix::discover`, which traverses
parent directories from `path` until it finds a `.git/` directory or a bare
repository.

The `GitBackend` struct wraps a `gix::ThreadSafeRepository`. Each method call
calls `to_thread_local()` to obtain a thread-local `gix::Repository` view —
zero-copy, no mutex, safe for concurrent async use.

## Jujutsu backend

`jj_repository(path)` opens a Jujutsu repository by locating the underlying
git object store.

Jujutsu uses git objects internally. endringer opens that store directly:

| Layout | Detection | Git store |
|---|---|---|
| Co-located | `.git/` and `.jj/` both present | project root |
| Native jj | only `.jj/` present | `.jj/repo/store/git/` |

All read operations are identical to the Git backend. The only behavioural
difference is `create_annotated_tag`, which returns an error on jj repositories
because jj does not support annotated tags — use `create_tag` instead.

### Jujutsu support level (git-store view)

**What works:** endringer reads jj repositories through the underlying git
object store. The following are fully supported:

- commit objects and parent chains;
- refs (branches, tags, remote-tracking refs as written by jj);
- tree objects and `file_at_commit`;
- lightweight tag create/delete;
- `status_digest`, `list_commits`, `find_commit`, `diff`, `blame`;
- `repository_info` (reports `BackendKind::Jj` and the project root as
  `repo_name`, `.jj/` as `vcs_dir`).

**What is not surfaced:** the following jj-native concepts are intentionally
absent from this API:

- jj **change IDs** (distinct from commit IDs);
- the jj **operation log**;
- the jj **working-copy commit** (a special ref written by jj);
- **first-class conflict objects** (conflicts stored inside commits).

This is the documented "git-store view" stance. Adding jj-native concepts
(change IDs, operation log) would require verifying their storage format
stability and is deferred to a future release.

**Verified jj version:** tests are written against jj ≥ 0.18. The CI
job runs `jj --version` and records the result. If you observe failures
with a newer version, please open an issue.

**Runtime independence:** the `jj` binary is **never invoked at runtime**.
It is only used in tests.

**Running jj tests locally:**

```sh
# Skip if jj is not installed (default):
cargo test -p endringer --test jj_real

# Fail if jj is missing (CI mode):
ENDRINGER_REQUIRE_JJ_CLI_TESTS=1 cargo test -p endringer --test jj_real
```

## Custom backends

Implement `endringer::VcsBackend` and inject it via:

```rust
use endringer::repository::Repository;
use endringer::{VcsBackend, BackendKind};

let repo = Repository::with_backend(Box::new(MyBackend), BackendKind::Git);
```

`VcsBackend` is public but **not yet stable** — its method set may change
before v1.0.
