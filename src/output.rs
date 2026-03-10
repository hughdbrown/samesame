//! Output formatting for duplicate detection results.

use std::fmt::Write;

use serde::Serialize;

use crate::rolling_hash::DuplicateGroup;

/// Location within a duplicate group, for JSON output.
#[derive(Serialize)]
pub struct LocationInfo {
    pub file: String,
    pub start: usize,
    pub end: usize,
}

/// A duplicate group for JSON output.
#[derive(Serialize)]
pub struct GroupInfo {
    pub lines: usize,
    pub locations: Vec<LocationInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<String>>,
}

/// Summary statistics for JSON output.
#[derive(Serialize)]
pub struct Summary {
    pub files_analyzed: usize,
    pub duplicate_groups: usize,
    pub total_duplicate_lines: usize,
}

/// Complete JSON output structure.
#[derive(Serialize)]
pub struct JsonOutput {
    pub version: String,
    pub summary: Summary,
    pub duplicates: Vec<GroupInfo>,
}

/// Format results as human-readable text.
pub fn format_text(groups: &[DuplicateGroup], verbose: bool, files_count: usize) -> String {
    let mut output = String::new();

    if groups.is_empty() {
        output.push_str("No duplicate code found.\n");
        return output;
    }

    output.push_str("=== Duplicate Code Found ===\n\n");

    let mut total_duplicate_lines = 0usize;

    for group in groups {
        total_duplicate_lines += group.line_count;

        let _ = writeln!(
            output,
            "{} lines duplicated across {} files:",
            group.line_count,
            group.locations.len(),
        );

        for (path, start, end) in &group.locations {
            let _ = writeln!(output, "  {}  lines {}-{}", path.display(), start + 1, end);
        }

        if verbose && let Some(ref content) = group.content {
            let start_line = group.locations[0].1;
            output.push('\n');
            for (i, line) in content.iter().enumerate() {
                let _ = writeln!(output, "  {:>4} | {}", start_line + i + 1, line);
            }
        }

        output.push_str("\n---\n\n");
    }

    let _ = writeln!(
        output,
        "Summary: {} files analyzed, {} duplicate groups ({} lines)",
        files_count,
        groups.len(),
        total_duplicate_lines,
    );

    output
}

/// Format results as JSON.
pub fn format_json(groups: &[DuplicateGroup], verbose: bool, files_count: usize) -> String {
    let mut total_duplicate_lines = 0usize;

    let duplicates: Vec<GroupInfo> = groups
        .iter()
        .map(|group| {
            total_duplicate_lines += group.line_count;

            let locations: Vec<LocationInfo> = group
                .locations
                .iter()
                .map(|(path, start, end)| LocationInfo {
                    file: path.display().to_string(),
                    start: start + 1, // 1-based for display
                    end: *end,
                })
                .collect();

            let content = if verbose { group.content.clone() } else { None };

            GroupInfo {
                lines: group.line_count,
                locations,
                content,
            }
        })
        .collect();

    let output = JsonOutput {
        version: env!("CARGO_PKG_VERSION").to_string(),
        summary: Summary {
            files_analyzed: files_count,
            duplicate_groups: duplicates.len(),
            total_duplicate_lines,
        },
        duplicates,
    };

    serde_json::to_string_pretty(&output).expect("JSON serialization of known types")
}
