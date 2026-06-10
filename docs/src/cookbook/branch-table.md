# Cookbook: Branch table

**When to use this pattern.** You are rendering a branch list with columns for
name, tip commit, upstream sync state (ahead/behind), and merge status.

## API calls

```rust
repo.local_branches()         // names and tip commits
repo.local_branch_tracking()  // upstream + ahead/behind in one call
repo.is_merged_into(b, main)  // whether b is fully merged into main
```

## Minimal example

```rust,no_run
use endringer::repository;
use std::path::Path;

fn branch_table(path: &Path) -> anyhow::Result<()> {
    let repo = repository(path)?;

    for info in repo.local_branch_tracking()? {
        let sync = match (&info.upstream, info.ahead_behind) {
            (None, _) => "no upstream".to_owned(),
            (Some(_), None) => "gone".to_owned(),
            (Some(_), Some(ab)) => format!("+{} -{}", ab.ahead, ab.behind),
        };
        println!(
            "{:<30} {} {}",
            info.tracking.name,
            &info.tracking.last_commit_id.short(),
            sync,
        );
    }
    Ok(())
}
```

## Cost notes

- `local_branch_tracking()` walks the commit graph once per branch to compute
  ahead/behind. For repositories with many branches or deep histories, prefer
  calling it on demand (e.g. when the user expands a branch row) rather than
  eagerly for every branch.
- `local_branches()` is cheap if sync state is not needed.

## Boundary note

The consumer decides which branches to display, how to sort them (beyond
the ascending-by-name default), and when to refresh. Filtering, sorting
overrides, and refresh scheduling are all consumer-owned.
