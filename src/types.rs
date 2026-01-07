use std::time::SystemTime;

#[derive(Debug)]
pub struct StatusDigest {
    pub repo_name: String,
    pub current_branch: String,

    pub last_commit_summary: String,
    pub last_commit_time: SystemTime,
}
