//! Tests for patience diff algorithm.

use samesame::diff::{
    compare_files, compute_lcs, diff_middle, find_unique_lines, lcs_diff,
    longest_increasing_subsequence, merge_runs, patience_diff,
};
use samesame::types::{FileDescription, LineRange, Range};
use std::path::PathBuf;

fn make_hashes(lines: &[&str]) -> Vec<u64> {
    lines
        .iter()
        .map(|s| {
            let hash = blake3::hash(s.as_bytes());
            let bytes = hash.as_bytes();
            u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ])
        })
        .collect()
}

#[test]
fn test_identical_files() {
    let h1 = make_hashes(&["a", "b", "c"]);
    let h2 = make_hashes(&["a", "b", "c"]);

    let runs = patience_diff(&h1, &h2);

    assert_eq!(runs.len(), 1);
    assert!(matches!(&runs[0], LineRange::Same { r1, r2 }
        if r1.start == 0 && r1.end == 3 && r2.start == 0 && r2.end == 3));
}

#[test]
fn test_completely_different() {
    let h1 = make_hashes(&["a", "b", "c"]);
    let h2 = make_hashes(&["x", "y", "z"]);

    let runs = patience_diff(&h1, &h2);

    assert_eq!(runs.len(), 1);
    assert!(matches!(&runs[0], LineRange::Diff { .. }));
}

#[test]
fn test_head_match() {
    let h1 = make_hashes(&["a", "b", "c"]);
    let h2 = make_hashes(&["a", "b", "x"]);

    let runs = patience_diff(&h1, &h2);

    assert!(runs
        .iter()
        .any(|r| matches!(r, LineRange::Same { r1, .. } if r1.len() == 2)));
}

#[test]
fn test_tail_match() {
    let h1 = make_hashes(&["x", "b", "c"]);
    let h2 = make_hashes(&["a", "b", "c"]);

    let runs = patience_diff(&h1, &h2);

    assert!(runs
        .iter()
        .any(|r| matches!(r, LineRange::Same { r1, .. } if r1.len() == 2)));
}

#[test]
fn test_empty_files() {
    let h1: Vec<u64> = vec![];
    let h2: Vec<u64> = vec![];

    let runs = patience_diff(&h1, &h2);
    assert!(runs.is_empty());
}

#[test]
fn test_one_empty() {
    let h1 = make_hashes(&["a", "b"]);
    let h2: Vec<u64> = vec![];

    let runs = patience_diff(&h1, &h2);

    assert_eq!(runs.len(), 1);
    assert!(matches!(&runs[0], LineRange::Diff { r1, r2 }
        if r1.len() == 2 && r2.is_empty()));
}

#[test]
fn test_second_empty() {
    let h1: Vec<u64> = vec![];
    let h2 = make_hashes(&["a", "b"]);

    let runs = patience_diff(&h1, &h2);

    assert_eq!(runs.len(), 1);
    assert!(matches!(&runs[0], LineRange::Diff { r1, r2 }
        if r1.is_empty() && r2.len() == 2));
}

#[test]
fn test_lis() {
    let matches = vec![(0, 2), (1, 0), (2, 3), (3, 1), (4, 4)];
    let lis = longest_increasing_subsequence(&matches);

    // LIS should be: (0,2), (2,3), (4,4) or similar
    assert!(lis.len() >= 3);

    // Verify it's actually increasing
    for i in 1..lis.len() {
        assert!(lis[i].1 > lis[i - 1].1);
    }
}

#[test]
fn test_lis_empty() {
    let matches: Vec<(usize, usize)> = vec![];
    let lis = longest_increasing_subsequence(&matches);
    assert!(lis.is_empty());
}

#[test]
fn test_lis_single() {
    let matches = vec![(0, 0)];
    let lis = longest_increasing_subsequence(&matches);
    assert_eq!(lis.len(), 1);
}

#[test]
fn test_lis_already_sorted() {
    let matches = vec![(0, 0), (1, 1), (2, 2), (3, 3)];
    let lis = longest_increasing_subsequence(&matches);
    assert_eq!(lis.len(), 4);
}

#[test]
fn test_lis_reverse_sorted() {
    let matches = vec![(0, 3), (1, 2), (2, 1), (3, 0)];
    let lis = longest_increasing_subsequence(&matches);
    // Only one element can be in LIS when fully reversed
    assert_eq!(lis.len(), 1);
}

#[test]
fn test_middle_insertion() {
    // File 1: a, b, c
    // File 2: a, x, b, c
    let h1 = make_hashes(&["a", "b", "c"]);
    let h2 = make_hashes(&["a", "x", "b", "c"]);

    let runs = patience_diff(&h1, &h2);

    // Should find "a" at head and "b", "c" somewhere
    let same_count: usize = runs
        .iter()
        .filter(|r| r.is_same())
        .map(|r| r.match_len())
        .sum();
    assert!(same_count >= 3);
}

#[test]
fn test_middle_deletion() {
    // File 1: a, x, b, c
    // File 2: a, b, c
    let h1 = make_hashes(&["a", "x", "b", "c"]);
    let h2 = make_hashes(&["a", "b", "c"]);

    let runs = patience_diff(&h1, &h2);

    let same_count: usize = runs
        .iter()
        .filter(|r| r.is_same())
        .map(|r| r.match_len())
        .sum();
    assert!(same_count >= 3);
}

#[test]
fn test_interleaved_changes() {
    // File 1: a, b, c, d, e
    // File 2: a, x, c, y, e
    let h1 = make_hashes(&["a", "b", "c", "d", "e"]);
    let h2 = make_hashes(&["a", "x", "c", "y", "e"]);

    let runs = patience_diff(&h1, &h2);

    // Should find a, c, e as matches
    let same_count: usize = runs
        .iter()
        .filter(|r| r.is_same())
        .map(|r| r.match_len())
        .sum();
    assert!(same_count >= 3);
}

#[test]
fn test_duplicate_lines() {
    // Lines that appear multiple times (not unique)
    let h1 = make_hashes(&["a", "a", "a"]);
    let h2 = make_hashes(&["a", "a", "a"]);

    let runs = patience_diff(&h1, &h2);

    // Should still find all as same (via LCS fallback)
    assert_eq!(runs.len(), 1);
    assert!(matches!(&runs[0], LineRange::Same { r1, .. } if r1.len() == 3));
}

#[test]
fn test_no_unique_lines() {
    // All lines appear multiple times in both files
    let h1 = make_hashes(&["a", "b", "a", "b"]);
    let h2 = make_hashes(&["a", "b", "a", "b"]);

    let runs = patience_diff(&h1, &h2);

    // Should find all as same (via LCS fallback since no unique anchors)
    let total_same: usize = runs
        .iter()
        .filter(|r| r.is_same())
        .map(|r| r.match_len())
        .sum();
    assert_eq!(total_same, 4);
}

#[test]
fn test_single_line_files() {
    let h1 = make_hashes(&["only"]);
    let h2 = make_hashes(&["only"]);

    let runs = patience_diff(&h1, &h2);

    assert_eq!(runs.len(), 1);
    assert!(matches!(&runs[0], LineRange::Same { r1, .. } if r1.len() == 1));
}

#[test]
fn test_single_line_different() {
    let h1 = make_hashes(&["one"]);
    let h2 = make_hashes(&["two"]);

    let runs = patience_diff(&h1, &h2);

    assert_eq!(runs.len(), 1);
    assert!(matches!(&runs[0], LineRange::Diff { .. }));
}

#[test]
fn test_head_and_tail_match() {
    // File 1: a, b, c, d, e
    // File 2: a, b, x, d, e
    let h1 = make_hashes(&["a", "b", "c", "d", "e"]);
    let h2 = make_hashes(&["a", "b", "x", "d", "e"]);

    let runs = patience_diff(&h1, &h2);

    // Should have head match (a, b) and tail match (d, e)
    let same_count: usize = runs
        .iter()
        .filter(|r| r.is_same())
        .map(|r| r.match_len())
        .sum();
    assert_eq!(same_count, 4);
}

#[test]
fn test_merge_runs_adjacent_same() {
    let runs = vec![
        LineRange::Same {
            r1: Range::new(0, 2),
            r2: Range::new(0, 2),
        },
        LineRange::Same {
            r1: Range::new(2, 4),
            r2: Range::new(2, 4),
        },
    ];

    let merged = merge_runs(runs);

    assert_eq!(merged.len(), 1);
    assert!(matches!(&merged[0], LineRange::Same { r1, r2 }
        if r1.start == 0 && r1.end == 4 && r2.start == 0 && r2.end == 4));
}

#[test]
fn test_merge_runs_adjacent_diff() {
    let runs = vec![
        LineRange::Diff {
            r1: Range::new(0, 2),
            r2: Range::new(0, 2),
        },
        LineRange::Diff {
            r1: Range::new(2, 4),
            r2: Range::new(2, 4),
        },
    ];

    let merged = merge_runs(runs);

    assert_eq!(merged.len(), 1);
    assert!(matches!(&merged[0], LineRange::Diff { r1, r2 }
        if r1.start == 0 && r1.end == 4 && r2.start == 0 && r2.end == 4));
}

#[test]
fn test_merge_runs_mixed() {
    let runs = vec![
        LineRange::Same {
            r1: Range::new(0, 2),
            r2: Range::new(0, 2),
        },
        LineRange::Diff {
            r1: Range::new(2, 4),
            r2: Range::new(2, 4),
        },
    ];

    let merged = merge_runs(runs);

    // Different types shouldn't merge
    assert_eq!(merged.len(), 2);
}

#[test]
fn test_merge_runs_non_adjacent() {
    let runs = vec![
        LineRange::Same {
            r1: Range::new(0, 2),
            r2: Range::new(0, 2),
        },
        LineRange::Same {
            r1: Range::new(5, 7),
            r2: Range::new(5, 7),
        },
    ];

    let merged = merge_runs(runs);

    // Non-adjacent shouldn't merge
    assert_eq!(merged.len(), 2);
}

#[test]
fn test_merge_runs_empty() {
    let runs: Vec<LineRange> = vec![];
    let merged = merge_runs(runs);
    assert!(merged.is_empty());
}

#[test]
fn test_find_unique_lines() {
    let hashes = vec![1, 2, 3, 2, 4];
    let unique = find_unique_lines(&hashes);

    // 1, 3, 4 are unique; 2 appears twice
    assert_eq!(unique.len(), 3);
    assert!(unique.contains_key(&1));
    assert!(unique.contains_key(&3));
    assert!(unique.contains_key(&4));
    assert!(!unique.contains_key(&2));
}

#[test]
fn test_find_unique_lines_all_same() {
    let hashes = vec![1, 1, 1, 1];
    let unique = find_unique_lines(&hashes);
    assert!(unique.is_empty());
}

#[test]
fn test_find_unique_lines_all_unique() {
    let hashes = vec![1, 2, 3, 4];
    let unique = find_unique_lines(&hashes);
    assert_eq!(unique.len(), 4);
}

#[test]
fn test_find_unique_lines_empty() {
    let hashes: Vec<u64> = vec![];
    let unique = find_unique_lines(&hashes);
    assert!(unique.is_empty());
}

#[test]
fn test_compute_lcs_identical() {
    let h1 = vec![1, 2, 3];
    let h2 = vec![1, 2, 3];
    let lcs = compute_lcs(&h1, &h2);
    assert_eq!(lcs.len(), 3);
}

#[test]
fn test_compute_lcs_completely_different() {
    let h1 = vec![1, 2, 3];
    let h2 = vec![4, 5, 6];
    let lcs = compute_lcs(&h1, &h2);
    assert!(lcs.is_empty());
}

#[test]
fn test_compute_lcs_partial() {
    let h1 = vec![1, 2, 3, 4];
    let h2 = vec![1, 3, 4];
    let lcs = compute_lcs(&h1, &h2);
    // LCS could be [1, 3, 4] (length 3)
    assert!(lcs.len() >= 2);
}

#[test]
fn test_compute_lcs_empty() {
    let h1: Vec<u64> = vec![];
    let h2 = vec![1, 2, 3];
    let lcs = compute_lcs(&h1, &h2);
    assert!(lcs.is_empty());

    let lcs2 = compute_lcs(&h2, &h1);
    assert!(lcs2.is_empty());
}

#[test]
fn test_lcs_diff_fallback() {
    // Test the LCS diff function directly
    let h1 = vec![1, 2, 3];
    let h2 = vec![4, 5, 6];
    let runs = lcs_diff(&h1, &h2, 0, 0);

    // Should be a single diff
    assert_eq!(runs.len(), 1);
    assert!(matches!(&runs[0], LineRange::Diff { .. }));
}

#[test]
fn test_compare_files() {
    let f1 = FileDescription {
        filename: PathBuf::from("a.rs"),
        hashes: make_hashes(&["a", "b", "c"]),
        lines: vec!["a".into(), "b".into(), "c".into()],
    };
    let f2 = FileDescription {
        filename: PathBuf::from("b.rs"),
        hashes: make_hashes(&["a", "b", "c"]),
        lines: vec!["a".into(), "b".into(), "c".into()],
    };

    let result = compare_files(&f1, &f2);

    assert_eq!(result.f1.filename, PathBuf::from("a.rs"));
    assert_eq!(result.f2.filename, PathBuf::from("b.rs"));
    assert!(result.has_significant_matches(3));
}

#[test]
fn test_large_head_tail_small_middle() {
    // Many matching lines at head and tail, small diff in middle
    let h1 = make_hashes(&["a", "b", "c", "d", "e", "f", "g", "h"]);
    let h2 = make_hashes(&["a", "b", "c", "x", "e", "f", "g", "h"]);

    let runs = patience_diff(&h1, &h2);

    let same_count: usize = runs
        .iter()
        .filter(|r| r.is_same())
        .map(|r| r.match_len())
        .sum();
    assert_eq!(same_count, 7); // All except 'd' vs 'x'
}

#[test]
fn test_diff_middle_empty_first() {
    let h1: Vec<u64> = vec![];
    let h2 = vec![1, 2, 3];
    let runs = diff_middle(&h1, &h2, 0, 0);

    assert_eq!(runs.len(), 1);
    assert!(matches!(&runs[0], LineRange::Diff { r1, r2 }
        if r1.is_empty() && r2.len() == 3));
}

#[test]
fn test_diff_middle_empty_second() {
    let h1 = vec![1, 2, 3];
    let h2: Vec<u64> = vec![];
    let runs = diff_middle(&h1, &h2, 0, 0);

    assert_eq!(runs.len(), 1);
    assert!(matches!(&runs[0], LineRange::Diff { r1, r2 }
        if r1.len() == 3 && r2.is_empty()));
}

#[test]
fn test_diff_middle_both_empty() {
    let h1: Vec<u64> = vec![];
    let h2: Vec<u64> = vec![];
    let runs = diff_middle(&h1, &h2, 0, 0);
    assert!(runs.is_empty());
}
