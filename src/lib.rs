use std::path::Path;

use anyhow::Result;

use crate::{core::gix, types::StatusDigest};

mod core;
pub mod types;

pub fn status_digest(repo_path: &Path) -> Result<StatusDigest> {
    gix::status_digest(repo_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = status_digest(Path::new("."));
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
