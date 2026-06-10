//! Opaque identity types: [`CommitId`] and [`ObjectId`].
//!
//! Both types are backed by a `Vec<u8>` holding the raw hash bytes.
//! SHA-1 produces 20 bytes (40 hex chars); SHA-256 produces 32 bytes (64 hex
//! chars). No `gix` types are exposed.

// ── Shared hex helpers ────────────────────────────────────────────────────── //

pub(super) fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

pub(super) fn nibble_char(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        _ => (b'a' + n - 10) as char,
    }
}

fn decode_hex(hex: &str) -> Option<Vec<u8>> {
    let len = hex.len();
    if len != 40 && len != 64 {
        return None;
    }
    let mut bytes = Vec::with_capacity(len / 2);
    for chunk in hex.as_bytes().chunks(2) {
        let hi = hex_nibble(chunk[0])?;
        let lo = hex_nibble(chunk[1])?;
        bytes.push((hi << 4) | lo);
    }
    Some(bytes)
}

fn bytes_to_hex_string(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(nibble_char(b >> 4));
        out.push(nibble_char(b & 0xf));
    }
    out
}

fn short_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(7);
    for &b in bytes.iter().take(4) {
        out.push(nibble_char(b >> 4));
        out.push(nibble_char(b & 0xf));
    }
    out.truncate(7);
    out
}

// ── CommitId ─────────────────────────────────────────────────────────────── //

/// Opaque commit identifier, stored as raw bytes.
///
/// Supports both SHA-1 (20 bytes / 40 hex chars, used by Git) and SHA-256
/// (32 bytes / 64 hex chars, used by Jujutsu). No VCS library types are
/// exposed.
///
/// # Ordering
///
/// `CommitId` implements `Ord` via byte-level lexicographic comparison.
/// IDs produced by different hash algorithms (SHA-1 vs SHA-256) compare
/// consistently but not meaningfully across algorithms.
///
/// # Example
///
/// ```
/// # use endringer_core::types::CommitId;
/// let id = CommitId::from_hex("0000000000000000000000000000000000000000").unwrap();
/// assert_eq!(id.short().len(), 7);
/// println!("{id}");   // full hex string
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CommitId(Vec<u8>);

impl CommitId {
    /// Constructs a `CommitId` from raw bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        CommitId(bytes)
    }

    /// Constructs a `CommitId` by decoding a lowercase hex string.
    ///
    /// Accepts 40-character (SHA-1) or 64-character (SHA-256) hex strings.
    ///
    /// ```
    /// # use endringer_core::types::CommitId;
    /// assert!(CommitId::from_hex("0000000000000000000000000000000000000000").is_ok());
    /// assert!(CommitId::from_hex("not-a-hash").is_err());
    /// assert!(CommitId::from_hex("abc123").is_err()); // too short
    /// ```
    pub fn from_hex(hex: &str) -> Result<Self, CommitIdFromHexError> {
        decode_hex(hex)
            .map(CommitId)
            .ok_or_else(|| CommitIdFromHexError(hex.to_owned()))
    }

    /// Returns the raw bytes of this commit identifier.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns the first 7 hex characters — the conventional "short" form.
    pub fn short(&self) -> String {
        short_hex(&self.0)
    }

    /// Returns this commit id as a generic [`ObjectId`].
    ///
    /// A commit id is always a valid object id; this conversion is lossless.
    pub fn to_object_id(&self) -> ObjectId {
        ObjectId(self.0.clone())
    }
}

impl std::fmt::Display for CommitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&bytes_to_hex_string(&self.0))
    }
}

impl From<CommitId> for ObjectId {
    fn from(c: CommitId) -> Self {
        ObjectId(c.0)
    }
}

/// Error returned when [`CommitId::from_hex`] receives an invalid hex string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitIdFromHexError(pub(super) String);

impl std::fmt::Display for CommitIdFromHexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid commit id {:?}: expected 40 (SHA-1) or 64 (SHA-256) hex chars",
            self.0
        )
    }
}

impl std::error::Error for CommitIdFromHexError {}

// ── ObjectId ─────────────────────────────────────────────────────────────── //

/// Opaque identifier for any Git/jj object (blob, tree, commit, or tag),
/// stored as raw bytes.
///
/// Mirrors [`CommitId`] but is not restricted to commits. Use this type
/// for tree entries, ref targets, conflict stages, or any other context
/// where the object kind is not known to be a commit.
///
/// # Relationship to `CommitId`
///
/// - [`CommitId`] means "an id known to denote a commit."
/// - `ObjectId` means "an id that may denote any object kind."
/// - A `CommitId` is always a valid `ObjectId`; convert with
///   [`CommitId::to_object_id`] or `ObjectId::from(commit_id)`.
/// - An `ObjectId` is not necessarily a commit. When you have verified
///   (or assert) that it is one, use [`ObjectId::assume_commit`].
///
/// # No `gix` exposure
///
/// `gix::ObjectId` never appears in the public API.
///
/// # Ordering
///
/// Byte-level lexicographic comparison, identical to [`CommitId`].
///
/// # Example
///
/// ```
/// # use endringer_core::types::ObjectId;
/// let id = ObjectId::from_hex("0000000000000000000000000000000000000000").unwrap();
/// assert_eq!(id.short().len(), 7);
/// println!("{id}");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId(Vec<u8>);

impl ObjectId {
    /// Constructs an `ObjectId` from raw bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        ObjectId(bytes)
    }

    /// Constructs an `ObjectId` by decoding a hex string.
    ///
    /// Accepts 40-character (SHA-1) or 64-character (SHA-256) hex strings.
    ///
    /// ```
    /// # use endringer_core::types::ObjectId;
    /// assert!(ObjectId::from_hex("0000000000000000000000000000000000000000").is_ok());
    /// assert!(ObjectId::from_hex("bad").is_err());
    /// ```
    pub fn from_hex(hex: &str) -> Result<Self, ObjectIdFromHexError> {
        decode_hex(hex)
            .map(ObjectId)
            .ok_or_else(|| ObjectIdFromHexError(hex.to_owned()))
    }

    /// Returns the raw bytes of this object identifier.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns the first 7 hex characters — the conventional "short" form.
    pub fn short(&self) -> String {
        short_hex(&self.0)
    }

    /// Reinterprets this `ObjectId` as a [`CommitId`].
    ///
    /// The caller asserts (or has verified externally) that this object is a
    /// commit. `endringer` does not check the object kind here.
    ///
    /// Prefer [`Repository::find_commit`][crate] when kind verification is
    /// required.
    pub fn assume_commit(self) -> CommitId {
        CommitId(self.0)
    }
}

impl std::fmt::Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&bytes_to_hex_string(&self.0))
    }
}

/// Error returned when [`ObjectId::from_hex`] receives an invalid hex string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectIdFromHexError(pub(super) String);

impl std::fmt::Display for ObjectIdFromHexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid object id {:?}: expected 40 (SHA-1) or 64 (SHA-256) hex chars",
            self.0
        )
    }
}

impl std::error::Error for ObjectIdFromHexError {}

// ── Tests ─────────────────────────────────────────────────────────────────── //

#[cfg(test)]
mod tests {
    use super::*;

    const SHA1_HEX: &str = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
    const SHA256_HEX: &str =
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    // ── CommitId ──────────────────────────────────────────────────────── //

    #[test]
    fn commit_id_from_hex_sha1() {
        let id = CommitId::from_hex(SHA1_HEX).unwrap();
        assert_eq!(id.as_bytes().len(), 20);
        assert_eq!(id.to_string(), SHA1_HEX);
    }

    #[test]
    fn commit_id_from_hex_sha256() {
        let id = CommitId::from_hex(SHA256_HEX).unwrap();
        assert_eq!(id.as_bytes().len(), 32);
        assert_eq!(id.to_string(), SHA256_HEX);
    }

    #[test]
    fn commit_id_from_hex_invalid() {
        assert!(CommitId::from_hex("abc123").is_err());
        assert!(CommitId::from_hex("").is_err());
        assert!(CommitId::from_hex("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz").is_err());
    }

    #[test]
    fn commit_id_short_is_7_chars() {
        let id = CommitId::from_hex(SHA1_HEX).unwrap();
        assert_eq!(id.short().len(), 7);
        assert_eq!(&id.short(), &SHA1_HEX[..7]);
    }

    #[test]
    fn commit_id_roundtrip_bytes() {
        let id = CommitId::from_hex(SHA1_HEX).unwrap();
        let id2 = CommitId::from_bytes(id.as_bytes().to_vec());
        assert_eq!(id, id2);
    }

    // ── ObjectId ──────────────────────────────────────────────────────── //

    #[test]
    fn object_id_from_hex_sha1() {
        let id = ObjectId::from_hex(SHA1_HEX).unwrap();
        assert_eq!(id.as_bytes().len(), 20);
        assert_eq!(id.to_string(), SHA1_HEX);
    }

    #[test]
    fn object_id_from_hex_sha256() {
        let id = ObjectId::from_hex(SHA256_HEX).unwrap();
        assert_eq!(id.as_bytes().len(), 32);
        assert_eq!(id.to_string(), SHA256_HEX);
    }

    #[test]
    fn object_id_from_hex_invalid() {
        assert!(ObjectId::from_hex("abc123").is_err());
        assert!(ObjectId::from_hex("").is_err());
    }

    #[test]
    fn object_id_short_is_7_chars() {
        let id = ObjectId::from_hex(SHA1_HEX).unwrap();
        assert_eq!(id.short().len(), 7);
        assert_eq!(&id.short(), &SHA1_HEX[..7]);
    }

    // ── CommitId ↔ ObjectId conversions ──────────────────────────────── //

    #[test]
    fn commit_to_object_id_roundtrip() {
        let commit = CommitId::from_hex(SHA1_HEX).unwrap();
        let obj: ObjectId = commit.clone().into();
        assert_eq!(obj.as_bytes(), commit.as_bytes());
        assert_eq!(obj.to_string(), commit.to_string());
    }

    #[test]
    fn to_object_id_method() {
        let commit = CommitId::from_hex(SHA1_HEX).unwrap();
        let obj = commit.to_object_id();
        assert_eq!(obj.as_bytes(), commit.as_bytes());
    }

    #[test]
    fn assume_commit_roundtrip() {
        let commit = CommitId::from_hex(SHA1_HEX).unwrap();
        let obj = ObjectId::from(commit.clone());
        let back = obj.assume_commit();
        assert_eq!(back, commit);
    }

    #[test]
    fn sha1_and_sha256_never_equal() {
        let sha1 = CommitId::from_hex(SHA1_HEX).unwrap();
        let sha256 = CommitId::from_hex(SHA256_HEX).unwrap();
        assert_ne!(sha1, sha256);
        let sha1_obj = ObjectId::from_hex(SHA1_HEX).unwrap();
        let sha256_obj = ObjectId::from_hex(SHA256_HEX).unwrap();
        assert_ne!(sha1_obj, sha256_obj);
    }

    #[test]
    fn commit_id_ord() {
        let a = CommitId::from_hex("0000000000000000000000000000000000000001").unwrap();
        let b = CommitId::from_hex("0000000000000000000000000000000000000002").unwrap();
        assert!(a < b);
    }

    #[test]
    fn object_id_ord() {
        let a = ObjectId::from_hex("0000000000000000000000000000000000000001").unwrap();
        let b = ObjectId::from_hex("0000000000000000000000000000000000000002").unwrap();
        assert!(a < b);
    }
}
