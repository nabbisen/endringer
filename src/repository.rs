use std::path::Path;

use anyhow::{Context, Result};

use crate::types::{BranchInfo, CommitInfo, StatusDigest};

pub mod branch;
pub mod commit;

#[derive(Clone, Debug)]
pub struct Repository {
    inner: gix::Repository,
}

impl Repository {
    pub fn local_branches(&self) -> Result<Vec<BranchInfo>> {
        branch::local_branches(&self.inner)
    }

    pub fn remote_branches(&self) -> Result<Vec<BranchInfo>> {
        branch::remote_branches(&self.inner)
    }

    pub fn list_commits(&self) -> Result<Vec<CommitInfo>> {
        branch::list_commits(&self.inner)
    }

    pub fn status_digest(&self) -> Result<StatusDigest> {
        commit::status_digest(&self.inner)
    }
}

/// get repository
pub fn repository(repo_path: &Path) -> Result<Repository> {
    let repository = gix::open(repo_path).context("failed to open git repository");
    match repository {
        Ok(x) => Ok(Repository { inner: x }),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use crate::commit_id_to_short_id;

    use super::*;

    #[test]
    fn it_works_repository() {
        let result = repository(&Path::new("."));
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn it_works_local_branches() {
        let repository = repository(&Path::new(".")).expect("failed to get repository");
        let result = repository.local_branches();
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn it_works_remote_branches() {
        let repository = repository(&Path::new(".")).expect("failed to get repository");
        let result = repository.remote_branches();
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn it_works_list_commits() {
        let repository = repository(&Path::new(".")).expect("failed to get repository");
        let result = repository.list_commits();
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn it_works_status_digest() {
        let repository = repository(&Path::new(".")).expect("failed to get repository");
        let result = repository.status_digest();
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn it_works_commit_id_to_short_id() {
        let repository = repository(&Path::new(".")).expect("failed to get repository");
        let status_digest = repository
            .status_digest()
            .expect("failed to get status digest");
        let result = commit_id_to_short_id(status_digest.last_commit_id);
        println!("{:?}", result);
        assert!(result.len() == 7);
    }
}
