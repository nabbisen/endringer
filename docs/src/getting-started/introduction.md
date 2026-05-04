# What is endringer?

`endringer` lets Rust applications inspect local Git and Jujutsu repositories
without shelling out to `git` or `jj`. It is designed for tools that need
VCS information — status bars, release scripts, code review UIs, CI helpers —
but do not want to manage process spawning, output parsing, or binary
availability.

## Key properties

**No gix in your public API.** `gix` types (`ObjectId`, `Repository`, …) are
fully contained inside endringer's internals. Your crate does not need a
`gix` dependency.

**Owned results.** Every method returns owned data. No borrows cross the API
boundary; no internal state is held between calls.

**Both Git and Jujutsu.** The same `Repository` type handles both backends.
Jujutsu support reads the underlying git object store directly — no `jj`
binary required.

**Optional async.** Add `endringer-async` to your dependencies when you need
to call VCS methods from async code.

## Workspace crates

| Crate | Purpose |
|---|---|
| `endringer` | Main façade — the crate most users depend on |
| `endringer-core` | Types and `VcsBackend` trait only |
| `endringer-git` | Git backend (depends on `gix`) |
| `endringer-jj` | Jujutsu backend (delegates to `endringer-git`) |
| `endringer-async` | Async wrapper via `tokio::task::spawn_blocking` |
