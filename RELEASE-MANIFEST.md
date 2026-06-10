# endringer release manifest

**Version.** v0.30.0
**Archive shape.** The release archive extracts with no intermediate parent
directory. Files unpack directly into the extraction root (e.g.
`tar -xf endringer-0.20.0.tar.gz` produces `Cargo.toml`, `crates/`, etc. in
the current directory).

---

## Required root files

- `Cargo.toml` (workspace manifest)
- `Cargo.lock`
- `README.md`
- `CHANGELOG.md`
- `ROADMAP.md`
- `RELEASE-MANIFEST.md` (this file)
- `LICENSE`
- `NOTICE`
- `TERMS_OF_USE.md`

## Required directories

- `crates/` (five workspace crates)
- `docs/src/` (mdBook documentation source)
- `rfcs/` (RFC lifecycle directory, including `000-rfc-lifecycle-policy.md`
  and `proposed/`, `done/`, `archive/` subdirectories)
- `scripts/` (release and CI helper scripts)

## Excluded from archives

The following must **not** appear in release archives:

- `target/` (build artefacts)
- `.git/` (version-control metadata)
- editor backup files (`*.bak`, `*~`, `*.swp`)
- temporary test repositories

## Verification command

```sh
sh scripts/verify-release-manifest.sh <unpacked-archive-root>
```

Or from the working tree before archiving:

```sh
sh scripts/check-public-contract.sh
```

---

## Crates and publish order

Publish in dependency order (each crate must be published before the crates
that depend on it):

1. `endringer-core`
2. `endringer-git`
3. `endringer-jj`
4. `endringer`
5. `endringer-async`

---

## Version synchronisation

All five crates share the same version number, defined in the workspace
`Cargo.toml` under `[workspace.package]`. Bump there, then verify with:

```sh
grep '^version' Cargo.toml crates/*/Cargo.toml
```
