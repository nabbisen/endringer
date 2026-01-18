use std::path::Path;

use anyhow::Result;
use gix::ObjectId;

use crate::repository::Repository;

mod repository;
pub mod types;
mod util;

/// get repository
pub fn repository(repo_path: &Path) -> Result<Repository> {
    repository::repository(repo_path)
}

/// convert commit id to short string id
pub fn commit_id_to_short_id(commit_id: ObjectId) -> String {
    util::commit_id_to_short_id(commit_id)
}
