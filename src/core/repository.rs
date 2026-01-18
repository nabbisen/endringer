use anyhow::{Context, Result};
use gix::Repository;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

use crate::types::{BranchInfo, CommitInfo, DagInfo, StatusDigest};

pub fn repository(repo_path: &Path) -> Result<Repository> {
    gix::open(repo_path).context("failed to open git repository")
}
