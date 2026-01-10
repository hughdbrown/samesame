//! Core data structures for the samesame application.

use serde::Serialize;
use std::path::PathBuf;

/// Represents a contiguous range of lines in a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Range {
    /// Starting line index (0-based, inclusive)
    pub start: usize,
    /// Ending line index (0-based, exclusive)
    pub end: usize,
}

impl Range {
    /// Creates a new range.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Returns the number of lines in this range.
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Returns true if the range is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

/// Represents a comparison result between two files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineRange {
    /// Lines that are identical between both files.
    Same { r1: Range, r2: Range },
    /// Lines that differ between files.
    Diff { r1: Range, r2: Range },
}

impl LineRange {
    /// Returns the range for the first file.
    #[allow(dead_code)]
    pub fn r1(&self) -> &Range {
        match self {
            LineRange::Same { r1, .. } => r1,
            LineRange::Diff { r1, .. } => r1,
        }
    }

    /// Returns the range for the second file.
    #[allow(dead_code)]
    pub fn r2(&self) -> &Range {
        match self {
            LineRange::Same { r2, .. } => r2,
            LineRange::Diff { r2, .. } => r2,
        }
    }

    /// Returns true if this is a Same variant.
    pub fn is_same(&self) -> bool {
        matches!(self, LineRange::Same { .. })
    }

    /// Returns the length of the match (for Same variants).
    pub fn match_len(&self) -> usize {
        match self {
            LineRange::Same { r1, .. } => r1.len(),
            LineRange::Diff { .. } => 0,
        }
    }
}

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
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.hashes.len()
    }

    /// Returns true if the file has no lines.
    pub fn is_empty(&self) -> bool {
        self.hashes.is_empty()
    }
}

/// Complete comparison result between two files.
#[derive(Debug)]
pub struct ComparisonResult<'a> {
    /// Reference to first file.
    pub f1: &'a FileDescription,
    /// Reference to second file.
    pub f2: &'a FileDescription,
    /// Sequence of matching/differing line ranges.
    pub runs: Vec<LineRange>,
}

impl<'a> ComparisonResult<'a> {
    /// Returns only the matching ranges that meet the minimum length threshold.
    pub fn significant_matches(&self, min_lines: usize) -> Vec<&LineRange> {
        self.runs
            .iter()
            .filter(|lr| lr.is_same() && lr.match_len() >= min_lines)
            .collect()
    }

    /// Returns true if there are any significant matches.
    pub fn has_significant_matches(&self, min_lines: usize) -> bool {
        self.runs
            .iter()
            .any(|lr| lr.is_same() && lr.match_len() >= min_lines)
    }
}
