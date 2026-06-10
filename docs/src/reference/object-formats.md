# Object format support

Git supports two object-storage hash algorithms. This page documents
endringer's support for each.

## SHA-1 (standard)

SHA-1 is the default Git object format. All endringer features are supported
and tested on SHA-1 repositories.

- `CommitId` stores SHA-1 OIDs as 20 bytes / 40 hex characters.
- `ObjectId` stores SHA-1 OIDs the same way.
- `CommitId::from_hex` accepts exactly 40 hex characters for SHA-1.

## SHA-256 (experimental)

SHA-256 (`--object-format=sha256`) is a newer Git format. Support status:

| Area | Status |
|---|---|
| `CommitId` storage | Supported — stores 32 bytes / 64 hex characters |
| `CommitId::from_hex` for 64-char hex | Supported — accepted, stored correctly |
| Git SHA-256 repository reads | Supported where `gix` reads them; tested if fixture can be created |
| jj SHA-256 repositories | **Not supported** — jj SHA-256 stores are not verified |
| SHA-256 in `repository_info().object_format` | Reported as `ObjectFormat::Sha256` |

### Creating a SHA-256 fixture

A SHA-256 Git repository requires a git binary that supports
`--object-format=sha256` (git ≥ 2.29). Tests that need this fixture
call `require_sha256_git()` and skip gracefully when the installed git
does not support it.

### CommitId behaviour across formats

SHA-1 and SHA-256 `CommitId` values are never equal, even if they happen
to share the same bytes, because the lengths differ. Ordering across
algorithms is consistent (byte-level lexicographic) but semantically
meaningless. These properties are tested in `endringer-core`'s unit tests.

## ObjectFormat in repository_info

```rust,no_run
use endringer::{repository, ObjectFormat};

let info = repository(path)?.repository_info()?;
match info.object_format {
    ObjectFormat::Sha1    => println!("SHA-1 repo"),
    ObjectFormat::Sha256  => println!("SHA-256 repo"),
    ObjectFormat::Unknown(s) => println!("unknown format: {s}"),
}
```

## jj SHA-256 stance

jj SHA-256 support is explicitly **not** claimed. endringer's jj backend
opens the underlying git object store; if that store uses SHA-256, behavior
is undefined and an error is likely. This will be revisited when RFC 007
verification is extended to SHA-256 jj repositories.
