# endringer

A Rust library for Git repository introspection, following the UNIX philosophy of doing one thing well.

[![crates.io](https://img.shields.io/crates/v/endringer?label=rust)](https://crates.io/crates/endringer)
[![License](https://img.shields.io/github/license/nabbisen/endringer)](https://github.com/nabbisen/endringer/blob/main/LICENSE)
[![Documentation](https://docs.rs/endringer/badge.svg?version=latest)](https://docs.rs/endringer)
[![Dependency Status](https://deps.rs/crate/endringer/latest/status.svg)](https://deps.rs/crate/endringer)

---

## What it does

`endringer` provides a clean, `gix`-free public API for reading and inspecting a local Git repository:

| Operation | Method |
|---|---|
| Open a repository | `repository(path)` |
| Local branches | `Repository::local_branches()` |
| Remote-tracking branches | `Repository::remote_branches()` |
| Full commit history | `Repository::list_commits()` |
| Commits in a time window | `Repository::log_since(since, until)` |
| Quick HEAD snapshot | `Repository::status_digest()` |
| List tags | `Repository::list_tags()` |
| Create a lightweight tag | `Repository::create_tag(name)` |
| Delete a tag | `Repository::delete_tag(name)` |

All methods return `anyhow::Result<_>` and owned data — no internal locks are held after a call returns.

## What it does not do

Persistence, scheduling, UI, i18n, and application-level configuration are deliberately out of scope.  Callers own those concerns.

---

## Quick start

Add the dependency:

```toml
[dependencies]
endringer = "0.8"
```

```rust
use std::path::Path;
use endringer::repository;

fn main() -> anyhow::Result<()> {
    let repo = repository(Path::new("."))?;

    // Current HEAD snapshot
    let digest = repo.status_digest()?;
    println!("branch : {}", digest.current_branch);
    println!("HEAD   : {} {}", digest.last_commit_id.short(), digest.last_commit_summary);

    // Recent commits (last 7 days)
    use std::time::{Duration, SystemTime};
    let until = SystemTime::now();
    let since = until - Duration::from_secs(7 * 24 * 3600);
    for c in repo.log_since(since, until)? {
        println!("{} {} ({})", c.commit_id.short(), c.summary, c.author);
    }

    // Tags
    for t in repo.list_tags()? {
        println!("tag {} → {}", t.name, t.commit_id.short());
    }

    Ok(())
}
```

---

## Public API

### Types (`endringer::types`)

| Type | Description |
|---|---|
| `CommitId` | Opaque SHA-1 commit identifier; implements `Display` (40-char hex) and `CommitId::short()` (7-char) |
| `BranchInfo` | Name, full ref, tip commit ID, summary, timestamp |
| `CommitInfo` | Commit ID, author, subject line, timestamp |
| `StatusDigest` | Repo name, current branch, HEAD commit ID, summary, timestamp |
| `TagInfo` | Tag name, full ref, target commit ID, commit summary, commit timestamp |

`gix` types are never exposed in the public API.

### `CommitId`

```rust
let short: String = commit_id.short();          // "a1b2c3d"
let full:  String = commit_id.to_string();       // "a1b2c3d4e5f6..."
let short2 = endringer::commit_id_to_short_id(commit_id); // same as .short()
```

---

## Design notes

- **Separation of concerns** — `endringer` is a VCS adapter library.  It reads Git state; it does not write application config, schedule tasks, or own any UI.
- **No `gix` in public API** — `gix::ObjectId` and other `gix` types are fully contained behind `CommitId` and the `pub(crate)` submodules.
- **Single dependency for VCS** — only `gix` and `anyhow` are needed at runtime.
