# Quick start

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
endringer = "0.33"
anyhow    = "1"
```

For async usage, also add:

```toml
endringer-async = "0.33"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Open a repository

```rust
use std::path::Path;
use endringer::repository::repository;

let repo = repository(Path::new("."))?;
```

`repository()` discovers the git root starting from the given path, so you
can pass any subdirectory of a worktree.

For a Jujutsu repository:

```rust
use endringer::repository::jj_repository;
let repo = jj_repository(Path::new("."))?;
```

## Read the current state

```rust
let d = repo.status_digest()?;
println!("branch : {}", d.current_branch);
println!("HEAD   : {} — {}", d.last_commit_id.short(), d.last_commit_summary);

if repo.is_dirty()? {
    println!("working tree has uncommitted changes");
}
```

## List recent commits

```rust
use std::time::{Duration, SystemTime};

let now   = SystemTime::now();
let since = now - Duration::from_secs(7 * 24 * 3600);

for c in repo.log_since(since, now)? {
    println!("{} {} ({})", c.commit_id.short(), c.summary, c.author);
}
```

## Work with tags

```rust
// List all tags sorted by name
use endringer::SortOrder;
for t in repo.list_tags_sorted(SortOrder::ByName)? {
    print!("{}", t.name);
    if let Some(ann) = &t.annotation {
        print!(" — {}", ann.message);
    }
    println!();
}

// Create and delete
repo.create_tag("v1.0.0")?;
repo.delete_tag("v1.0.0")?;
```

## Diff between two commits

```rust
let commits = repo.list_commits()?;
if commits.len() >= 2 {
    let d = repo.diff(&commits[1].commit_id, &commits[0].commit_id)?;
    println!("added: {:?}", d.added);
    println!("modified: {:?}", d.modified);
    println!("deleted: {:?}", d.deleted);
}
```

## Blame a file

```rust
use std::path::Path;
for entry in repo.blame(Path::new("src/lib.rs"))? {
    println!("lines {}-{}: {}", entry.start_line, entry.end_line,
             entry.commit_id.short());
}
```

## Read file content at a specific commit

```rust
let commits = repo.list_commits()?;
let content = repo.file_at_commit(Path::new("README.md"), &commits[0].commit_id)?;
println!("{}", String::from_utf8_lossy(&content));
```

## Async usage

```rust
use endringer_async::AsyncRepository;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let repo = AsyncRepository::open(std::path::Path::new(".")).await?;
    let digest = repo.status_digest().await?;
    println!("{}", digest.current_branch);
    Ok(())
}
```
