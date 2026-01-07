use std::path::Path;

use anyhow::Result;

use crate::{
    core::gix,
    types::{DagInfo, StatusDigest},
};

mod core;
pub mod types;

pub fn status_digest(repo_path: &Path) -> Result<StatusDigest> {
    gix::status_digest(repo_path)
}

pub fn dag(repo_path: &Path) -> Result<DagInfo> {
    gix::dag(repo_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works_status_digest() {
        let result = status_digest(&Path::new("."));
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn it_works_status_dag() {
        let result = dag(&Path::new("."));
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
