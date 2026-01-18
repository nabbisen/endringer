use std::path::Path;

use anyhow::Result;
use gix::{ObjectId, Repository};

use crate::{
    core::{branch, commit, repository},
    types::{BranchInfo, CommitInfo, DagInfo, StatusDigest},
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

pub fn list_commits(repository: &Repository) -> Result<Vec<CommitInfo>> {
    branch::list_commits(repository)
}

pub fn status_digest(repository: &Repository) -> Result<StatusDigest> {
    commit::status_digest(&repository)
}

pub fn commit_id_to_short_id(commit_id: ObjectId) -> String {
    commit::commit_id_to_short_id(commit_id)
}

#[deprecated(since = "0.4.0", note = "dag info fns possibly will be removed")]
pub fn dag(repo_path: &Path) -> Result<DagInfo> {
    commit::dag(repo_path)
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
    fn it_works_list_commits() {
        let repository = repository(&Path::new(".")).unwrap();
        let result = list_commits(&repository);
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

    #[test]
    fn it_works_commit_id_to_short_id() {
        let repository = repository(&Path::new(".")).unwrap();
        let status_digest = status_digest(&repository).expect("failed to get status digest");
        let result = commit_id_to_short_id(status_digest.last_commit_id);
        println!("{:?}", result);
        assert!(result.len() == 7);
    }

    #[deprecated(since = "0.4.0", note = "dag info fns possibly will be removed")]
    #[test]
    fn it_works_status_dag() {
        let result = dag(&Path::new("."));
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
