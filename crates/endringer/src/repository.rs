//! Repository handle and constructors.

use std::path::Path;
use std::time::SystemTime;

use endringer_core::error::Result;
use endringer_core::backend::VcsBackend;
use endringer_core::types::{
    AheadBehind, BackendKind, BlameEntry, BranchInfo, BranchTrackingInfo, CommitId,
    CommitInfo, CommitQuery, CommitQueryResult, ConflictSummary, DiffSummary,
    OperationState, RefInfo, RefKind, RemoteInfo, RepositoryInfo,
    SortOrder, StashEntry, StatusDigest, SubmoduleInfo, TagInfo, TreeEntry,
    WorktreeInfo, WorktreeStatus,
};
use endringer_git::GitBackend;
use endringer_jj::JjBackend;

// ── Constructors ─────────────────────────────────────────────────────────── //

/// Opens a Git repository at `repo_path`.
///
/// ```no_run
/// use endringer::repository::repository;
///
/// let repo = repository(std::path::Path::new(".")).expect("open repo");
/// let digest = repo.status_digest().expect("status digest");
/// println!("{}", digest.current_branch);
/// ```
pub fn repository(repo_path: &Path) -> Result<Repository> {
    let backend = GitBackend::open(repo_path)
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("could not find repository") || msg.contains("Could not find")
                || msg.contains("not a git repository") || msg.contains("no .git")
            {
                endringer_core::error::Error::NotARepository { path: repo_path.to_path_buf() }
            } else {
                endringer_core::error::anyhow_to_backend(e)
            }
        })?;
    Ok(Repository::with_backend(Box::new(backend), BackendKind::Git))
}

/// Opens a Jujutsu repository at `repo_path`.
///
/// The `jj` binary is **not** required; the underlying git store is read
/// directly with gix.
///
/// ```no_run
/// use endringer::repository::jj_repository;
///
/// let repo = jj_repository(std::path::Path::new(".")).expect("open jj repo");
/// let digest = repo.status_digest().expect("status digest");
/// println!("{}", digest.current_branch);
/// ```
pub fn jj_repository(repo_path: &Path) -> Result<Repository> {
    let backend = JjBackend::open(repo_path)
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not a jj repository") {
                endringer_core::error::Error::NotARepository { path: repo_path.to_path_buf() }
            } else {
                endringer_core::error::anyhow_to_backend(e)
            }
        })?;
    Ok(Repository::with_backend(Box::new(backend), BackendKind::Jj))
}

// ── Repository ───────────────────────────────────────────────────────────── //

/// Handle for all VCS operations.
///
/// Returned by [`repository`] (Git) or [`jj_repository`] (Jujutsu). All
/// methods are backend-agnostic; the active backend is visible via
/// [`Repository::backend_kind`].
///
/// Custom backends can be injected via [`Repository::with_backend`].
pub struct Repository {
    backend: Box<dyn VcsBackend>,
    kind: BackendKind,
}

impl Repository {
    /// Constructs a `Repository` from a custom [`VcsBackend`] implementation.
    ///
    /// This is the extension point for third-party backends.
    pub fn with_backend(backend: Box<dyn VcsBackend>, kind: BackendKind) -> Self {
        Repository { backend, kind }
    }

    /// Returns which VCS backend this repository uses.
    pub fn backend_kind(&self) -> BackendKind {
        self.kind
    }

    // ── Status ─────────────────────────────────────────────────────────── //

    /// Returns a lightweight snapshot of the repository's current state.
    pub fn status_digest(&self) -> Result<StatusDigest> {
        self.backend.status_digest()
    }

    // ── Branches ───────────────────────────────────────────────────────── //

    /// Returns all local branches.
    pub fn local_branches(&self) -> Result<Vec<BranchInfo>> {
        self.backend.local_branches()
    }

    /// Returns all remote-tracking branches.
    pub fn remote_branches(&self) -> Result<Vec<BranchInfo>> {
        self.backend.remote_branches()
    }

    // ── Commits ────────────────────────────────────────────────────────── //

    /// Returns the full commit history reachable from HEAD, newest first.
    pub fn list_commits(&self) -> Result<Vec<CommitInfo>> {
        self.backend.list_commits()
    }

    /// Returns the full commit history sorted by `order`.
    pub fn list_commits_sorted(&self, order: SortOrder) -> Result<Vec<CommitInfo>> {
        self.backend.list_commits_sorted(order)
    }

    /// Returns commits whose author timestamp falls within `[since, until]`.
    pub fn log_since(&self, since: SystemTime, until: SystemTime) -> Result<Vec<CommitInfo>> {
        self.backend.log_since(since, until)
    }

    /// Looks up a single commit by its [`CommitId`] (O(1) object-DB lookup).
    pub fn find_commit(&self, id: &CommitId) -> Result<CommitInfo> {
        self.backend.find_commit(id)
    }

    /// Returns a bounded page of commit history according to `query`.
    ///
    /// `CommitQueryResult::truncated` is `true` when `max_count` was reached
    /// and more commits may exist. Use [`CommitQuery::head_page`] for the
    /// common first-page case.
    pub fn query_commits(&self, query: CommitQuery) -> Result<CommitQueryResult> {
        self.backend.query_commits(query)
    }

    // ── Tags ───────────────────────────────────────────────────────────── //

    /// Returns all tags.
    pub fn list_tags(&self) -> Result<Vec<TagInfo>> {
        self.backend.list_tags()
    }

    /// Returns all tags sorted by `order`.
    pub fn list_tags_sorted(&self, order: SortOrder) -> Result<Vec<TagInfo>> {
        self.backend.list_tags_sorted(order)
    }

    /// Creates a lightweight tag at HEAD.
    pub fn create_tag(&self, name: &str) -> Result<()> {
        self.backend.create_tag(name)
    }

    /// Creates an annotated tag at HEAD.
    ///
    /// Requires `user.name` and `user.email` to be set in git config.
    /// On the jj backend this returns an error: jj does not support annotated
    /// tags. Use [`create_tag`][Repository::create_tag] for a lightweight tag
    /// instead.
    pub fn create_annotated_tag(&self, name: &str, message: &str) -> Result<()> {
        self.backend.create_annotated_tag(name, message)
    }

    /// Deletes the named tag.
    pub fn delete_tag(&self, name: &str) -> Result<()> {
        self.backend.delete_tag(name)
    }

    // ── Diff ───────────────────────────────────────────────────────────── //

    /// Returns a file-level diff summary between two commits.
    ///
    /// Paths within each category of [`DiffSummary`] are sorted ascending.
    pub fn diff(&self, from: &CommitId, to: &CommitId) -> Result<DiffSummary> {
        self.backend.diff(from, to)
    }

    // ── Remotes ────────────────────────────────────────────────────────── //

    /// Returns the fetch URL of the named remote.
    ///
    /// Returns `Ok(None)` if no remote with that name is configured.
    /// Returns `Err` only on an actual I/O or config parsing failure.
    pub fn remote_url(&self, name: &str) -> Result<Option<String>> {
        self.backend.remote_url(name)
    }

    // ── Working tree ───────────────────────────────────────────────────── //

    /// Returns `true` if the working tree has any uncommitted changes
    /// (staged or unstaged).
    ///
    /// Bare repositories always return `false`. On the jj backend this
    /// delegates to the underlying git store.
    pub fn is_dirty(&self) -> Result<bool> {
        self.backend.is_dirty()
    }

    // ── Commit graph ───────────────────────────────────────────────────── //

    /// Returns the best common ancestor of `a` and `b`, or `None` if the
    /// two commits have no shared history.
    pub fn merge_base(&self, a: &CommitId, b: &CommitId) -> Result<Option<CommitId>> {
        self.backend.merge_base(a, b)
    }

    /// Returns `true` if `candidate` is an ancestor (direct or transitive)
    /// of `descendant`. A commit is its own ancestor.
    pub fn is_ancestor(&self, candidate: &CommitId, descendant: &CommitId) -> Result<bool> {
        self.backend.is_ancestor(candidate, descendant)
    }

    // ── Ahead / behind ─────────────────────────────────────────────────── //

    /// Returns ahead/behind counts between `local` and `upstream` commit tips.
    ///
    /// Uses a single symmetric-difference traversal equivalent to
    /// `git rev-list --left-right --count local...upstream`.
    /// Cost is O(commits strictly between the merge base and the two tips).
    ///
    /// See [`AheadBehind`] for the edge-case contract.
    pub fn ahead_behind(
        &self,
        local: &CommitId,
        upstream: &CommitId,
    ) -> Result<AheadBehind> {
        self.backend.ahead_behind(local, upstream)
    }

    /// Returns ahead/behind counts for the configured upstream of `branch`.
    ///
    /// Returns `Ok(None)` when the branch has no configured upstream.
    /// Returns `Err` when the configured upstream ref no longer exists locally.
    pub fn branch_ahead_behind(&self, branch: &str) -> Result<Option<AheadBehind>> {
        self.backend.branch_ahead_behind(branch)
    }

    // ── Repository info ────────────────────────────────────────────────── //

    /// Returns a lightweight metadata snapshot of the repository.
    ///
    /// Includes backend kind, working tree path, HEAD state, object format,
    /// and backend capabilities. All fields are a fresh read at call time.
    pub fn repository_info(&self) -> Result<RepositoryInfo> {
        self.backend.repository_info()
    }

    // ── Branch tracking ────────────────────────────────────────────────── //

    /// Returns tracking metadata and divergence data for a single local branch.
    ///
    /// Includes whether the configured upstream exists, and ahead/behind counts
    /// relative to that upstream.
    pub fn branch_tracking(&self, branch: &str) -> Result<BranchTrackingInfo> {
        self.backend.branch_tracking(branch)
    }

    /// Returns tracking metadata for all local branches, sorted ascending
    /// by full ref name.
    pub fn local_branch_tracking(&self) -> Result<Vec<BranchTrackingInfo>> {
        self.backend.local_branch_tracking()
    }

    /// Returns `true` if `branch` has been merged into `target`.
    ///
    /// Equivalent to `is_ancestor(branch_tip, target_tip)` but named to
    /// prevent accidental argument reversal.
    pub fn is_merged_into(&self, branch: &str, target: &str) -> Result<bool> {
        self.backend.is_merged_into(branch, target)
    }

    // ── Operation and conflict state ───────────────────────────────────── //

    /// Returns the current in-progress repository operation, if any.
    ///
    /// Reads Git marker files: `rebase-merge/`, `rebase-apply/`,
    /// `MERGE_HEAD`, `CHERRY_PICK_HEAD`, `REVERT_HEAD`, `BISECT_LOG`.
    /// Returns `Ok(OperationState::None)` when no operation is in progress.
    pub fn operation_state(&self) -> Result<OperationState> {
        self.backend.operation_state()
    }

    /// Returns paths with unmerged (higher-stage) index entries, sorted
    /// ascending.
    ///
    /// Returns an empty `Vec` when there are no conflicts.
    pub fn unmerged_paths(&self) -> Result<Vec<std::path::PathBuf>> {
        self.backend.unmerged_paths()
    }

    /// Returns a structured summary of all conflicted index entries.
    ///
    /// Includes per-stage object IDs for each conflicted path.
    /// For a lighter-weight check use [`unmerged_paths`][Self::unmerged_paths].
    pub fn conflict_summary(&self) -> Result<ConflictSummary> {
        self.backend.conflict_summary()
    }

    // ── Point-in-time reads ────────────────────────────────────────────── //

    /// Returns per-line commit attribution for `path` at `commit_id`.
    ///
    /// Like [`blame`][Self::blame] but at an arbitrary historical commit
    /// rather than HEAD.
    pub fn blame_at(&self, path: &std::path::Path, commit_id: &CommitId) -> Result<Vec<BlameEntry>> {
        self.backend.blame_at(path, commit_id)
    }

    /// Returns the root-level tree entries at `commit_id`, sorted ascending
    /// by name. Non-recursive — use [`tree_at_path`][Self::tree_at_path]
    /// to descend into directories.
    pub fn tree_at_commit(&self, commit_id: &CommitId) -> Result<Vec<TreeEntry>> {
        self.backend.tree_at_commit(commit_id)
    }

    /// Returns the tree entries of the directory at `path` within `commit_id`,
    /// sorted ascending by name.
    ///
    /// Returns `Err` if `path` does not exist in the commit or is not a
    /// directory.
    pub fn tree_at_path(&self, commit_id: &CommitId, path: &std::path::Path) -> Result<Vec<TreeEntry>> {
        self.backend.tree_at_path(commit_id, path)
    }

    // ── Remote and reference inventory ────────────────────────────────── //

    /// Returns all configured remotes, sorted ascending by name.
    ///
    /// Each [`RemoteInfo`] carries the remote name and fetch/push URLs.
    /// `push_urls` is empty when no explicit push URL is configured.
    pub fn remotes(&self) -> Result<Vec<RemoteInfo>> {
        self.backend.remotes()
    }

    /// Returns all references, sorted ascending by full name.
    ///
    /// Includes local branches, remote-tracking branches, tags, HEAD, and
    /// any other refs present in the repository.
    pub fn references(&self) -> Result<Vec<RefInfo>> {
        self.backend.references()
    }

    /// Returns references of the given `kind`, sorted ascending by full name.
    ///
    /// Use [`RefKind::LocalBranch`], [`RefKind::RemoteBranch`],
    /// [`RefKind::Tag`], [`RefKind::Head`], or [`RefKind::Other`] to filter.
    pub fn references_by_kind(&self, kind: RefKind) -> Result<Vec<RefInfo>> {
        self.backend.references_by_kind(kind)
    }

    // ── Blame ──────────────────────────────────────────────────────────── //

    /// Returns per-line commit attribution for the file at `path` (relative
    /// to the repository root) as of HEAD.
    ///
    /// Entries are in ascending line order. Each [`BlameEntry`] covers a
    /// contiguous range of lines introduced by the same commit.
    pub fn blame(&self, path: &std::path::Path) -> Result<Vec<BlameEntry>> {
        self.backend.blame(path)
    }

    // ── Working tree status ────────────────────────────────────────────── //

    /// Returns per-file working-tree status.
    ///
    /// Equivalent to `git status` output, broken into staged changes,
    /// unstaged changes, and untracked files. Bare repositories always
    /// return an empty [`WorktreeStatus`].
    ///
    /// Gitignore rules are applied to untracked files. If exclude-stack
    /// initialisation fails, the backend degrades gracefully and reports
    /// all untracked files without filtering.
    pub fn worktree_status(&self) -> Result<WorktreeStatus> {
        self.backend.worktree_status()
    }

    // ── File content ───────────────────────────────────────────────────── //

    /// Returns the raw content of `path` (relative to the repository root)
    /// as it exists in the tree of `commit_id`.
    ///
    /// Useful for reading historical file versions without checking out the
    /// commit.
    pub fn file_at_commit(
        &self,
        path: &std::path::Path,
        commit_id: &CommitId,
    ) -> Result<Vec<u8>> {
        self.backend.file_at_commit(path, commit_id)
    }

    // ── Submodules ─────────────────────────────────────────────────────── //

    /// Returns metadata for every submodule declared in `.gitmodules`.
    /// Returns an empty `Vec` when no `.gitmodules` file is present.
    pub fn submodules(&self) -> Result<Vec<SubmoduleInfo>> {
        self.backend.submodules()
    }

    // ── Stash ──────────────────────────────────────────────────────────── //

    /// Returns all stash entries, newest first (`stash@{0}` first).
    /// Returns an empty `Vec` when there are no stashed changes.
    pub fn stash_entries(&self) -> Result<Vec<StashEntry>> {
        self.backend.stash_entries()
    }

    // ── Linked worktrees ───────────────────────────────────────────────── //

    /// Returns all linked worktrees. The main worktree is not included.
    /// Returns an empty `Vec` for repositories with no linked worktrees.
    pub fn worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        self.backend.worktrees()
    }
}

#[cfg(test)]
mod tests;
