use std::fs;
use std::path::Path;

pub fn read_text(path: impl AsRef<Path>) -> std::io::Result<String> {
    fs::read_to_string(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn read_text_succeeds_with_valid_file() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "Hello, World!").unwrap();
        
        let result = read_text(file.path()).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn read_text_fails_with_nonexistent_file() {
        let result = read_text("/this/path/does/not/exist.txt");
        assert!(result.is_err());
    }

    #[test]
    fn read_text_handles_empty_files() {
        let file = NamedTempFile::new().unwrap();
        let result = read_text(file.path()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn read_text_preserves_multiline_content() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Line 1").unwrap();
        writeln!(file, "Line 2").unwrap();
        writeln!(file, "Line 3").unwrap();
        
        let result = read_text(file.path()).unwrap();
        assert!(result.contains("Line 1"));
        assert!(result.contains("Line 2"));
        assert!(result.contains("Line 3"));
    }
}
