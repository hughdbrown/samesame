//! Tests for core data types.

use samesame::types::{ComparisonResult, FileDescription, LineRange, Range};
use std::path::PathBuf;

// Range tests
#[test]
fn test_range_len() {
    let r = Range::new(5, 10);
    assert_eq!(r.len(), 5);
}

#[test]
fn test_range_len_zero() {
    let r = Range::new(5, 5);
    assert_eq!(r.len(), 0);
}

#[test]
fn test_range_len_saturating() {
    // Edge case: start > end should saturate to 0
    let r = Range::new(10, 5);
    assert_eq!(r.len(), 0);
}

#[test]
fn test_range_empty() {
    let r = Range::new(5, 5);
    assert!(r.is_empty());
}

#[test]
fn test_range_not_empty() {
    let r = Range::new(5, 10);
    assert!(!r.is_empty());
}

#[test]
fn test_range_empty_inverted() {
    // start > end is also empty
    let r = Range::new(10, 5);
    assert!(r.is_empty());
}

// LineRange tests
#[test]
fn test_line_range_same() {
    let lr = LineRange::Same {
        r1: Range::new(0, 5),
        r2: Range::new(10, 15),
    };
    assert!(lr.is_same());
    assert_eq!(lr.match_len(), 5);
}

#[test]
fn test_line_range_diff() {
    let lr = LineRange::Diff {
        r1: Range::new(0, 5),
        r2: Range::new(10, 15),
    };
    assert!(!lr.is_same());
    assert_eq!(lr.match_len(), 0);
}

#[test]
fn test_line_range_r1_same() {
    let lr = LineRange::Same {
        r1: Range::new(0, 5),
        r2: Range::new(10, 15),
    };
    assert_eq!(lr.r1().start, 0);
    assert_eq!(lr.r1().end, 5);
}

#[test]
fn test_line_range_r1_diff() {
    let lr = LineRange::Diff {
        r1: Range::new(3, 8),
        r2: Range::new(10, 15),
    };
    assert_eq!(lr.r1().start, 3);
    assert_eq!(lr.r1().end, 8);
}

#[test]
fn test_line_range_r2_same() {
    let lr = LineRange::Same {
        r1: Range::new(0, 5),
        r2: Range::new(10, 15),
    };
    assert_eq!(lr.r2().start, 10);
    assert_eq!(lr.r2().end, 15);
}

#[test]
fn test_line_range_r2_diff() {
    let lr = LineRange::Diff {
        r1: Range::new(0, 5),
        r2: Range::new(20, 25),
    };
    assert_eq!(lr.r2().start, 20);
    assert_eq!(lr.r2().end, 25);
}

// FileDescription tests
#[test]
fn test_file_description_len() {
    let fd = FileDescription {
        filename: PathBuf::from("test.rs"),
        hashes: vec![1, 2, 3, 4, 5],
        lines: vec!["a".into(), "b".into(), "c".into(), "d".into(), "e".into()],
    };
    assert_eq!(fd.len(), 5);
}

#[test]
fn test_file_description_is_empty_false() {
    let fd = FileDescription {
        filename: PathBuf::from("test.rs"),
        hashes: vec![1],
        lines: vec!["a".into()],
    };
    assert!(!fd.is_empty());
}

#[test]
fn test_file_description_is_empty_true() {
    let fd = FileDescription {
        filename: PathBuf::from("test.rs"),
        hashes: vec![],
        lines: vec![],
    };
    assert!(fd.is_empty());
}

// ComparisonResult tests
fn make_test_files() -> (FileDescription, FileDescription) {
    let f1 = FileDescription {
        filename: PathBuf::from("file1.rs"),
        hashes: vec![1, 2, 3, 4, 5],
        lines: vec!["a".into(), "b".into(), "c".into(), "d".into(), "e".into()],
    };
    let f2 = FileDescription {
        filename: PathBuf::from("file2.rs"),
        hashes: vec![1, 2, 3, 6, 7],
        lines: vec!["a".into(), "b".into(), "c".into(), "x".into(), "y".into()],
    };
    (f1, f2)
}

#[test]
fn test_comparison_result_significant_matches() {
    let (f1, f2) = make_test_files();
    let result = ComparisonResult {
        f1: &f1,
        f2: &f2,
        runs: vec![
            LineRange::Same {
                r1: Range::new(0, 3),
                r2: Range::new(0, 3),
            },
            LineRange::Diff {
                r1: Range::new(3, 5),
                r2: Range::new(3, 5),
            },
        ],
    };

    // With min_match = 3, should find 1 match
    let matches = result.significant_matches(3);
    assert_eq!(matches.len(), 1);

    // With min_match = 4, should find no matches
    let matches = result.significant_matches(4);
    assert_eq!(matches.len(), 0);
}

#[test]
fn test_comparison_result_has_significant_matches() {
    let (f1, f2) = make_test_files();
    let result = ComparisonResult {
        f1: &f1,
        f2: &f2,
        runs: vec![LineRange::Same {
            r1: Range::new(0, 5),
            r2: Range::new(0, 5),
        }],
    };

    assert!(result.has_significant_matches(5));
    assert!(result.has_significant_matches(3));
    assert!(!result.has_significant_matches(6));
}

#[test]
fn test_comparison_result_no_matches() {
    let (f1, f2) = make_test_files();
    let result = ComparisonResult {
        f1: &f1,
        f2: &f2,
        runs: vec![LineRange::Diff {
            r1: Range::new(0, 5),
            r2: Range::new(0, 5),
        }],
    };

    assert!(!result.has_significant_matches(1));
    assert!(result.significant_matches(1).is_empty());
}

#[test]
fn test_comparison_result_multiple_matches() {
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

    // Both matches are 2 lines
    let matches = result.significant_matches(2);
    assert_eq!(matches.len(), 2);

    let matches = result.significant_matches(3);
    assert_eq!(matches.len(), 0);
}
