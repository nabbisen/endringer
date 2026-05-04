//! Jujutsu (jj) backend — reads jj repositories via gix.
//!
//! Jujutsu stores its commit history in a git object database. This
//! implementation opens that git store directly with gix; the `jj` binary is
//! **not** required.
//!
//! # Repository layout
//!
//! | Mode | Condition | Git store opened |
//! |------|-----------|-----------------|
//! | Co-located | `.git/` **and** `.jj/` both present | The project root (same as a plain Git repository) |
//! | Native jj  | Only `.jj/` present | `.jj/repo/store/git/` (bare repository) |
//!
//! # Annotated tags
//!
//! Jujutsu itself only supports lightweight tags. `create_annotated_tag`
//! creates a lightweight tag and ignores the message argument. This matches
//! jj's own behaviour and avoids silent data loss.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Result, bail};

use crate::{
    backend::VcsBackend,
    git::GitBackend,
    types::{BranchInfo, CommitId, CommitInfo, DiffSummary, SortOrder, StatusDigest, TagInfo},
};

/// Jujutsu backend backed by the repository's underlying git object store.
///
/// All operations delegate to a [`GitBackend`] opened on the jj git store.
pub(crate) struct JjBackend {
    git: GitBackend,
    /// The user-visible project root (the directory that contains `.jj/`).
    ///
    /// Stored separately so that `status_digest` can report the correct
    /// `repo_name` even when the git store lives under `.jj/repo/store/git`.
    root: PathBuf,
}

impl JjBackend {
    /// Opens a Jujutsu repository at `path`.
    ///
    /// Verifies that `path` contains a `.jj/` directory, then locates and
    /// opens the underlying git object store with gix. The `jj` binary is
    /// not consulted.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `path` does not contain a `.jj/` directory.
    /// - No git backend can be found (neither `.git/` nor `.jj/repo/store/git/`).
    /// - The git backend cannot be opened by gix.
    pub(crate) fn open(path: &Path) -> Result<Self> {
        let root = path
            .canonicalize()
            .unwrap_or_else(|_| path.to_path_buf());

        let jj_dir = root.join(".jj");
        if !jj_dir.is_dir() {
            bail!(
                "not a jj repository: no .jj directory at {}",
                root.display()
            );
        }

        // Resolve the git store path.
        //   1. Co-located: project root has .git/ → open the root itself.
        //   2. Native jj:  git store lives at .jj/repo/store/git/ (bare repo).
        let git_store_path = if root.join(".git").exists() {
            root.clone()
        } else {
            let native = jj_dir.join("repo").join("store").join("git");
            if !native.is_dir() {
                bail!(
                    "jj repository at {} has no git backend \
                     (looked for {} and {})",
                    root.display(),
                    root.join(".git").display(),
                    native.display()
                );
            }
            native
        };

        let git = GitBackend::open(&git_store_path)?;
        Ok(JjBackend { git, root })
    }
}

// ── VcsBackend impl ──────────────────────────────────────────────────────── //

impl VcsBackend for JjBackend {
    fn status_digest(&self) -> Result<StatusDigest> {
        let mut digest = self.git.status_digest()?;
        // When the git store is a bare repository at .jj/repo/store/git, its
        // directory name is "git", not the project name. Override repo_name
        // with the actual project root's directory name.
        digest.repo_name = self
            .root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_owned();
        Ok(digest)
    }

    fn local_branches(&self) -> Result<Vec<BranchInfo>> {
        self.git.local_branches()
    }

    fn remote_branches(&self) -> Result<Vec<BranchInfo>> {
        self.git.remote_branches()
    }

    fn list_commits(&self) -> Result<Vec<CommitInfo>> {
        self.git.list_commits()
    }

    fn list_commits_sorted(&self, order: SortOrder) -> Result<Vec<CommitInfo>> {
        self.git.list_commits_sorted(order)
    }

    fn log_since(&self, since: SystemTime, until: SystemTime) -> Result<Vec<CommitInfo>> {
        self.git.log_since(since, until)
    }

    fn find_commit(&self, id: &CommitId) -> Result<CommitInfo> {
        self.git.find_commit(id)
    }

    fn list_tags(&self) -> Result<Vec<TagInfo>> {
        self.git.list_tags()
    }

    fn list_tags_sorted(&self, order: SortOrder) -> Result<Vec<TagInfo>> {
        self.git.list_tags_sorted(order)
    }

    fn create_tag(&self, name: &str) -> Result<()> {
        self.git.create_tag(name)
    }

    /// Creates a lightweight tag, ignoring `message`.
    ///
    /// Jujutsu only supports lightweight tags; passing a message has no effect.
    fn create_annotated_tag(&self, name: &str, _message: &str) -> Result<()> {
        self.git.create_tag(name)
    }

    fn delete_tag(&self, name: &str) -> Result<()> {
        self.git.delete_tag(name)
    }

    fn diff(&self, from: &CommitId, to: &CommitId) -> Result<DiffSummary> {
        self.git.diff(from, to)
    }

    fn remote_url(&self, name: &str) -> Option<String> {
        self.git.remote_url(name)
    }
}
