//! Error types for the samesame application.

use std::path::PathBuf;
use thiserror::Error;

/// Application-level errors.
#[derive(Error, Debug)]
pub enum SameError {
    #[error("Failed to read file '{path}': {source}")]
    FileRead {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("No files found matching the specified patterns")]
    NoFilesFound,

    #[error("Invalid glob pattern '{pattern}': {message}")]
    InvalidGlob { pattern: String, message: String },
}

pub type Result<T> = std::result::Result<T, SameError>;
