//! Rolling hash duplicate detection algorithm.
//!
//! Replaces patience diff + union-find with a rolling XOR hash approach.
//! Computes rolling hash fingerprints over windows of `min_match` consecutive
//! lines and uses a hash table for O(1) duplicate block lookup.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Maps file paths to compact sequential numbers and back.
///
/// File numbers are 0-based and assigned in registration order.
#[derive(Debug)]
pub struct FileRegistry {
    name_to_num: HashMap<PathBuf, usize>,
    num_to_name: Vec<PathBuf>,
}

impl FileRegistry {
    /// Creates an empty file registry.
    pub fn new() -> Self {
        Self {
            name_to_num: HashMap::new(),
            num_to_name: Vec::new(),
        }
    }

    /// Registers a file path and returns its assigned number.
    /// If the path is already registered, returns the existing number.
    pub fn register(&mut self, path: PathBuf) -> usize {
        if let Some(&num) = self.name_to_num.get(&path) {
            return num;
        }
        let num = self.num_to_name.len();
        self.num_to_name.push(path.clone());
        self.name_to_num.insert(path, num);
        num
    }

    /// Returns the path for a given file number.
    pub fn get_path(&self, file_num: usize) -> &Path {
        &self.num_to_name[file_num]
    }

    /// Returns the number of registered files.
    pub fn len(&self) -> usize {
        self.num_to_name.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_registry_assigns_sequential_numbers() {
        let mut reg = FileRegistry::new();
        assert_eq!(reg.register(PathBuf::from("a.rs")), 0);
        assert_eq!(reg.register(PathBuf::from("b.rs")), 1);
        assert_eq!(reg.register(PathBuf::from("c.rs")), 2);
    }

    #[test]
    fn test_file_registry_get_path() {
        let mut reg = FileRegistry::new();
        reg.register(PathBuf::from("src/main.rs"));
        reg.register(PathBuf::from("src/lib.rs"));
        assert_eq!(reg.get_path(0), Path::new("src/main.rs"));
        assert_eq!(reg.get_path(1), Path::new("src/lib.rs"));
    }

    #[test]
    fn test_file_registry_dedup() {
        let mut reg = FileRegistry::new();
        let first = reg.register(PathBuf::from("a.rs"));
        let second = reg.register(PathBuf::from("a.rs"));
        assert_eq!(first, second);
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn test_file_registry_len() {
        let mut reg = FileRegistry::new();
        assert_eq!(reg.len(), 0);
        reg.register(PathBuf::from("a.rs"));
        assert_eq!(reg.len(), 1);
        reg.register(PathBuf::from("b.rs"));
        assert_eq!(reg.len(), 2);
        reg.register(PathBuf::from("a.rs")); // duplicate
        assert_eq!(reg.len(), 2);
    }
}
