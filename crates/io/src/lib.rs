use std::fs;
use std::path::Path;

pub fn read_text(path: impl AsRef<Path>) -> std::io::Result<String> {
    fs::read_to_string(path)
}
