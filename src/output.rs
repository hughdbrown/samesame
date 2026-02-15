//! Output formatting for duplicate detection results.

use serde::Serialize;

use crate::types::{ComparisonResult, LineRange, Range};

/// A single match between two files for JSON output.
#[derive(Serialize)]
pub struct MatchInfo {
    pub lines: usize,
    pub file1_range: RangeInfo,
    pub file2_range: RangeInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<String>>,
}

/// Range information for JSON output (1-based line numbers for user display).
#[derive(Serialize)]
pub struct RangeInfo {
    pub start: usize,
    pub end: usize,
}

impl From<&Range> for RangeInfo {
    fn from(r: &Range) -> Self {
        // Convert to 1-based line numbers for display
        RangeInfo {
            start: r.start + 1,
            end: r.end,
        }
    }
}

/// A duplicate found between two files for JSON output.
#[derive(Serialize)]
pub struct DuplicateInfo {
    pub file1: String,
    pub file2: String,
    pub matches: Vec<MatchInfo>,
}

/// Summary statistics for JSON output.
#[derive(Serialize)]
pub struct Summary {
    pub files_analyzed: usize,
    pub pairs_compared: usize,
    pub duplicates_found: usize,
    pub total_duplicate_lines: usize,
}

/// Complete JSON output structure.
#[derive(Serialize)]
pub struct JsonOutput {
    pub version: String,
    pub summary: Summary,
    pub duplicates: Vec<DuplicateInfo>,
}

/// Format results as human-readable text.
pub fn format_text(
    results: &[ComparisonResult<'_>],
    min_match: usize,
    verbose: bool,
    files_count: usize,
    pairs_count: usize,
) -> String {
    let mut output = String::new();

    let significant_results: Vec<_> = results
        .iter()
        .filter(|r| r.has_significant_matches(min_match))
        .collect();

    if significant_results.is_empty() {
        output.push_str("No duplicate code found.\n");
        return output;
    }

    let mut total_duplicate_lines = 0usize;
    let mut total_matches = 0usize;

    let mut matches_output = String::new();

    for result in &significant_results {
        let matches = result.significant_matches(min_match);

        for m in &matches {
            if let LineRange::Same { r1, r2 } = m {
                total_duplicate_lines += r1.len();
                total_matches += 1;

                matches_output.push_str(&format!(
                    "Files: {} <-> {}\n",
                    result.f1.filename.display(),
                    result.f2.filename.display()
                ));

                matches_output.push_str(&format!(
                    "Match: {} lines ({}:{}-{}, {}:{}-{})\n",
                    r1.len(),
                    result.f1.filename.file_name().unwrap_or_default().to_string_lossy(),
                    r1.start + 1,
                    r1.end,
                    result.f2.filename.file_name().unwrap_or_default().to_string_lossy(),
                    r2.start + 1,
                    r2.end,
                ));

                if verbose {
                    matches_output.push('\n');
                    let offset2 = r2.start as isize - r1.start as isize;
                    for i in r1.start..r1.end {
                        let j = (i as isize + offset2) as usize;
                        if i < result.f1.lines.len() {
                            matches_output.push_str(&format!(
                                "  {:>4} | {:>4} | {}\n",
                                i + 1,
                                j + 1,
                                result.f1.lines[i]
                            ));
                        }
                    }
                }

                matches_output.push_str("\n---\n\n");
            }
        }
    }

    if total_matches == 0 {
        output.push_str("No duplicate code found.\n");
        return output;
    }

    output.push_str("=== Duplicate Code Found ===\n\n");
    output.push_str(&matches_output);

    output.push_str(&format!(
        "Summary: {} files analyzed, {} pairs compared, {} duplicate regions ({} lines)\n",
        files_count,
        pairs_count,
        total_matches,
        total_duplicate_lines,
    ));

    output
}

/// Format results as JSON.
pub fn format_json(
    results: &[ComparisonResult<'_>],
    min_match: usize,
    verbose: bool,
    files_count: usize,
    pairs_count: usize,
) -> String {
    let mut duplicates = Vec::new();
    let mut total_duplicate_lines = 0usize;

    for result in results {
        let matches = result.significant_matches(min_match);

        if matches.is_empty() {
            continue;
        }

        let match_infos: Vec<MatchInfo> = matches
            .iter()
            .filter_map(|m| {
                if let LineRange::Same { r1, r2 } = m {
                    total_duplicate_lines += r1.len();

                    let content = if verbose {
                        Some(result.f1.lines[r1.start..r1.end].to_vec())
                    } else {
                        None
                    };

                    Some(MatchInfo {
                        lines: r1.len(),
                        file1_range: r1.into(),
                        file2_range: r2.into(),
                        content,
                    })
                } else {
                    None
                }
            })
            .collect();

        if !match_infos.is_empty() {
            duplicates.push(DuplicateInfo {
                file1: result.f1.filename.display().to_string(),
                file2: result.f2.filename.display().to_string(),
                matches: match_infos,
            });
        }
    }

    let output = JsonOutput {
        version: env!("CARGO_PKG_VERSION").to_string(),
        summary: Summary {
            files_analyzed: files_count,
            pairs_compared: pairs_count,
            duplicates_found: duplicates.len(),
            total_duplicate_lines,
        },
        duplicates,
    };

    serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
}
