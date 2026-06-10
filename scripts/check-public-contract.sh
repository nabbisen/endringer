#!/usr/bin/env sh
# check-public-contract.sh — Verify that known-stale public claims are absent.
#
# Run from the repository root:
#   sh scripts/check-public-contract.sh
#
# Exit codes:
#   0  — all checks passed
#   1  — at least one stale claim was found; output shows the locations
#
# This script is intentionally small and grep-based. It enforces the
# high-signal contracts documented in docs/src/reference/contract.md and
# catches the specific stale phrases that have been incorrect in the past.
# It is not a full documentation linter.

set -eu

ROOT="${1:-.}"
FAILED=0

fail() {
    echo "FAIL: $1" >&2
    FAILED=1
}

check_absent() {
    phrase="$1"
    description="$2"
    # Exclude: this script itself, rfcs/ (which may quote old phrases as
    # motivation), docs/legacy/ (archived pre-mdbook content), CHANGELOG.md
    # (records historical behavior accurately — stale claims belong in current
    # docs/code, not in the historical record).
    if grep -rn --include="*.rs" --include="*.md" \
            --exclude-dir=".git" --exclude-dir="target" \
            --exclude-dir="rfcs" --exclude-dir="legacy" \
            --exclude="CHANGELOG.md" \
            "$phrase" "$ROOT" 2>/dev/null \
       | grep -qv "check-public-contract.sh"; then
        echo "---" >&2
        echo "Stale claim found: $description" >&2
        grep -rn --include="*.rs" --include="*.md" \
            --exclude-dir=".git" --exclude-dir="target" \
            --exclude-dir="rfcs" --exclude-dir="legacy" \
            --exclude="CHANGELOG.md" \
            "$phrase" "$ROOT" 2>/dev/null \
        | grep -v "check-public-contract.sh" >&2
        fail "$description"
    fi
}

check_present() {
    path="$1"
    description="$2"
    if [ ! -e "$ROOT/$path" ]; then
        fail "Required file/directory missing: $path ($description)"
    fi
}

echo "Running public contract checks..."

# ── Stale claims that must not reappear ───────────────────────────────────── #

check_absent \
    "gitignore rules are not applied" \
    "gitignore-not-applied claim (gitignore IS applied since v0.17.0)"

check_absent \
    "Gitignore rules are not applied" \
    "gitignore-not-applied claim (capitalised variant)"

check_absent \
    "falls back to a lightweight tag" \
    "jj annotated-tag fallback claim (jj returns an error, not a fallback)"

check_absent \
    "creates a lightweight tag and ignores the message" \
    "jj annotated-tag silent-fallback claim"

# ── Required files ────────────────────────────────────────────────────────── #

check_present "rfcs/000-rfc-lifecycle-policy.md" "RFC lifecycle policy"
check_present "rfcs/proposed" "RFC proposed/ directory"
check_present "rfcs/done" "RFC done/ directory"
check_present "rfcs/archive" "RFC archive/ directory"
check_present "docs/src/reference/contract.md" "public contract statements"
check_present "RELEASE-MANIFEST.md" "release manifest"
check_present "CHANGELOG.md" "changelog"
check_present "ROADMAP.md" "roadmap"

# ── Packaging safety ─────────────────────────────────────────────────────── #

if [ -d "$ROOT/target" ]; then
    echo "NOTE: target/ directory present (expected in working tree; must be excluded from archives)."
fi

# ── Result ────────────────────────────────────────────────────────────────── #

if [ "$FAILED" -eq 0 ]; then
    echo "All contract checks passed."
else
    echo ""
    echo "One or more checks failed. See output above." >&2
    exit 1
fi
