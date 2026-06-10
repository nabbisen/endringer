//! [`AsyncRepository`] — async wrapper around [`endringer::repository::Repository`].

use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

use endringer::Error as EndringerError;
use endringer::Result;
use endringer::repository::{Repository, jj_repository, repository};
use endringer::{
    AheadBehind, BlameEntry, BranchInfo, BranchTrackingInfo, CommitId, CommitInfo,
    ConflictSummary, DiffSummary, OperationState, RepositoryInfo, SortOrder,
    StashEntry, StatusDigest, SubmoduleInfo, TagInfo, WorktreeInfo, WorktreeStatus,
};

/// Maps a tokio `JoinError` to `endringer::Error::TaskJoin`.
fn join_err(e: tokio::task::JoinError) -> EndringerError {
    EndringerError::TaskJoin { message: e.to_string() }
}

/// Async wrapper around [`Repository`].
///
/// Cloneable. Each method clones the inner `Arc<Repository>` and dispatches
/// to `tokio::task::spawn_blocking` so that blocking VCS I/O does not stall
/// the async executor.
#[derive(Clone)]
pub struct AsyncRepository {
    inner: Arc<Repository>,
}

impl AsyncRepository {
    /// Opens a Git repository at `path`.
    pub async fn open(path: &Path) -> Result<Self> {
        let path = path.to_path_buf();
        let inner = tokio::task::spawn_blocking(move || repository(&path))
            .await
            .map_err(join_err)??;
        Ok(AsyncRepository { inner: Arc::new(inner) })
    }

    /// Opens a Jujutsu repository at `path`.
    pub async fn open_jj(path: &Path) -> Result<Self> {
        let path = path.to_path_buf();
        let inner = tokio::task::spawn_blocking(move || jj_repository(&path))
            .await
            .map_err(join_err)??;
        Ok(AsyncRepository { inner: Arc::new(inner) })
    }

    // ── Status ─────────────────────────────────────────────────────────── //

    pub async fn status_digest(&self) -> Result<StatusDigest> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.status_digest()).await.map_err(join_err)?
    }

    // ── Branches ───────────────────────────────────────────────────────── //

    pub async fn local_branches(&self) -> Result<Vec<BranchInfo>> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.local_branches()).await.map_err(join_err)?
    }

    pub async fn remote_branches(&self) -> Result<Vec<BranchInfo>> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.remote_branches()).await.map_err(join_err)?
    }

    // ── Commits ────────────────────────────────────────────────────────── //

    pub async fn list_commits(&self) -> Result<Vec<CommitInfo>> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.list_commits()).await.map_err(join_err)?
    }

    pub async fn list_commits_sorted(&self, order: SortOrder) -> Result<Vec<CommitInfo>> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.list_commits_sorted(order)).await.map_err(join_err)?
    }

    pub async fn log_since(&self, since: SystemTime, until: SystemTime) -> Result<Vec<CommitInfo>> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.log_since(since, until)).await.map_err(join_err)?
    }

    pub async fn find_commit(&self, id: CommitId) -> Result<CommitInfo> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.find_commit(&id)).await.map_err(join_err)?
    }

    // ── Tags ───────────────────────────────────────────────────────────── //

    pub async fn list_tags(&self) -> Result<Vec<TagInfo>> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.list_tags()).await.map_err(join_err)?
    }

    pub async fn list_tags_sorted(&self, order: SortOrder) -> Result<Vec<TagInfo>> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.list_tags_sorted(order)).await.map_err(join_err)?
    }

    pub async fn create_tag(&self, name: String) -> Result<()> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.create_tag(&name)).await.map_err(join_err)?
    }

    pub async fn create_annotated_tag(&self, name: String, message: String) -> Result<()> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.create_annotated_tag(&name, &message)).await.map_err(join_err)?
    }

    pub async fn delete_tag(&self, name: String) -> Result<()> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.delete_tag(&name)).await.map_err(join_err)?
    }

    // ── Diff ───────────────────────────────────────────────────────────── //

    pub async fn diff(&self, from: CommitId, to: CommitId) -> Result<DiffSummary> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.diff(&from, &to)).await.map_err(join_err)?
    }

    // ── Remotes ────────────────────────────────────────────────────────── //

    pub async fn remote_url(&self, name: String) -> Result<Option<String>> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.remote_url(&name))
            .await
            .map_err(join_err)?
    }

    // ── Working tree ───────────────────────────────────────────────────── //

    pub async fn is_dirty(&self) -> Result<bool> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.is_dirty()).await.map_err(join_err)?
    }

    // ── Commit graph ───────────────────────────────────────────────────── //

    pub async fn merge_base(&self, a: CommitId, b: CommitId) -> Result<Option<CommitId>> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.merge_base(&a, &b)).await.map_err(join_err)?
    }

    pub async fn is_ancestor(&self, candidate: CommitId, descendant: CommitId) -> Result<bool> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.is_ancestor(&candidate, &descendant)).await.map_err(join_err)?
    }

    // ── Blame ──────────────────────────────────────────────────────────── //

    pub async fn blame(&self, path: std::path::PathBuf) -> Result<Vec<BlameEntry>> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.blame(&path)).await.map_err(join_err)?
    }

    pub async fn worktree_status(&self) -> Result<WorktreeStatus> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.worktree_status()).await.map_err(join_err)?
    }

    pub async fn file_at_commit(
        &self,
        path: std::path::PathBuf,
        commit_id: CommitId,
    ) -> Result<Vec<u8>> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.file_at_commit(&path, &commit_id)).await.map_err(join_err)?
    }

    pub async fn submodules(&self) -> Result<Vec<SubmoduleInfo>> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.submodules()).await.map_err(join_err)?
    }

    pub async fn stash_entries(&self) -> Result<Vec<StashEntry>> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.stash_entries()).await.map_err(join_err)?
    }

    pub async fn worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.worktrees()).await.map_err(join_err)?
    }

    // ── Ahead / behind ─────────────────────────────────────────────────── //

    pub async fn ahead_behind(
        &self,
        local: CommitId,
        upstream: CommitId,
    ) -> Result<AheadBehind> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.ahead_behind(&local, &upstream)).await.map_err(join_err)?
    }

    pub async fn branch_ahead_behind(
        &self,
        branch: String,
    ) -> Result<Option<AheadBehind>> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.branch_ahead_behind(&branch)).await.map_err(join_err)?
    }

    // ── Repository info ────────────────────────────────────────────────── //

    pub async fn repository_info(&self) -> Result<RepositoryInfo> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.repository_info()).await.map_err(join_err)?
    }

    // ── Branch tracking ────────────────────────────────────────────────── //

    pub async fn branch_tracking(&self, branch: String) -> Result<BranchTrackingInfo> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.branch_tracking(&branch)).await.map_err(join_err)?
    }

    pub async fn local_branch_tracking(&self) -> Result<Vec<BranchTrackingInfo>> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.local_branch_tracking()).await.map_err(join_err)?
    }

    pub async fn is_merged_into(&self, branch: String, target: String) -> Result<bool> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.is_merged_into(&branch, &target)).await.map_err(join_err)?
    }

    // ── Operation and conflict state ───────────────────────────────────── //

    pub async fn operation_state(&self) -> Result<OperationState> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.operation_state()).await.map_err(join_err)?
    }

    pub async fn unmerged_paths(&self) -> Result<Vec<std::path::PathBuf>> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.unmerged_paths()).await.map_err(join_err)?
    }

    pub async fn conflict_summary(&self) -> Result<ConflictSummary> {
        let r = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || r.conflict_summary()).await.map_err(join_err)?
    }
}
