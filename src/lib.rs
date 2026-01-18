use std::path::Path;

use anyhow::Result;
use gix::Repository;

use crate::{
    core::{branch, repository, status},
    types::{BranchInfo, DagInfo, StatusDigest},
};

mod core;
pub mod types;

pub fn repository(repo_path: &Path) -> Result<Repository> {
    repository::repository(repo_path)
}

pub fn local_branches(repository: &Repository) -> Result<Vec<BranchInfo>> {
    branch::local_branches(repository)
}

pub fn remote_branches(repository: &Repository) -> Result<Vec<BranchInfo>> {
    branch::remote_branches(repository)
}

pub fn status_digest(repository: &Repository) -> Result<StatusDigest> {
    status::status_digest(&repository)
}

#[deprecated(since = "0.4.0", note = "dag info fns possibly will be removed")]
pub fn dag(repo_path: &Path) -> Result<DagInfo> {
    status::dag(repo_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works_repository() {
        let result = repository(&Path::new("."));
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn it_works_local_branches() {
        let repository = repository(&Path::new(".")).unwrap();
        let result = local_branches(&repository);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn it_works_remote_branches() {
        let repository = repository(&Path::new(".")).unwrap();
        let result = remote_branches(&repository);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn it_works_status_digest() {
        let repository = repository(&Path::new(".")).unwrap();
        let result = status_digest(&repository);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[deprecated(since = "0.4.0", note = "dag info fns possibly will be removed")]
    #[test]
    fn it_works_status_dag() {
        let result = dag(&Path::new("."));
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
