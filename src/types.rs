//! Core data structures for the samesame application.

use std::path::PathBuf;

/// Represents a file's content as a sequence of line hashes.
#[derive(Debug)]
pub struct FileDescription {
    /// Path to the source file.
    pub filename: PathBuf,
    /// Hash of each line (after normalization).
    pub hashes: Vec<u64>,
    /// Original line content (for output display).
    pub lines: Vec<String>,
}

impl FileDescription {
    /// Returns the number of lines in the file.
    pub fn len(&self) -> usize {
        self.hashes.len()
    }

    /// Returns true if the file has no lines.
    pub fn is_empty(&self) -> bool {
        self.hashes.is_empty()
    }
}
