#!/usr/bin/env bash
# scripts/release.sh — endringer release helper
#
# Usage:
#   ./scripts/release.sh             # release the version in Cargo.toml
#   ./scripts/release.sh --dry-run   # simulate without creating tag or tarball
#
# What it does:
#   1. Read version from Cargo.toml
#   2. Verify the working tree is clean
#   3. Run the full test suite
#   4. Create an annotated git tag  v{version}
#   5. Create a source tarball      endringer-v{version}.tar.gz
#      (via git archive — no build artifacts, no .git directory)
#   6. Print a release checklist

set -euo pipefail

# ── configuration ────────────────────────────────────────────────────────────
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${REPO_ROOT}/dist"
DRY_RUN=false

if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN=true
    echo "[dry-run] no tag or tarball will be created"
fi

cd "${REPO_ROOT}"

# ── 1. read version ───────────────────────────────────────────────────────────
VERSION="$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')"
TAG="v${VERSION}"
TARBALL="endringer-${TAG}.tar.gz"

echo "releasing endringer ${TAG}"
echo ""

# ── 2. clean working tree ────────────────────────────────────────────────────
if [[ -n "$(git status --porcelain)" ]]; then
    echo "ERROR: working tree is not clean — commit or stash changes first"
    git status --short
    exit 1
fi
echo "[ok] working tree is clean"

# ── 3. duplicate tag guard ────────────────────────────────────────────────────
if git rev-parse "${TAG}" >/dev/null 2>&1; then
    echo "ERROR: tag ${TAG} already exists — bump the version in Cargo.toml first"
    exit 1
fi
echo "[ok] tag ${TAG} does not yet exist"

# ── 4. test suite ─────────────────────────────────────────────────────────────
echo ""
echo "running tests…"
CARGO="${CARGO_CMD:-cargo-1.91}"
"${CARGO}" test --lib --quiet
echo "[ok] all tests passed"

# ── 5. git tag ────────────────────────────────────────────────────────────────
echo ""
if $DRY_RUN; then
    echo "[dry-run] would create annotated tag: ${TAG}"
else
    git tag -a "${TAG}" -m "Release ${TAG}"
    echo "[ok] created annotated tag ${TAG}"
fi

# ── 6. source tarball ─────────────────────────────────────────────────────────
mkdir -p "${DIST_DIR}"
TARBALL_PATH="${DIST_DIR}/${TARBALL}"

echo ""
if $DRY_RUN; then
    echo "[dry-run] would create ${TARBALL_PATH}"
else
    git archive \
        --format=tar.gz \
        --prefix="endringer-${VERSION}/" \
        --output="${TARBALL_PATH}" \
        "${TAG}"
    BYTES="$(wc -c < "${TARBALL_PATH}")"
    echo "[ok] created ${TARBALL_PATH} (${BYTES} bytes)"
fi

# ── 7. checklist ──────────────────────────────────────────────────────────────
echo ""
echo "────────────────────────────────────────────────────────────────"
echo " Release checklist for ${TAG}"
echo "────────────────────────────────────────────────────────────────"
echo " [ ] Review CHANGELOG.md — mark [${VERSION}] with today's date"
echo " [ ] Push the commit:    git push origin master"
echo " [ ] Push the tag:       git push origin ${TAG}"
if ! $DRY_RUN; then
    echo " [ ] Upload tarball:     dist/${TARBALL}"
fi
echo " [ ] Publish to crates.io (if desired): cargo publish"
echo "────────────────────────────────────────────────────────────────"
