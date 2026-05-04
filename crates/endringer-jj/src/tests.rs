use std::path::Path;
use super::backend::JjBackend;

#[test]
fn rejects_non_jj_path() {
    // A plain directory (or missing path) is not a jj repo.
    assert!(JjBackend::open(Path::new("/tmp")).is_err());
    assert!(JjBackend::open(Path::new("/no/such/dir")).is_err());
}
