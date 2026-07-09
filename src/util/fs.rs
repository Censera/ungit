use std::path::Path;

/// Size threshold above which `commands::save` warns before committing a
/// file, to catch accidental large-binary commits.
pub const LARGE_FILE_THRESHOLD_BYTES: u64 = 5 * 1024 * 1024; // 5 MiB

/// Size of `path` in bytes, or `None` if it can't be read (e.g. deleted
/// between listing and checking, or a symlink to nowhere).
pub fn file_size(path: &Path) -> Option<u64> {
    std::fs::metadata(path).ok().map(|m| m.len())
}

/// True if `path`'s size exceeds `LARGE_FILE_THRESHOLD_BYTES`.
pub fn is_large_file(path: &Path) -> bool {
    file_size(path)
        .map(|size| size > LARGE_FILE_THRESHOLD_BYTES)
        .unwrap_or(false)
}
