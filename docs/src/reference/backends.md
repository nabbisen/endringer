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

## Custom backends

Implement `endringer::VcsBackend` and inject it via:

```rust
use endringer::repository::Repository;
use endringer::{VcsBackend, BackendKind};

let repo = Repository::with_backend(Box::new(MyBackend), BackendKind::Git);
```

`VcsBackend` is public but **not yet stable** — its method set may change
before v1.0.
