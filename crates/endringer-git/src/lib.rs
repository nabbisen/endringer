//! Git backend for endringer, powered by [`gix`].
//!
//! Exposes [`GitBackend`], which implements
//! [`endringer_core::backend::VcsBackend`].

pub(crate) mod branch;
pub(crate) mod commit;
pub(crate) mod diff;
pub(crate) mod tag;
pub(crate) mod status;
pub(crate) mod util;

mod backend;
pub use backend::GitBackend;

#[cfg(test)]
mod tests;
