//! Public types for endringer.
//!
//! Identity types (`CommitId`, `ObjectId`, and their error types) live in
//! [`types::identity`] and are re-exported from here so that all public paths
//! remain `endringer_core::types::CommitId`, unchanged.

pub mod identity;

pub use identity::{
    CommitId, CommitIdFromHexError, ObjectId, ObjectIdFromHexError,
};

use std::time::SystemTime;

// ── Remote and reference inventory (RFC 011) ─────────────────────────────── //

/// Metadata about a configured git remote.
///
/// Returned by [`crate::backend::VcsBackend::remotes`].
///
/// If the remote has no explicit `pushurl` configured, `push_urls` is empty —
/// git falls back to the fetch URL for pushes, but endringer reports only what
/// is explicitly configured.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteInfo {
    /// The remote's short name (e.g. `origin`).
    pub name: String,
    /// Configured fetch URLs (`remote.<name>.url`).
    pub fetch_urls: Vec<String>,
    /// Explicitly configured push URLs (`remote.<name>.pushurl`).
    /// Empty when no dedicated push URL is set.
    pub push_urls: Vec<String>,
}

/// Classifies a git reference by its prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefKind {
    /// A local branch (`refs/heads/…`).
    LocalBranch,
    /// A remote-tracking branch (`refs/remotes/…`).
    RemoteBranch,
    /// A tag (`refs/tags/…`).
    Tag,
    /// The `HEAD` pseudo-ref.
    Head,
    /// Any other ref (notes, stash, bisect, …).
    Other,
}

/// The target of a git reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefTarget {
    /// Points directly to an object (commit, tag object, tree, blob).
    Direct(ObjectId),
    /// Symbolic reference pointing to another ref by name.
    Symbolic(String),
    /// Symbolic ref that names a branch not yet created (fresh `git init`).
    Unborn,
}

/// A single git reference.
///
/// Returned by [`crate::backend::VcsBackend::references`] and
/// [`crate::backend::VcsBackend::references_by_kind`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefInfo {
    /// The full ref name (e.g. `refs/heads/main`, `HEAD`).
    pub name: String,
    /// Classifies the ref by its prefix.
    pub kind: RefKind,
    /// What the ref points to.
    pub target: RefTarget,
}

// ── Tree entries (RFC 010) ────────────────────────────────────────────────── //

/// The kind of a tree entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeEntryKind {
    /// A regular file.
    File,
    /// A directory (sub-tree).
    Directory,
    /// A symbolic link.
    Symlink,
    /// A git submodule (commit reference).
    Submodule,
    /// An unrecognised entry kind.
    Other,
}

/// An entry in a git tree (directory listing) at a specific commit.
///
/// Returned by [`crate::backend::VcsBackend::tree_at_commit`] and
/// [`crate::backend::VcsBackend::tree_at_path`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeEntry {
    /// Path relative to the root of the queried tree, using `/` as separator.
    pub path: std::path::PathBuf,
    /// Bare file/directory name (last component of `path`).
    pub name: String,
    /// Entry kind.
    pub kind: TreeEntryKind,
    /// Object ID of the entry (blob, tree, or commit for submodules).
    pub object_id: ObjectId,
    /// Byte size of the entry's content, if available.
    /// `None` for directories and submodules.
    pub size: Option<u64>,
    /// Whether the file is executable (only meaningful for `File` entries).
    pub executable: bool,
}

// ── AheadBehind ──────────────────────────────────────────────────────────── //

/// Ahead/behind counts between two commit tips.
///
/// Returned by [`crate::backend::VcsBackend::ahead_behind`].
///
/// # Edge cases
///
/// | Relationship | `ahead` | `behind` | `merge_base` |
/// |---|---|---|---|
/// | `local == upstream` | 0 | 0 | `Some(local)` |
/// | local descends from upstream | > 0 | 0 | `Some(...)` |
/// | upstream descends from local | 0 | > 0 | `Some(...)` |
/// | both diverged | > 0 | > 0 | `Some(...)` |
/// | unrelated histories | > 0 | > 0 | `None` |
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AheadBehind {
    /// Commits reachable from `local` but not from `upstream`.
    pub ahead: usize,
    /// Commits reachable from `upstream` but not from `local`.
    pub behind: usize,
    /// Best common ancestor commit, if one exists.
    pub merge_base: Option<CommitId>,
}

// ── Operation and conflict state (RFC 008) ────────────────────────────────── //

/// The kind of rebase in progress.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RebaseKind {
    /// Rebase using the merge backend (`rebase-merge/` present).
    Merge,
    /// Rebase using the apply/am backend (`rebase-apply/` present).
    Apply,
    /// A rebase directory exists but the kind cannot be determined.
    Unknown,
}

/// The in-progress repository operation, if any.
///
/// Returned by [`crate::backend::VcsBackend::operation_state`].
///
/// Detected by reading Git marker files under the `.git/` directory:
///
/// | State | Marker |
/// |---|---|
/// | `Rebase { kind: Merge }` | `rebase-merge/` |
/// | `Rebase { kind: Apply }` | `rebase-apply/` |
/// | `Merge` | `MERGE_HEAD` |
/// | `CherryPick` | `CHERRY_PICK_HEAD` |
/// | `Revert` | `REVERT_HEAD` |
/// | `Bisect` | `BISECT_LOG` or `refs/bisect/` |
/// | `None` | none of the above |
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperationState {
    /// No in-progress operation.
    None,
    /// A merge is in progress. `heads` holds the OIDs from `MERGE_HEAD`.
    Merge { heads: Vec<CommitId> },
    /// A rebase is in progress.
    Rebase { kind: RebaseKind },
    /// A cherry-pick is in progress. `head` is the OID from `CHERRY_PICK_HEAD`.
    CherryPick { head: Option<CommitId> },
    /// A revert is in progress. `head` is the OID from `REVERT_HEAD`.
    Revert { head: Option<CommitId> },
    /// A bisect is in progress.
    Bisect,
}

/// One stage slot of a conflicted path (base = 1, ours = 2, theirs = 3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictStage {
    /// Stage number (1 = base, 2 = ours, 3 = theirs).
    pub stage: u8,
    /// Object ID of the blob in this stage.
    pub object_id: ObjectId,
}

/// A path with one or more conflict stages in the index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictPath {
    /// Path relative to the repository root.
    pub path: std::path::PathBuf,
    /// The present stage slots (subset of base/ours/theirs).
    pub stages: Vec<ConflictStage>,
}

/// A summary of all conflicted paths in the index.
///
/// Returned by [`crate::backend::VcsBackend::conflict_summary`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictSummary {
    /// All paths with higher-stage index entries, sorted ascending.
    pub paths: Vec<ConflictPath>,
}

impl ConflictSummary {
    /// Returns `true` if there are no conflicted paths.
    pub fn is_empty(&self) -> bool { self.paths.is_empty() }
    /// Returns the number of conflicted paths.
    pub fn len(&self) -> usize { self.paths.len() }
}

// ── BranchInfo ────────────────────────────────────────────────────────────── //

/// Tracking and divergence metadata for a local branch.
///
/// Returned by [`crate::backend::VcsBackend::branch_tracking`] and
/// [`crate::backend::VcsBackend::local_branch_tracking`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchTrackingInfo {
    /// Short local branch name (e.g. `main`).
    pub branch: String,
    /// Full ref name (e.g. `refs/heads/main`).
    pub full_name: String,
    /// Commit at the branch tip.
    pub tip_commit_id: CommitId,
    /// Configured upstream full ref name, if any.
    pub upstream: Option<String>,
    /// `true` if an upstream is configured but no longer resolvable locally
    /// (e.g. the remote-tracking ref was pruned after a remote branch deletion).
    pub upstream_gone: bool,
    /// Ahead/behind counts relative to the upstream tip.
    /// `None` when no upstream is configured or when the upstream is gone.
    pub ahead_behind: Option<AheadBehind>,
}

// ── RepositoryInfo types (RFC 009) ─────────────────────────────────────────── //

/// The object hashing algorithm used by a repository.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ObjectFormat {
    /// SHA-1 (the default for Git repositories).
    Sha1,
    /// SHA-256 (opt-in via `extensions.objectFormat = sha256`).
    Sha256,
    /// An object format reported by gix that endringer does not model.
    /// The string carries the raw format name for diagnostics.
    Unknown(String),
}

/// The state of the HEAD reference.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum HeadState {
    /// HEAD points at a branch that has at least one commit.
    Attached {
        /// Short branch name (e.g. `main`).
        branch: String,
        /// Full ref name (e.g. `refs/heads/main`).
        full_name: String,
        /// The commit HEAD resolves to.
        commit_id: CommitId,
    },
    /// HEAD is detached at a specific commit (not on any branch).
    Detached {
        /// The commit HEAD points to directly.
        commit_id: CommitId,
    },
    /// HEAD names a branch that has no commits yet (fresh `git init`).
    Unborn {
        /// The target branch name if it can be determined, or `None`.
        branch: Option<String>,
    },
    /// HEAD reference is absent or unreadable.
    Missing,
}

/// Which features this backend and repository support.
///
/// Capabilities are read at the time of the `repository_info()` call.
/// They are not a subscription; external mutations may change them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryCapabilities {
    /// Repository has a working tree (not bare).
    pub working_tree: bool,
    /// [`VcsBackend::create_tag`] is supported for this repository.
    pub tag_create_lightweight: bool,
    /// [`VcsBackend::create_annotated_tag`] is supported for this repository.
    pub tag_create_annotated: bool,
    /// [`VcsBackend::delete_tag`] is supported for this repository.
    pub tag_delete: bool,
    /// [`VcsBackend::branch_tracking`] and related methods are supported.
    pub branch_tracking: bool,
    /// Operation/conflict state reads (RFC 008) are supported.
    pub operation_state: bool,
    /// Conflict state reads (RFC 008) are supported.
    pub conflict_state: bool,
    /// jj-native concepts (op log, change IDs) are exposed — false until a
    /// future jj-native RFC.
    pub jj_native_state: bool,
}

/// Lightweight repository metadata snapshot.
///
/// Returned by [`crate::backend::VcsBackend::repository_info`].
/// All fields are a fresh read at call time; this is not a subscription.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryInfo {
    /// Which VCS backend opened this repository.
    pub backend: BackendKind,
    /// Directory name of the working tree (or git-dir for bare repos).
    pub repo_name: String,
    /// Absolute path to the working tree, or `None` for bare repositories.
    pub workdir: Option<std::path::PathBuf>,
    /// Absolute path to the VCS metadata directory (`.git/` or `.jj/`).
    pub vcs_dir: std::path::PathBuf,
    /// Whether this is a bare repository (no working tree).
    pub is_bare: bool,
    /// Object hashing algorithm in use.
    pub object_format: ObjectFormat,
    /// Current HEAD state.
    pub head: HeadState,
    /// Backend and repository capabilities.
    pub capabilities: RepositoryCapabilities,
}
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

// ── StatusDigest ─────────────────────────────────────────────────────────── //

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

// ── CommitInfo ───────────────────────────────────────────────────────────── //

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

// ── TagAnnotation / TagInfo ───────────────────────────────────────────────── //

/// Annotation data for an annotated tag.
///
/// Absent (`TagInfo::annotation` is `None`) for lightweight tags.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagAnnotation {
    /// The annotation message (trimmed).
    pub message: String,
    /// Tagger name, if recorded in the tag object.
    pub tagger_name: Option<String>,
    /// Tagger email, if recorded in the tag object.
    pub tagger_email: Option<String>,
    /// Tagger timestamp, if recorded in the tag object.
    pub tagger_timestamp: Option<SystemTime>,
}

/// Information about a tag.
///
/// **Breaking change (v0.18)**: an `annotation` field was added.
/// **Breaking change (v0.28)**: `TagAnnotation` gained a `tagger_email` field.
/// Code constructing `TagAnnotation` directly must add `tagger_email: None`.
///
/// `commit_id` is always the commit reached by peeling the tag target through
/// any tag objects. Tags that cannot be peeled to a commit are skipped by
/// list methods.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagInfo {
    /// Short tag name, e.g. `v1.0.0`.
    pub name: String,
    /// Full ref name, e.g. `refs/tags/v1.0.0`.
    pub full_name: String,
    /// Commit ID reached by peeling the tag to a commit.
    pub commit_id: CommitId,
    /// First line of the tagged commit's message.
    pub commit_summary: String,
    /// Author timestamp of the tagged commit.
    pub commit_timestamp: SystemTime,
    /// Present for annotated tags; `None` for lightweight tags.
    pub annotation: Option<TagAnnotation>,
}

// ── SortOrder / DiffSummary / BackendKind / BlameEntry ───────────────────── //

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

/// One contiguous span of lines in a file, attributed to a single commit.
///
/// Lines are **1-indexed** and inclusive on both ends.
/// `start_line == end_line` means a single-line entry.
///
/// Returned by [`crate::repository::Repository::blame`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlameEntry {
    /// Commit that introduced these lines.
    pub commit_id: CommitId,
    /// First line in the blamed file (1-indexed, inclusive).
    pub start_line: u32,
    /// Last line in the blamed file (1-indexed, inclusive).
    pub end_line: u32,
    /// Original file path in the source commit, present only when the file
    /// was renamed between that commit and the blamed file.
    pub original_path: Option<std::path::PathBuf>,
}

// ── Working tree status ───────────────────────────────────────────────────── //

/// The kind of change a [`StatusEntry`] represents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeKind {
    /// A new file was added (not present in the reference point).
    Added,
    /// An existing file was modified.
    Modified,
    /// A tracked file was deleted.
    Deleted,
}

/// A single file entry in a [`WorktreeStatus`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusEntry {
    /// Path relative to the repository root, using the platform path separator.
    pub path: std::path::PathBuf,
    /// Nature of the change.
    pub kind: ChangeKind,
}

/// Detailed working-tree status, equivalent to the output of `git status`.
///
/// Returned by [`crate::repository::Repository::worktree_status`].
///
/// ## Untracked files
///
/// `untracked` lists files present in the working tree that are not tracked
/// by the index. Active gitignore rules (`.gitignore`, `info/exclude`, global
/// excludes) are applied so that ignored files do not appear here. If the
/// exclude stack cannot be initialised, the backend degrades gracefully and
/// reports all untracked files without filtering.
#[derive(Clone, Debug, Default)]
pub struct WorktreeStatus {
    /// Files whose staged blob OID differs from the HEAD tree
    /// (includes new files added to the index and staged deletions).
    pub staged: Vec<StatusEntry>,
    /// Files whose on-disk content or metadata differs from the index
    /// (modifications and deletions that have not been staged yet).
    pub unstaged: Vec<StatusEntry>,
    /// Files present in the working tree but not tracked by git.
    pub untracked: Vec<std::path::PathBuf>,
}

// ── Submodule / Stash / WorktreeInfo ─────────────────────────────────────── //

/// Information about a single Git submodule as declared in `.gitmodules`.
///
/// Returned by [`crate::repository::Repository::submodules`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubmoduleInfo {
    /// Submodule name as declared in `.gitmodules` (typically the same as `path`).
    pub name: String,
    /// Path of the submodule working tree relative to the repository root.
    pub path: std::path::PathBuf,
    /// Remote URL the submodule tracks, if configured.
    pub url: Option<String>,
}

/// A single entry from the stash, corresponding to `stash@{N}`.
///
/// Returned by [`crate::repository::Repository::stash_entries`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StashEntry {
    /// Zero-based index (`stash@{0}` = 0, `stash@{1}` = 1, …).
    /// Entries are returned newest-first.
    pub index: usize,
    /// OID of the stash commit.
    pub commit_id: CommitId,
    /// Stash message (e.g. `"WIP on main: abc1234 initial commit"`).
    pub message: String,
}

/// Information about a linked git worktree.
///
/// Returned by [`crate::repository::Repository::worktrees`]. The main
/// worktree is **not** included; only linked worktrees created via
/// `git worktree add` appear here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorktreeInfo {
    /// The worktree's identifier (the directory name under `.git/worktrees/`).
    pub id: String,
    /// Absolute path to the worktree's working directory.
    pub path: std::path::PathBuf,
    /// Currently checked-out branch (short name), or `"(detached)"` when
    /// the HEAD is in a detached state.
    pub current_branch: String,
    /// Whether the worktree is locked (`git worktree lock`).
    pub is_locked: bool,
}
