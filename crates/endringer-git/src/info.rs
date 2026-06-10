//! Repository information and capability discovery (RFC 009).

use anyhow::Result;
use endringer_core::types::{
    BackendKind, HeadState, ObjectFormat, RepositoryCapabilities, RepositoryInfo,
};
use gix::Repository;

use crate::util::gix_id_to_commit_id;

/// Returns a lightweight metadata snapshot of the repository.
pub(crate) fn repository_info(repo: &Repository, backend: BackendKind) -> Result<RepositoryInfo> {
    let is_bare = repo.workdir().is_none();
    let workdir = repo.workdir().map(|p| p.to_path_buf());
    let vcs_dir = repo.git_dir().to_path_buf();

    let repo_name = workdir
        .as_deref()
        .and_then(|p| p.file_name())
        .or_else(|| vcs_dir.parent().and_then(|p| p.file_name()))
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    let object_format = match repo.object_hash() {
        gix::hash::Kind::Sha1 => ObjectFormat::Sha1,
        // SHA-256 requires the gix "sha256" feature; treat anything else as Unknown.
        #[allow(unreachable_patterns)]
        other => ObjectFormat::Unknown(format!("{other:?}")),
    };

    let head = read_head_state(repo)?;
    let capabilities = capabilities_for(backend, is_bare);

    Ok(RepositoryInfo {
        backend,
        repo_name,
        workdir,
        vcs_dir,
        is_bare,
        object_format,
        head,
        capabilities,
    })
}

fn read_head_state(repo: &Repository) -> Result<HeadState> {
    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return Ok(HeadState::Missing),
    };

    if head.is_unborn() {
        // Fresh repo with no commits yet.
        let branch = head
            .referent_name()
            .map(|n| {
                let s = n.to_string();
                s.strip_prefix("refs/heads/")
                    .map(str::to_owned)
                    .unwrap_or(s)
            });
        return Ok(HeadState::Unborn { branch });
    }

    if head.is_detached() {
        let commit_id = head
            .id()
            .map(|id| gix_id_to_commit_id(id.detach()))
            .ok_or_else(|| anyhow::anyhow!("detached HEAD has no commit id"))?;
        return Ok(HeadState::Detached { commit_id });
    }

    // Symbolic (attached) HEAD.
    let full_name = head
        .referent_name()
        .map(|n| n.to_string())
        .ok_or_else(|| anyhow::anyhow!("symbolic HEAD has no referent name"))?;
    let branch = full_name
        .strip_prefix("refs/heads/")
        .unwrap_or(&full_name)
        .to_owned();

    let commit_id = head
        .id()
        .map(|id| gix_id_to_commit_id(id.detach()))
        .ok_or_else(|| anyhow::anyhow!("HEAD has no commit (unborn?)"))?;

    Ok(HeadState::Attached { branch, full_name, commit_id })
}

fn capabilities_for(backend: BackendKind, is_bare: bool) -> RepositoryCapabilities {
    match backend {
        BackendKind::Git => RepositoryCapabilities {
            working_tree:           !is_bare,
            tag_create_lightweight: true,
            tag_create_annotated:   true,
            tag_delete:             true,
            branch_tracking:        true,
            operation_state:        true,  // RFC 008 implemented
            conflict_state:         true,  // RFC 008 implemented
            jj_native_state:        false,
        },
        BackendKind::Jj => RepositoryCapabilities {
            working_tree:           !is_bare,
            tag_create_lightweight: true,
            tag_create_annotated:   false, // jj does not support annotated tags
            tag_delete:             true,
            branch_tracking:        true,
            operation_state:        false,
            conflict_state:         false,
            jj_native_state:        false,
        },
    }
}
