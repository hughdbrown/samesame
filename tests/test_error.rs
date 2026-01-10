//! Tests for error types.

use samesame::error::SameError;
use std::io::{Error as IoError, ErrorKind};
use std::path::PathBuf;

#[test]
fn test_file_read_error_display() {
    let err = SameError::FileRead {
        path: PathBuf::from("/some/path/file.rs"),
        source: IoError::new(ErrorKind::NotFound, "file not found"),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("/some/path/file.rs"));
    assert!(msg.contains("file not found"));
}

#[test]
fn test_no_files_found_error_display() {
    let err = SameError::NoFilesFound;
    let msg = format!("{}", err);
    assert!(msg.contains("No files found"));
}

#[test]
fn test_invalid_glob_error_display() {
    let err = SameError::InvalidGlob {
        pattern: "**[invalid".to_string(),
        message: "unclosed bracket".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("**[invalid"));
    assert!(msg.contains("unclosed bracket"));
}

#[test]
fn test_error_debug() {
    let err = SameError::NoFilesFound;
    let debug = format!("{:?}", err);
    assert!(debug.contains("NoFilesFound"));
}
