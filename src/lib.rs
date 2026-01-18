use std::path::Path;

use anyhow::Result;

use crate::{
    core::{branch, status},
    types::{BranchInfo, DagInfo, StatusDigest},
};

mod core;
pub mod types;

pub fn local_branches(repo_path: &Path) -> Result<Vec<BranchInfo>> {
    branch::local_branches(repo_path)
}

pub fn remote_branches(repo_path: &Path) -> Result<Vec<BranchInfo>> {
    branch::remote_branches(repo_path)
}

pub fn status_digest(repo_path: &Path) -> Result<StatusDigest> {
    status::status_digest(&repo_path)
}

#[deprecated(since = "0.4.0", note = "dag info fns possibly will be removed")]
pub fn dag(repo_path: &Path) -> Result<DagInfo> {
    status::dag(repo_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works_local_branches() {
        let result = local_branches(&Path::new("."));
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn it_works_remote_branches() {
        let result = remote_branches(&Path::new("."));
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn it_works_status_digest() {
        let result = status_digest(&Path::new("."));
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
