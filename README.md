# endringer

[![crates.io](https://img.shields.io/crates/v/endringer?label=rust)](https://crates.io/crates/endringer)
[![License](https://img.shields.io/github/license/nabbisen/endringer)](https://github.com/nabbisen/endringer/blob/main/LICENSE)
[![Documentation](https://docs.rs/endringer/badge.svg?version=latest)](https://docs.rs/endringer)
[![Dependency Status](https://deps.rs/crate/endringer/latest/status.svg)](https://deps.rs/crate/endringer)

**Lightweight VCS introspection for Rust — Git and `jj` (Jujutsu), no binaries required.**

---

## Overview

`endringer` reads local Git and `jj` (Jujutsu) repositories, powered by the excellent [`gix`](https://github.com/Byron/gitoxide) crate.
It exposes an ergonomic API — no internal types leak into your code.
All results are owned values; no internal state is held between calls.

```toml
[dependencies]
endringer = "0.33"
```

---

## Why endringer?

| Situation | endringer fits when… |
|---|---|
| Status bar / VCS widget | You need branch, HEAD, and dirty-state on every frame |
| Release tooling | You iterate tags, commits, and diffs without shelling out |
| Jujutsu support | You open `.jj/` repos with the same API as Git |
| Async application | `endringer-async` wraps everything in `spawn_blocking` |

---

## Quick start

```rust
use std::path::Path;
use endringer::repository::repository;

fn main() -> anyhow::Result<()> {
    let repo = repository(Path::new("."))?;

    // HEAD snapshot
    let d = repo.status_digest()?;
    println!("{} @ {} — {}", d.current_branch, d.last_commit_id.short(), d.last_commit_summary);

    // Dirty check
    if repo.is_dirty()? {
        println!("working tree has uncommitted changes");
    }

    // Recent commits
    use std::time::{Duration, SystemTime};
    let now = SystemTime::now();
    for c in repo.log_since(now - Duration::from_secs(7 * 24 * 3600), now)? {
        println!("  {} {}", c.commit_id.short(), c.summary);
    }

    Ok(())
}
```

**`jj` (Jujutsu)** (no binary needed):

```rust
use endringer::repository::jj_repository;
let repo = jj_repository(Path::new("."))?;
```

**Async** (add `endringer-async` to your dependencies):

```rust
use endringer_async::AsyncRepository;
let repo = AsyncRepository::open(Path::new(".")).await?;
let commits = repo.list_commits().await?;
```

---

## Design notes

- **`gix` stays internal** — `gix` types never appear in the public API.  
  Your crate does not need to depend on `gix`.
- **Read-oriented** — only lightweight tag writes are in scope.  
  Commit, merge, and push are out of scope by design.
- **Workspace crates** — the implementation is split so you can depend on just what you need:  
  `endringer-core` (types), `endringer-git`, `endringer-jj`, `endringer-async`.
- **No binaries** — both the Git and Jujutsu backends use `gix` directly;  
  `git` and `jj` do not need to be on `$PATH`.

---

## For more detail

→ **[Full documentation](docs/src/SUMMARY.md)**

Key sections:
- [Getting started & tutorial](docs/src/getting-started/quickstart.md)
- [API overview](docs/src/reference/api-overview.md)
- [Architecture & contributing](docs/src/development/architecture.md)
