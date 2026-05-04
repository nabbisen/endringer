use std::time::SystemTime;

/// Opaque commit identifier, stored as raw bytes.
///
/// Supports both SHA-1 (20 bytes / 40 hex chars, used by Git) and SHA-256
/// (32 bytes / 64 hex chars, used by Jujutsu). No VCS library types are
/// exposed.
///
/// # Ordering
///
/// `CommitId` implements `Ord` via byte-level lexicographic comparison.
/// IDs produced by different hash algorithms (SHA-1 vs SHA-256) compare
/// consistently but not meaningfully across algorithms.
///
/// # Example
///
/// ```
/// # use endringer_core::types::CommitId;
/// let id = CommitId::from_hex("0000000000000000000000000000000000000000").unwrap();
/// assert_eq!(id.short().len(), 7);
/// println!("{id}");   // full hex string
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CommitId(Vec<u8>);

impl CommitId {
    /// Constructs a `CommitId` from raw bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        CommitId(bytes)
    }

    /// Constructs a `CommitId` by decoding a lowercase hex string.
    ///
    /// Accepts 40-character (SHA-1) or 64-character (SHA-256) hex strings.
    ///
    /// ```
    /// # use endringer_core::types::CommitId;
    /// assert!(CommitId::from_hex("0000000000000000000000000000000000000000").is_ok());
    /// assert!(CommitId::from_hex("not-a-hash").is_err());
    /// assert!(CommitId::from_hex("abc123").is_err());  // too short
    /// ```
    pub fn from_hex(hex: &str) -> Result<Self, CommitIdFromHexError> {
        let len = hex.len();
        if len != 40 && len != 64 {
            return Err(CommitIdFromHexError(hex.to_owned()));
        }
        let mut bytes = Vec::with_capacity(len / 2);
        for chunk in hex.as_bytes().chunks(2) {
            let hi = hex_nibble(chunk[0]).ok_or_else(|| CommitIdFromHexError(hex.to_owned()))?;
            let lo = hex_nibble(chunk[1]).ok_or_else(|| CommitIdFromHexError(hex.to_owned()))?;
            bytes.push((hi << 4) | lo);
        }
        Ok(CommitId(bytes))
    }

    /// Returns the raw bytes of this commit identifier.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns the first 7 hex characters — the conventional "short" form.
    pub fn short(&self) -> String {
        let mut out = String::with_capacity(7);
        for &b in self.0.iter().take(4) {
            out.push(nibble_char(b >> 4));
            out.push(nibble_char(b & 0xf));
        }
        out.truncate(7);
        out
    }
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn nibble_char(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        _ => (b'a' + n - 10) as char,
    }
}

impl std::fmt::Display for CommitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for &b in &self.0 {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

/// Error returned when [`CommitId::from_hex`] receives an invalid hex string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitIdFromHexError(String);

impl std::fmt::Display for CommitIdFromHexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid commit id {:?}: expected 40 (SHA-1) or 64 (SHA-256) hex chars",
            self.0
        )
    }
}

impl std::error::Error for CommitIdFromHexError {}

/// Information about a branch (local or remote).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchInfo {
    /// Short branch name, e.g. `main`.
    pub name: String,
    /// Full ref name, e.g. `refs/heads/main` or `refs/remotes/origin/main`.
    pub full_name: String,
    /// Commit ID at the tip of the branch.
    pub last_commit_id: CommitId,
    /// First line of the most recent commit message.
    pub last_commit_summary: String,
    /// Author timestamp of the most recent commit.
    pub last_commit_timestamp: SystemTime,
}

/// Lightweight summary of the repository's current state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusDigest {
    /// Directory name of the repository's working tree.
    pub repo_name: String,
    /// Name of the currently checked-out branch, or `"(detached)"`.
    pub current_branch: String,
    /// Commit ID of the current HEAD.
    pub last_commit_id: CommitId,
    /// First line of HEAD's commit message.
    pub last_commit_summary: String,
    /// Author timestamp of HEAD.
    pub last_commit_timestamp: SystemTime,
}

/// Information about a single commit.
///
/// **Breaking change (v0.14)**: a `parents` field was added. Code that
/// constructs `CommitInfo` directly (outside this library) must be updated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitInfo {
    /// Full commit identifier.
    pub commit_id: CommitId,
    /// Direct parent commit IDs (empty for the initial commit).
    pub parents: Vec<CommitId>,
    /// Author name.
    pub author: String,
    /// Committer name. Differs from `author` after cherry-pick, rebase, or amend.
    pub committer: String,
    /// First line of the commit message (subject line).
    pub summary: String,
    /// Author timestamp.
    pub timestamp: SystemTime,
    /// Committer timestamp.
    pub committer_timestamp: SystemTime,
}

/// Information about a tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagInfo {
    /// Short tag name, e.g. `v1.0.0`.
    pub name: String,
    /// Full ref name, e.g. `refs/tags/v1.0.0`.
    pub full_name: String,
    /// Commit ID the tag points to (after peeling any tag objects).
    pub commit_id: CommitId,
    /// First line of the tagged commit's message.
    pub commit_summary: String,
    /// Author timestamp of the tagged commit.
    pub commit_timestamp: SystemTime,
}

/// Sort order for commit and tag listings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortOrder {
    /// Newest first (descending timestamp).
    NewestFirst,
    /// Oldest first (ascending timestamp).
    OldestFirst,
    /// Alphabetical by tag name or commit summary (ascending).
    ByName,
}

/// Summary of file-level changes between two commits.
///
/// Paths within each category (`added`, `modified`, `deleted`) are sorted
/// in ascending lexicographic order.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct DiffSummary {
    /// Paths of files added between `from` and `to`.
    pub added: Vec<std::path::PathBuf>,
    /// Paths of files modified between `from` and `to`.
    pub modified: Vec<std::path::PathBuf>,
    /// Paths of files deleted between `from` and `to`.
    pub deleted: Vec<std::path::PathBuf>,
}

/// Which VCS backend a [`Repository`][crate] is backed by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendKind {
    /// Git (via `gix`).
    Git,
    /// Jujutsu (git store read via `gix`).
    Jj,
}
