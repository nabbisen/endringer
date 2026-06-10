# Cookbook: Async multi-repo scan

**When to use this pattern.** You need to query many repositories concurrently
— a dashboard showing status for an entire workspace of repos, or a CI tool
scanning dozens of directories in parallel.

## API calls

```rust
endringer_async::AsyncRepository::open(path)
AsyncRepository::status_digest()
AsyncRepository::is_dirty()
```

## Minimal example

```rust,no_run
use endringer_async::AsyncRepository;
use std::path::PathBuf;
use tokio::sync::Semaphore;
use std::sync::Arc;

async fn scan_all(paths: Vec<PathBuf>) -> Vec<(PathBuf, String)> {
    // Bound concurrent blocking-pool threads to avoid overwhelming disk IO.
    let sem = Arc::new(Semaphore::new(8));

    let handles: Vec<_> = paths.into_iter().map(|path| {
        let sem = Arc::clone(&sem);
        tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let repo = AsyncRepository::open(&path).await?;
            let digest = repo.status_digest().await?;
            Ok::<_, anyhow::Error>((path, digest.current_branch))
        })
    }).collect();

    let mut results = Vec::new();
    for h in handles {
        if let Ok(Ok(item)) = h.await { results.push(item); }
    }
    results
}
```

## Cost notes

- `AsyncRepository` wraps `spawn_blocking`; each method dispatches to the
  blocking thread pool. The semaphore bounds your resource use — it is not
  working around library contention, since `GitBackend` is lock-free.
- `AsyncRepository` is cheap to clone and share across tasks.

## Boundary note

Concurrency policy (max parallel reads, prioritisation, cancellation, retry)
is consumer-owned. endringer provides thread-safe, lock-free reads; the
consumer decides how many to run at once.
