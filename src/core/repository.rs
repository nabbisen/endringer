use anyhow::{Context, Result};
use gix::Repository;
use std::path::Path;

pub fn repository(repo_path: &Path) -> Result<Repository> {
    gix::open(repo_path).context("failed to open git repository")
}
