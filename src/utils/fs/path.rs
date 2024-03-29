use std::path::{Path, PathBuf};

pub fn hash_size_to_path<P: AsRef<Path>>(root_path: P, hash: u64, size: u64) -> PathBuf {
    let bytes = hash.to_le_bytes();
    // avoid putting to many files in a single directory
    let first = hex::encode(&bytes[..1]);
    let second = hex::encode(&bytes[1..]);
    root_path.as_ref().join(first).join(second).join(size.to_string())
}
