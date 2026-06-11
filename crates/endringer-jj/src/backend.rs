//! [`JjBackend`] — delegates to [`GitBackend`] opened on jj's git store.
//!
//! Jujutsu stores commit history in a git object database. This backend opens
//! that store directly with gix; no `jj` binary is required.
//!
//! # Repository layout
//!
//! | Mode | Detection | Git store path |
//! |------|-----------|----------------|
//! | Co-located | `.git/` **and** `.jj/` present | project root |
//! | Native jj  | only `.jj/` present | `.jj/repo/store/git/` |
//!
//! # Annotated tags
//!
//! Jujutsu only supports lightweight tags. `create_annotated_tag` returns an
//! explicit error; callers must use `create_tag` or handle the error.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::bail;
use endringer_core::error::{Error as CrateError, Result};
use endringer_core::backend::VcsBackend;
use endringer_core::types::{AheadBehind, BlameEntry, BranchInfo, BranchTrackingInfo, CommitId, CommitInfo, CommitQuery, CommitQueryResult, DiffEntry, DiffOptions, DiffSummary, RefInfo, RefKind, RemoteInfo, RepositoryInfo, RichWorktreeStatus, SortOrder, StashDetail, StashEntry, StatusDigest, StatusOptions, SubmoduleInfo, SubmoduleSummary, TagInfo, TreeEntry, WorktreeDetail, WorktreeInfo, WorktreeStatus};
use endringer_git::GitBackend;

/// Jujutsu backend backed by the repository's underlying git object store.
pub struct JjBackend {
    git: GitBackend,
    /// Project root (the directory containing `.jj/`).
    root: PathBuf,
}

impl JjBackend {
    /// Opens a Jujutsu repository at `path`.
    ///
    /// Verifies that `path` contains `.jj/`, locates the git store, and opens
    /// it with gix. The `jj` binary is not consulted.
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let root = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        let jj_dir = root.join(".jj");
        if !jj_dir.is_dir() {
            bail!("not a jj repository: no .jj directory at {}", root.display());
        }

        let git_store_path = if root.join(".git").exists() {
            // Co-located: the project root is also the git repository.
            root.clone()
        } else {
            // Native jj: git store lives at .jj/repo/store/git/ (bare repo).
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

impl VcsBackend for JjBackend {
    fn status_digest(&self) -> Result<StatusDigest> {
        let mut digest = self.git.status_digest()?;
        // For native jj repos the git store is at .jj/repo/store/git, whose
        // directory name is "git", not the project name. Override here.
        digest.repo_name = self
            .root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_owned();
        Ok(digest)
    }

    fn local_branches(&self) -> Result<Vec<BranchInfo>> { self.git.local_branches() }
    fn remote_branches(&self) -> Result<Vec<BranchInfo>> { self.git.remote_branches() }
    fn list_commits(&self) -> Result<Vec<CommitInfo>> { self.git.list_commits() }
    fn list_commits_sorted(&self, order: SortOrder) -> Result<Vec<CommitInfo>> { self.git.list_commits_sorted(order) }
    fn log_since(&self, since: SystemTime, until: SystemTime) -> Result<Vec<CommitInfo>> { self.git.log_since(since, until) }
    fn find_commit(&self, id: &CommitId) -> Result<CommitInfo> { self.git.find_commit(id) }
    fn list_tags(&self) -> Result<Vec<TagInfo>> { self.git.list_tags() }
    fn list_tags_sorted(&self, order: SortOrder) -> Result<Vec<TagInfo>> { self.git.list_tags_sorted(order) }
    fn create_tag(&self, name: &str) -> Result<()> { self.git.create_tag(name) }

    /// Always returns an error: Jujutsu does not support annotated tags.
    ///
    /// Use [`create_tag`][Self::create_tag] for a lightweight tag instead.
    fn create_annotated_tag(&self, _name: &str, _message: &str) -> Result<()> {
        Err(CrateError::UnsupportedBackendFeature {
            backend: Some(endringer_core::types::BackendKind::Jj),
            feature: "create_annotated_tag",
        })
    }

    fn delete_tag(&self, name: &str) -> Result<()> { self.git.delete_tag(name) }
    fn diff(&self, from: &CommitId, to: &CommitId) -> Result<DiffSummary> { self.git.diff(from, to) }
    fn remote_url(&self, name: &str) -> Result<Option<String>> { self.git.remote_url(name) }
    fn is_dirty(&self) -> Result<bool> { self.git.is_dirty() }
    fn merge_base(&self, a: &CommitId, b: &CommitId) -> Result<Option<CommitId>> { self.git.merge_base(a, b) }
    fn is_ancestor(&self, candidate: &CommitId, descendant: &CommitId) -> Result<bool> { self.git.is_ancestor(candidate, descendant) }
    fn ahead_behind(&self, local: &CommitId, upstream: &CommitId) -> Result<AheadBehind> { self.git.ahead_behind(local, upstream) }
    fn branch_ahead_behind(&self, branch: &str) -> Result<Option<AheadBehind>> { self.git.branch_ahead_behind(branch) }

    fn repository_info(&self) -> Result<RepositoryInfo> {
        let mut info = self.git.repository_info()?;
        // Override backend kind and repo_name to reflect the jj project root.
        info.backend = endringer_core::types::BackendKind::Jj;
        info.repo_name = self
            .root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_owned();
        // vcs_dir should point to .jj/, not the underlying git store.
        info.vcs_dir = self.root.join(".jj");
        // Recalculate capabilities for jj.
        info.capabilities = endringer_core::types::RepositoryCapabilities {
            working_tree: info.workdir.is_some(),
            tag_create_lightweight: true,
            tag_create_annotated: false, // jj does not support annotated tags
            tag_delete: true,
            branch_tracking: true,
            operation_state: false,
            conflict_state: false,
            jj_native_state: false,
        };
        Ok(info)
    }

    fn branch_tracking(&self, branch: &str) -> Result<BranchTrackingInfo> { self.git.branch_tracking(branch) }
    fn local_branch_tracking(&self) -> Result<Vec<BranchTrackingInfo>> { self.git.local_branch_tracking() }
    fn is_merged_into(&self, b: &str, target: &str) -> Result<bool> { self.git.is_merged_into(b, target) }
    fn blame(&self, path: &std::path::Path) -> Result<Vec<BlameEntry>> { self.git.blame(path) }
    fn worktree_status(&self) -> Result<WorktreeStatus> { self.git.worktree_status() }
    fn file_at_commit(&self, path: &std::path::Path, commit_id: &CommitId) -> Result<Vec<u8>> { self.git.file_at_commit(path, commit_id) }
    fn submodules(&self) -> Result<Vec<SubmoduleInfo>> { self.git.submodules() }
    fn stash_entries(&self) -> Result<Vec<StashEntry>> { self.git.stash_entries() }
    fn worktrees(&self) -> Result<Vec<WorktreeInfo>> { self.git.worktrees() }

    // ── Pure git-store reads: delegate to the inner GitBackend ──────────── //
    // These operate on the underlying git object store and behave identically
    // for jj repositories. (operation_state, unmerged_paths, and
    // conflict_summary are intentionally NOT delegated: jj models operations
    // and conflicts differently from git, and repository_info() declares
    // operation_state and conflict_state as unsupported.)

    fn query_commits(&self, query: CommitQuery) -> Result<CommitQueryResult> { self.git.query_commits(query) }
    fn blame_at(&self, path: &std::path::Path, commit_id: &CommitId) -> Result<Vec<BlameEntry>> { self.git.blame_at(path, commit_id) }
    fn tree_at_commit(&self, commit_id: &CommitId) -> Result<Vec<TreeEntry>> { self.git.tree_at_commit(commit_id) }
    fn tree_at_path(&self, commit_id: &CommitId, path: &std::path::Path) -> Result<Vec<TreeEntry>> { self.git.tree_at_path(commit_id, path) }
    fn references(&self) -> Result<Vec<RefInfo>> { self.git.references() }
    fn references_by_kind(&self, kind: RefKind) -> Result<Vec<RefInfo>> { self.git.references_by_kind(kind) }
    fn remotes(&self) -> Result<Vec<RemoteInfo>> { self.git.remotes() }
    fn diff_entries(&self, from: &CommitId, to: &CommitId, options: DiffOptions) -> Result<Vec<DiffEntry>> { self.git.diff_entries(from, to, options) }
    fn rich_worktree_status(&self, options: StatusOptions) -> Result<RichWorktreeStatus> { self.git.rich_worktree_status(options) }
    fn submodule_summaries(&self) -> Result<Vec<SubmoduleSummary>> { self.git.submodule_summaries() }
    fn stash_detail(&self, index: usize) -> Result<StashDetail> { self.git.stash_detail(index) }
    fn stash_diff(&self, index: usize) -> Result<DiffSummary> { self.git.stash_diff(index) }
    fn worktree_details(&self) -> Result<Vec<WorktreeDetail>> { self.git.worktree_details() }
}
