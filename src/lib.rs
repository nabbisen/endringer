use std::path::Path;

use ::gix::Repository;
use anyhow::Result;

use crate::{core::gix, types::StatusDigest};

mod core;
pub mod types;

pub fn repo(repo_path: &Path) -> Result<Repository> {
    gix::repo(repo_path)
}

pub fn status_digest(repo: &Repository) -> Result<StatusDigest> {
    gix::status_digest(repo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works_repo() {
        let result = repo(Path::new("."));
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn it_works_status_digest() {
        let repo = repo(Path::new(".")).expect("failed to get repo");
        let result = status_digest(&repo);
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
