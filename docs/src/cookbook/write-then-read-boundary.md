# Cookbook: Write-then-read boundary

**When to use this pattern.** You need to perform VCS operations that endringer
does not own (fetch, merge, checkout, resolve conflicts) and then read the
resulting state.

## The pattern

endringer reads; the consumer writes. For any operation outside tag
create/delete, shell out to `git` or `jj`, then call endringer to read the
new state. No invalidation or refresh call is needed — the next read always
sees the current on-disk state.

```rust,no_run
use endringer::repository;
use std::path::Path;
use std::process::Command;

fn fetch_and_read(path: &Path, remote: &str) -> anyhow::Result<()> {
    // 1. Write: shell out to git for the network operation.
    let ok = Command::new("git")
        .args(["fetch", remote])
        .current_dir(path)
        .status()?.success();
    if !ok { anyhow::bail!("git fetch failed"); }

    // 2. Read: endringer sees the fetched state immediately.
    let repo = repository(path)?;
    for branch in repo.local_branch_tracking()? {
        if let Some(ab) = branch.ahead_behind {
            println!("{}: +{} -{}", branch.tracking.name, ab.ahead, ab.behind);
        }
    }
    Ok(())
}
```

## Operations that follow this pattern

| Operation | Shell out to | Then read with |
|---|---|---|
| Fetch | `git fetch` / `jj git fetch` | `local_branch_tracking()` |
| Merge | `git merge` | `operation_state()`, `conflict_summary()` |
| Checkout | `git checkout` / `jj edit` | `status_digest()` |
| Rebase | `git rebase` | `operation_state()` |
| Push | `git push` | `references()` |
| Submodule update | `git submodule update` | `submodule_summaries()` |

## Boundary note

This is the boundary as designed. endringer will never grow fetch, merge,
or checkout — those operations bring credential management, hook execution,
and conflict resolution that contradict the read-first stance. The consumer
owns writes; endringer provides the read view of the resulting state.
