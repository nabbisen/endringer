//! Remote and reference inventory (RFC 011).

use anyhow::Result;
use endringer_core::types::{RefInfo, RefKind, RefTarget, RemoteInfo};
use gix::Repository;

use crate::util::gix_id_to_object_id;

// ── Remotes ──────────────────────────────────────────────────────────────── //

/// Returns all configured remotes, sorted ascending by name.
pub(crate) fn remotes(repo: &Repository) -> Result<Vec<RemoteInfo>> {
    let mut result: Vec<RemoteInfo> = repo
        .remote_names()
        .into_iter()
        .filter_map(|name_bstr| {
            let name = name_bstr.to_string();
            let remote = repo.find_remote(name_bstr.as_ref()).ok()?;

            let fetch_url = remote
                .url(gix::remote::Direction::Fetch)
                .map(|u| u.to_bstring().to_string())
                .into_iter()
                .collect::<Vec<_>>();

            // Push URL is only populated when an explicit pushurl is set.
            // When it equals the fetch URL, gix returns the same URL for both
            // directions; only emit push_urls when they differ.
            let push_url: Vec<String> = {
                let fetch = remote
                    .url(gix::remote::Direction::Fetch)
                    .map(|u| u.to_bstring());
                let push = remote
                    .url(gix::remote::Direction::Push)
                    .map(|u| u.to_bstring());
                match (fetch, push) {
                    (Some(f), Some(p)) if f != p => vec![p.to_string()],
                    (None, Some(p)) => vec![p.to_string()],
                    _ => vec![],
                }
            };

            Some(RemoteInfo {
                name,
                fetch_urls: fetch_url,
                push_urls: push_url,
            })
        })
        .collect();

    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

// ── References ───────────────────────────────────────────────────────────── //

/// Returns all refs, sorted ascending by full name.
pub(crate) fn references(repo: &Repository) -> Result<Vec<RefInfo>> {
    let mut result = Vec::new();

    for reference in repo.references()?.all()? {
        let reference = reference.map_err(|e| anyhow::anyhow!("{e}"))?;
        result.push(gix_ref_to_info(&reference));
    }

    // Also include HEAD.
    if let Some(head_info) = head_ref_info(repo) {
        result.push(head_info);
    }

    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

/// Returns refs matching `kind`, sorted ascending by full name.
pub(crate) fn references_by_kind(
    repo: &Repository,
    kind: RefKind,
) -> Result<Vec<RefInfo>> {
    if kind == RefKind::Head {
        return Ok(head_ref_info(repo).into_iter().collect());
    }

    let prefix = match kind {
        RefKind::LocalBranch  => "refs/heads/",
        RefKind::RemoteBranch => "refs/remotes/",
        RefKind::Tag          => "refs/tags/",
        _                     => "",
    };

    let mut result = Vec::new();

    if prefix.is_empty() {
        // RefKind::Other — enumerate all non-HEAD refs and filter.
        for reference in repo.references()?.all()? {
            let reference = reference.map_err(|e| anyhow::anyhow!("{e}"))?;
            let info = gix_ref_to_info(&reference);
            if info.kind == RefKind::Other {
                result.push(info);
            }
        }
    } else {
        for reference in repo.references()?.prefixed(prefix)? {
            let reference = reference.map_err(|e| anyhow::anyhow!("{e}"))?;
            result.push(gix_ref_to_info(&reference));
        }
    }

    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

// ── Helpers ──────────────────────────────────────────────────────────────── //

fn gix_ref_to_info(reference: &gix::Reference<'_>) -> RefInfo {
    let full_name = reference.name().as_bstr().to_string();
    let kind = classify_ref(&full_name);

    let target = match reference.target() {
        gix::refs::TargetRef::Object(oid) => {
            RefTarget::Direct(gix_id_to_object_id(oid.to_owned()))
        }
        gix::refs::TargetRef::Symbolic(name) => {
            RefTarget::Symbolic(name.as_bstr().to_string())
        }
    };

    RefInfo { name: full_name, kind, target }
}

fn head_ref_info(repo: &Repository) -> Option<RefInfo> {
    let head = repo.head().ok()?;
    let target = match head.kind {
        gix::head::Kind::Detached { target, .. } => {
            RefTarget::Direct(gix_id_to_object_id(target))
        }
        gix::head::Kind::Symbolic(ref r) => {
            RefTarget::Symbolic(r.name.as_bstr().to_string())
        }
        gix::head::Kind::Unborn(_) => RefTarget::Unborn,
    };
    Some(RefInfo { name: "HEAD".to_owned(), kind: RefKind::Head, target })
}

fn classify_ref(full_name: &str) -> RefKind {
    if full_name.starts_with("refs/heads/")      { RefKind::LocalBranch  }
    else if full_name.starts_with("refs/remotes/") { RefKind::RemoteBranch }
    else if full_name.starts_with("refs/tags/")    { RefKind::Tag          }
    else                                            { RefKind::Other        }
}
