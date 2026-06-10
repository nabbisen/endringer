# Cookbook: Status widget

**When to use this pattern.** You are building a VCS status bar, tray icon,
or IDE panel that needs to refresh repository state on a timer or on file
change events.

## API calls

```rust
repo.status_digest()     // branch name, HEAD commit, repo name
repo.is_dirty()          // single clean/dirty boolean
repo.worktree_status()   // staged / unstaged / untracked file lists
repo.operation_state()   // None / Merge / Rebase / CherryPick / ...
```

## Minimal example

```rust,no_run
use endringer::{repository, OperationState};
use std::path::Path;

fn refresh(path: &Path) -> anyhow::Result<()> {
    let repo = repository(path)?;

    let digest = repo.status_digest()?;
    println!("branch: {}", digest.current_branch);
    println!("HEAD:   {} {}", digest.last_commit_id.short(), digest.last_commit_summary);
    println!("dirty:  {}", repo.is_dirty()?);

    match repo.operation_state()? {
        OperationState::None => {}
        OperationState::Merge { .. }      => println!("MERGING"),
        OperationState::Rebase { .. }     => println!("REBASING"),
        OperationState::CherryPick { .. } => println!("CHERRY-PICKING"),
        OperationState::Revert { .. }     => println!("REVERTING"),
        OperationState::Bisect            => println!("BISECTING"),
    }
    Ok(())
}
```

## Cost notes

- `status_digest()` — cheap: one HEAD resolution.
- `is_dirty()` — two-pass heuristic (mtime + content hash fallback). Fast on
  most repos; can be slow on very large working trees.
- `worktree_status()` — iterates the full index. Avoid calling on every
  keypress; call on a throttled timer or file-watcher event.
- `operation_state()` — reads marker files only; always cheap.

## Boundary note

The consumer owns the refresh timer and decides when to call. endringer
holds no state between calls — each read sees the current on-disk reality
with no invalidation needed.

## Error handling

```rust,no_run
use endringer::{repository, Error};

match repository(path) {
    Err(Error::NotARepository { .. }) => show_setup_prompt(),
    Err(e) => log::error!("repo open failed: {e}"),
    Ok(repo) => { /* refresh ... */ }
}
```
