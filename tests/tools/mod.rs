// Test modules for all tools
pub mod test_read_file;
pub mod test_write_file;

// Common test utilities
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Create a temporary directory for testing
pub fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Create a test file with content in the given directory
pub fn create_test_file(dir: &Path, filename: &str, content: &str) -> String {
    let file_path = dir.join(filename);
    fs::write(&file_path, content).expect("Failed to write test file");
    file_path.to_string_lossy().to_string()
}