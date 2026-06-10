//! Async facade for endringer.
//!
//! All operations delegate to the synchronous [`endringer`] implementation via
//! [`tokio::task::spawn_blocking`], keeping blocking VCS I/O on a dedicated
//! thread pool and freeing the async executor thread.
//!
//! # Example
//!
//! ```no_run
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! use endringer_async::AsyncRepository;
//!
//! let repo = AsyncRepository::open(std::path::Path::new(".")).await?;
//! let digest = repo.status_digest().await?;
//! println!("{} @ {}", digest.current_branch, digest.last_commit_id.short());
//! # Ok(())
//! # }
//! ```

pub mod async_api;
pub use async_api::AsyncRepository;

pub use endringer::{Error, NotFoundKind, Result};
