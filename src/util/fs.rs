use std::path::Path;

/// Maximum allowable source file size cap before staging triggers a safety warning.
pub const LARGE_FILE_THRESHOLD_BYTES: u64 = 5 * 1024 * 1024; // 5 MiB

/// Reads metadata parameters to establish file length attributes.
pub fn file_size(path: &Path) -> Option<u64> {
    std::fs::metadata(path).ok().map(|m| m.len())
}

/// Evaluates if a specified resource size crosses the large data file limit.
pub fn is_large_file(path: &Path) -> bool {
    file_size(path)
        .map(|size| size > LARGE_FILE_THRESHOLD_BYTES)
        .unwrap_or(false)
}
