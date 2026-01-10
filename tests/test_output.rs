//! Tests for output formatting.

use regex::Regex;
use samesame::output::{format_json, format_text, RangeInfo};
use samesame::types::{ComparisonResult, FileDescription, LineRange, Range};
use std::path::PathBuf;

#[test]
fn test_range_info_conversion() {
    let r = Range::new(0, 5);
    let info: RangeInfo = (&r).into();

    // Should be 1-based
    assert_eq!(info.start, 1);
    assert_eq!(info.end, 5);
}

#[test]
fn test_range_info_conversion_nonzero_start() {
    let r = Range::new(10, 20);
    let info: RangeInfo = (&r).into();

    assert_eq!(info.start, 11);
    assert_eq!(info.end, 20);
}

fn make_test_files() -> (FileDescription, FileDescription) {
    let f1 = FileDescription {
        filename: PathBuf::from("src/file1.rs"),
        hashes: vec![1, 2, 3, 4, 5],
        lines: vec![
            "fn main() {".into(),
            "    println!(\"Hello\");".into(),
            "}".into(),
            "".into(),
            "fn other() {}".into(),
        ],
    };
    let f2 = FileDescription {
        filename: PathBuf::from("src/file2.rs"),
        hashes: vec![1, 2, 3, 6, 7],
        lines: vec![
            "fn main() {".into(),
            "    println!(\"Hello\");".into(),
            "}".into(),
            "".into(),
            "fn different() {}".into(),
        ],
    };
    (f1, f2)
}

#[test]
fn test_format_text_no_duplicates() {
    let (f1, f2) = make_test_files();
    let result = ComparisonResult {
        f1: &f1,
        f2: &f2,
        runs: vec![LineRange::Diff {
            r1: Range::new(0, 5),
            r2: Range::new(0, 5),
        }],
    };

    let output = format_text(&[result], 5, false, 2, 1, None);
    assert!(output.contains("No duplicate code found"));
}

#[test]
fn test_format_text_with_duplicates() {
    let (f1, f2) = make_test_files();
    let result = ComparisonResult {
        f1: &f1,
        f2: &f2,
        runs: vec![LineRange::Same {
            r1: Range::new(0, 3),
            r2: Range::new(0, 3),
        }],
    };

    let output = format_text(&[result], 3, false, 2, 1, None);
    assert!(output.contains("Duplicate Code Found"));
    assert!(output.contains("file1.rs"));
    assert!(output.contains("file2.rs"));
    assert!(output.contains("3 lines"));
    assert!(output.contains("Summary:"));
}

#[test]
fn test_format_text_verbose() {
    let (f1, f2) = make_test_files();
    let result = ComparisonResult {
        f1: &f1,
        f2: &f2,
        runs: vec![LineRange::Same {
            r1: Range::new(0, 3),
            r2: Range::new(0, 3),
        }],
    };

    let output = format_text(&[result], 3, true, 2, 1, None);
    assert!(output.contains("fn main()"));
    assert!(output.contains("println!"));
}

#[test]
fn test_format_text_below_threshold() {
    let (f1, f2) = make_test_files();
    let result = ComparisonResult {
        f1: &f1,
        f2: &f2,
        runs: vec![LineRange::Same {
            r1: Range::new(0, 3),
            r2: Range::new(0, 3),
        }],
    };

    // Threshold is 5, but match is only 3 lines
    let output = format_text(&[result], 5, false, 2, 1, None);
    assert!(output.contains("No duplicate code found"));
}

#[test]
fn test_format_json_no_duplicates() {
    let (f1, f2) = make_test_files();
    let result = ComparisonResult {
        f1: &f1,
        f2: &f2,
        runs: vec![LineRange::Diff {
            r1: Range::new(0, 5),
            r2: Range::new(0, 5),
        }],
    };

    let output = format_json(&[result], 5, false, 2, 1, None);
    assert!(output.contains("\"duplicates_found\": 0"));
    assert!(output.contains("\"duplicates\": []"));
}

#[test]
fn test_format_json_with_duplicates() {
    let (f1, f2) = make_test_files();
    let result = ComparisonResult {
        f1: &f1,
        f2: &f2,
        runs: vec![LineRange::Same {
            r1: Range::new(0, 3),
            r2: Range::new(0, 3),
        }],
    };

    let output = format_json(&[result], 3, false, 2, 1, None);
    assert!(output.contains("\"duplicates_found\": 1"));
    assert!(output.contains("\"file1\": \"src/file1.rs\""));
    assert!(output.contains("\"file2\": \"src/file2.rs\""));
    assert!(output.contains("\"lines\": 3"));
}

#[test]
fn test_format_json_verbose_includes_content() {
    let (f1, f2) = make_test_files();
    let result = ComparisonResult {
        f1: &f1,
        f2: &f2,
        runs: vec![LineRange::Same {
            r1: Range::new(0, 3),
            r2: Range::new(0, 3),
        }],
    };

    let output = format_json(&[result], 3, true, 2, 1, None);
    assert!(output.contains("\"content\":"));
    assert!(output.contains("fn main()"));
}

#[test]
fn test_format_json_not_verbose_no_content() {
    let (f1, f2) = make_test_files();
    let result = ComparisonResult {
        f1: &f1,
        f2: &f2,
        runs: vec![LineRange::Same {
            r1: Range::new(0, 3),
            r2: Range::new(0, 3),
        }],
    };

    let output = format_json(&[result], 3, false, 2, 1, None);
    // content field should be skipped when None
    assert!(!output.contains("\"content\":"));
}

#[test]
fn test_format_json_structure() {
    let (f1, f2) = make_test_files();
    let result = ComparisonResult {
        f1: &f1,
        f2: &f2,
        runs: vec![LineRange::Same {
            r1: Range::new(0, 3),
            r2: Range::new(0, 3),
        }],
    };

    let output = format_json(&[result], 3, false, 2, 1, None);

    // Parse as JSON to verify structure
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(parsed["version"].is_string());
    assert!(parsed["summary"]["files_analyzed"].is_number());
    assert!(parsed["summary"]["pairs_compared"].is_number());
    assert!(parsed["duplicates"].is_array());
}

#[test]
fn test_format_text_multiple_results() {
    let f1 = FileDescription {
        filename: PathBuf::from("a.rs"),
        hashes: vec![1, 2, 3],
        lines: vec!["a".into(), "b".into(), "c".into()],
    };
    let f2 = FileDescription {
        filename: PathBuf::from("b.rs"),
        hashes: vec![1, 2, 3],
        lines: vec!["a".into(), "b".into(), "c".into()],
    };
    let f3 = FileDescription {
        filename: PathBuf::from("c.rs"),
        hashes: vec![1, 2, 3],
        lines: vec!["a".into(), "b".into(), "c".into()],
    };

    let result1 = ComparisonResult {
        f1: &f1,
        f2: &f2,
        runs: vec![LineRange::Same {
            r1: Range::new(0, 3),
            r2: Range::new(0, 3),
        }],
    };
    let result2 = ComparisonResult {
        f1: &f1,
        f2: &f3,
        runs: vec![LineRange::Same {
            r1: Range::new(0, 3),
            r2: Range::new(0, 3),
        }],
    };

    let output = format_text(&[result1, result2], 3, false, 3, 3, None);
    assert!(output.contains("a.rs"));
    assert!(output.contains("b.rs"));
    assert!(output.contains("c.rs"));
    assert!(output.contains("2 duplicate regions"));
}

#[test]
fn test_format_json_multiple_matches_same_pair() {
    let (f1, f2) = make_test_files();
    let result = ComparisonResult {
        f1: &f1,
        f2: &f2,
        runs: vec![
            LineRange::Same {
                r1: Range::new(0, 2),
                r2: Range::new(0, 2),
            },
            LineRange::Diff {
                r1: Range::new(2, 3),
                r2: Range::new(2, 3),
            },
            LineRange::Same {
                r1: Range::new(3, 5),
                r2: Range::new(3, 5),
            },
        ],
    };

    let output = format_json(&[result], 2, false, 2, 1, None);
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    // Should have 2 matches in the matches array
    let matches = &parsed["duplicates"][0]["matches"];
    assert_eq!(matches.as_array().unwrap().len(), 2);
}

// ==================== Regex filtering tests ====================

fn make_mixed_files() -> (FileDescription, FileDescription) {
    let f1 = FileDescription {
        filename: PathBuf::from("src/file1.py"),
        hashes: vec![1, 2, 3, 4, 5, 6, 7, 8],
        lines: vec![
            "def hello():".into(),
            "    print(\"Hello\")".into(),
            "    return None".into(),
            "".into(),
            "class MyClass:".into(),
            "    def __init__(self):".into(),
            "        pass".into(),
            "".into(),
        ],
    };
    let f2 = FileDescription {
        filename: PathBuf::from("src/file2.py"),
        hashes: vec![1, 2, 3, 4, 5, 6, 7, 8],
        lines: vec![
            "def hello():".into(),
            "    print(\"Hello\")".into(),
            "    return None".into(),
            "".into(),
            "class MyClass:".into(),
            "    def __init__(self):".into(),
            "        pass".into(),
            "".into(),
        ],
    };
    (f1, f2)
}

#[test]
fn test_format_text_regex_filters_matches() {
    let (f1, f2) = make_mixed_files();
    let result = ComparisonResult {
        f1: &f1,
        f2: &f2,
        runs: vec![
            // Match starting with "def hello():"
            LineRange::Same {
                r1: Range::new(0, 3),
                r2: Range::new(0, 3),
            },
            LineRange::Diff {
                r1: Range::new(3, 4),
                r2: Range::new(3, 4),
            },
            // Match starting with "class MyClass:"
            LineRange::Same {
                r1: Range::new(4, 7),
                r2: Range::new(4, 7),
            },
        ],
    };

    // Filter to only show matches starting with "def"
    let regex = Regex::new(r"^def ").unwrap();
    let output = format_text(&[result], 3, false, 2, 1, Some(&regex));

    assert!(output.contains("Duplicate Code Found"));
    assert!(output.contains("3 lines"));
    // Should only have 1 match (the def match), not 2
    assert!(output.contains("1 duplicate regions"));
}

#[test]
fn test_format_text_regex_filters_all() {
    let (f1, f2) = make_mixed_files();
    let result = ComparisonResult {
        f1: &f1,
        f2: &f2,
        runs: vec![
            LineRange::Same {
                r1: Range::new(4, 7),
                r2: Range::new(4, 7),
            },
        ],
    };

    // Filter to only show matches starting with "def" (but match starts with "class")
    let regex = Regex::new(r"^def ").unwrap();
    let output = format_text(&[result], 3, false, 2, 1, Some(&regex));

    // Should show "No duplicate code found" because regex filters out all matches
    assert!(output.contains("No duplicate code found"));
}

#[test]
fn test_format_json_regex_filters_matches() {
    let (f1, f2) = make_mixed_files();
    let result = ComparisonResult {
        f1: &f1,
        f2: &f2,
        runs: vec![
            LineRange::Same {
                r1: Range::new(0, 3),
                r2: Range::new(0, 3),
            },
            LineRange::Diff {
                r1: Range::new(3, 4),
                r2: Range::new(3, 4),
            },
            LineRange::Same {
                r1: Range::new(4, 7),
                r2: Range::new(4, 7),
            },
        ],
    };

    // Filter to only show matches starting with "class"
    let regex = Regex::new(r"^class ").unwrap();
    let output = format_json(&[result], 3, false, 2, 1, Some(&regex));
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    // Should only have 1 match (the class match)
    assert_eq!(parsed["summary"]["duplicates_found"], 1);
    let matches = &parsed["duplicates"][0]["matches"];
    assert_eq!(matches.as_array().unwrap().len(), 1);
    // The match should start at line 5 (1-indexed)
    assert_eq!(parsed["duplicates"][0]["matches"][0]["file1_range"]["start"], 5);
}

#[test]
fn test_format_json_regex_filters_all() {
    let (f1, f2) = make_mixed_files();
    let result = ComparisonResult {
        f1: &f1,
        f2: &f2,
        runs: vec![
            LineRange::Same {
                r1: Range::new(0, 3),
                r2: Range::new(0, 3),
            },
        ],
    };

    // Filter with regex that matches nothing
    let regex = Regex::new(r"^struct ").unwrap();
    let output = format_json(&[result], 3, false, 2, 1, Some(&regex));
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    // Should have no duplicates
    assert_eq!(parsed["summary"]["duplicates_found"], 0);
    assert!(parsed["duplicates"].as_array().unwrap().is_empty());
}

#[test]
fn test_format_text_regex_verbose_shows_content() {
    let (f1, f2) = make_mixed_files();
    let result = ComparisonResult {
        f1: &f1,
        f2: &f2,
        runs: vec![
            LineRange::Same {
                r1: Range::new(0, 3),
                r2: Range::new(0, 3),
            },
        ],
    };

    let regex = Regex::new(r"^def ").unwrap();
    let output = format_text(&[result], 3, true, 2, 1, Some(&regex));

    assert!(output.contains("def hello():"));
    assert!(output.contains("print(\"Hello\")"));
}
