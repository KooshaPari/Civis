//! I/O utilities for simulation state persistence

use std::fs;
use std::path::Path;

/// Read text file contents
pub fn read_text(path: impl AsRef<Path>) -> std::io::Result<String> {
    fs::read_to_string(path)
}

/// Write text file contents
pub fn write_text(path: impl AsRef<Path>, contents: &str) -> std::io::Result<()> {
    fs::write(path, contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_write() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test content").unwrap();

        let contents = read_text(file.path()).unwrap();
        assert_eq!(contents, "test content");
    }
}
