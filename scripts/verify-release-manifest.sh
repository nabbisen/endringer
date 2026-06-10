#!/usr/bin/env sh
# verify-release-manifest.sh — Verify an unpacked release archive.
#
# Usage:
#   sh scripts/verify-release-manifest.sh <archive-root>
#
# Run this against an unpacked release archive to confirm it contains
# everything listed in RELEASE-MANIFEST.md and nothing it should not.
#
# Also usable against the working tree (target/ check is informational there).

set -eu

ROOT="${1:-.}"
FAILED=0

ok()   { echo "  OK  $1"; }
fail() { echo "  FAIL $1" >&2; FAILED=1; }

require_file() { [ -f "$ROOT/$1" ] && ok "$1" || fail "missing file: $1"; }
require_dir()  { [ -d "$ROOT/$1" ] && ok "$1/" || fail "missing directory: $1/"; }
reject_dir()   {
    if [ -d "$ROOT/$1" ]; then
        echo "  FAIL directory must not be present: $1/" >&2
        FAILED=1
    else
        ok "(absent) $1/"
    fi
}

echo "Verifying release manifest for: $ROOT"
echo ""

echo "Required root files:"
require_file "Cargo.toml"
require_file "Cargo.lock"
require_file "README.md"
require_file "CHANGELOG.md"
require_file "ROADMAP.md"
require_file "RELEASE-MANIFEST.md"
require_file "LICENSE"
require_file "NOTICE"

echo ""
echo "Required directories:"
require_dir "crates"
require_dir "docs/src"
require_dir "rfcs"
require_dir "rfcs/proposed"
require_dir "rfcs/done"
require_dir "rfcs/archive"
require_file "rfcs/000-rfc-lifecycle-policy.md"
require_dir "scripts"

echo ""
echo "Must not be present:"
reject_dir "target"

echo ""
if [ "$FAILED" -eq 0 ]; then
    echo "Release manifest verification passed."
else
    echo "Verification FAILED — see errors above." >&2
    exit 1
fi
