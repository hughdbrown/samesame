//! Patience diff algorithm implementation.

use std::collections::HashMap;

use crate::types::{ComparisonResult, FileDescription, LineRange, Range};

/// Compare two files using the patience diff algorithm.
pub fn compare_files<'a>(f1: &'a FileDescription, f2: &'a FileDescription) -> ComparisonResult<'a> {
    let runs = patience_diff(&f1.hashes, &f2.hashes);
    ComparisonResult { f1, f2, runs }
}

/// Patience diff algorithm.
///
/// 1. Match identical lines at head
/// 2. Match identical lines at tail
/// 3. For the middle section, find unique lines as anchors
/// 4. Use longest increasing subsequence on anchors
/// 5. Recursively process gaps between anchors
#[doc(hidden)]
pub fn patience_diff(hashes1: &[u64], hashes2: &[u64]) -> Vec<LineRange> {
    if hashes1.is_empty() && hashes2.is_empty() {
        return vec![];
    }

    if hashes1.is_empty() {
        return vec![LineRange::Diff {
            r1: Range::new(0, 0),
            r2: Range::new(0, hashes2.len()),
        }];
    }

    if hashes2.is_empty() {
        return vec![LineRange::Diff {
            r1: Range::new(0, hashes1.len()),
            r2: Range::new(0, 0),
        }];
    }

    patience_diff_recursive(hashes1, hashes2, 0, 0)
}

/// Recursive patience diff with offset tracking.
#[doc(hidden)]
pub fn patience_diff_recursive(
    hashes1: &[u64],
    hashes2: &[u64],
    offset1: usize,
    offset2: usize,
) -> Vec<LineRange> {
    let len1 = hashes1.len();
    let len2 = hashes2.len();

    if len1 == 0 && len2 == 0 {
        return vec![];
    }

    if len1 == 0 {
        return vec![LineRange::Diff {
            r1: Range::new(offset1, offset1),
            r2: Range::new(offset2, offset2 + len2),
        }];
    }

    if len2 == 0 {
        return vec![LineRange::Diff {
            r1: Range::new(offset1, offset1 + len1),
            r2: Range::new(offset2, offset2),
        }];
    }

    // Phase 1: Match head
    let mut head_match = 0;
    while head_match < len1 && head_match < len2 && hashes1[head_match] == hashes2[head_match] {
        head_match += 1;
    }

    // Phase 2: Match tail
    let mut tail_match = 0;
    while tail_match < (len1 - head_match)
        && tail_match < (len2 - head_match)
        && hashes1[len1 - 1 - tail_match] == hashes2[len2 - 1 - tail_match]
    {
        tail_match += 1;
    }

    let mut runs = Vec::new();

    // Add head match
    if head_match > 0 {
        runs.push(LineRange::Same {
            r1: Range::new(offset1, offset1 + head_match),
            r2: Range::new(offset2, offset2 + head_match),
        });
    }

    // Process middle section
    let mid_start1 = head_match;
    let mid_end1 = len1 - tail_match;
    let mid_start2 = head_match;
    let mid_end2 = len2 - tail_match;

    if mid_start1 < mid_end1 || mid_start2 < mid_end2 {
        let middle1 = &hashes1[mid_start1..mid_end1];
        let middle2 = &hashes2[mid_start2..mid_end2];

        let middle_runs = diff_middle(
            middle1,
            middle2,
            offset1 + mid_start1,
            offset2 + mid_start2,
        );
        runs.extend(middle_runs);
    }

    // Add tail match
    if tail_match > 0 {
        runs.push(LineRange::Same {
            r1: Range::new(offset1 + len1 - tail_match, offset1 + len1),
            r2: Range::new(offset2 + len2 - tail_match, offset2 + len2),
        });
    }

    runs
}

/// Process the middle section using patience diff anchors.
#[doc(hidden)]
pub fn diff_middle(hashes1: &[u64], hashes2: &[u64], offset1: usize, offset2: usize) -> Vec<LineRange> {
    if hashes1.is_empty() && hashes2.is_empty() {
        return vec![];
    }

    if hashes1.is_empty() {
        return vec![LineRange::Diff {
            r1: Range::new(offset1, offset1),
            r2: Range::new(offset2, offset2 + hashes2.len()),
        }];
    }

    if hashes2.is_empty() {
        return vec![LineRange::Diff {
            r1: Range::new(offset1, offset1 + hashes1.len()),
            r2: Range::new(offset2, offset2),
        }];
    }

    // Find unique lines in each sequence
    let unique1 = find_unique_lines(hashes1);
    let unique2 = find_unique_lines(hashes2);

    // Find matching unique lines (hash -> (index in 1, index in 2))
    let mut matches: Vec<(usize, usize)> = Vec::new();
    for (&hash, &idx1) in &unique1 {
        if let Some(&idx2) = unique2.get(&hash) {
            matches.push((idx1, idx2));
        }
    }

    if matches.is_empty() {
        // No unique anchors - fall back to LCS
        return lcs_diff(hashes1, hashes2, offset1, offset2);
    }

    // Sort by position in first sequence
    matches.sort_by_key(|&(idx1, _)| idx1);

    // Find longest increasing subsequence by index in second sequence
    let lis = longest_increasing_subsequence(&matches);

    if lis.is_empty() {
        // No valid LIS - fall back to LCS
        return lcs_diff(hashes1, hashes2, offset1, offset2);
    }

    // Use LIS anchors to divide and conquer
    let mut runs = Vec::new();
    let mut pos1 = 0usize;
    let mut pos2 = 0usize;

    for &(idx1, idx2) in &lis {
        // Process gap before this anchor
        if pos1 < idx1 || pos2 < idx2 {
            let gap_runs = patience_diff_recursive(
                &hashes1[pos1..idx1],
                &hashes2[pos2..idx2],
                offset1 + pos1,
                offset2 + pos2,
            );
            runs.extend(gap_runs);
        }

        // Add the matching anchor line
        runs.push(LineRange::Same {
            r1: Range::new(offset1 + idx1, offset1 + idx1 + 1),
            r2: Range::new(offset2 + idx2, offset2 + idx2 + 1),
        });

        pos1 = idx1 + 1;
        pos2 = idx2 + 1;
    }

    // Process remaining gap after last anchor
    if pos1 < hashes1.len() || pos2 < hashes2.len() {
        let gap_runs = patience_diff_recursive(
            &hashes1[pos1..],
            &hashes2[pos2..],
            offset1 + pos1,
            offset2 + pos2,
        );
        runs.extend(gap_runs);
    }

    // Merge adjacent Same ranges
    merge_runs(runs)
}

/// Find lines that appear exactly once in a sequence.
/// Returns a map from hash to index.
#[doc(hidden)]
pub fn find_unique_lines(hashes: &[u64]) -> HashMap<u64, usize> {
    let mut counts: HashMap<u64, (usize, usize)> = HashMap::new();

    for (idx, &hash) in hashes.iter().enumerate() {
        counts
            .entry(hash)
            .and_modify(|(_, count)| *count += 1)
            .or_insert((idx, 1));
    }

    counts
        .into_iter()
        .filter_map(|(hash, (idx, count))| {
            if count == 1 {
                Some((hash, idx))
            } else {
                None
            }
        })
        .collect()
}

/// Find the longest increasing subsequence of indices in the second file.
/// Uses an O(n log n) algorithm with binary search.
#[doc(hidden)]
pub fn longest_increasing_subsequence(matches: &[(usize, usize)]) -> Vec<(usize, usize)> {
    if matches.is_empty() {
        return vec![];
    }

    let n = matches.len();
    // tails[i] holds the index into `matches` of the smallest tail element
    // for an increasing subsequence of length i+1.
    let mut tails: Vec<usize> = Vec::with_capacity(n);
    // prev[i] holds the index of the predecessor of matches[i] in the LIS.
    let mut prev = vec![None::<usize>; n];

    for i in 0..n {
        let val = matches[i].1;

        // Binary search for the leftmost position in tails where
        // matches[tails[pos]].1 >= val
        let pos = tails.partition_point(|&t| matches[t].1 < val);

        if pos > 0 {
            prev[i] = Some(tails[pos - 1]);
        }

        if pos == tails.len() {
            tails.push(i);
        } else {
            tails[pos] = i;
        }
    }

    // Reconstruct the subsequence by following prev pointers from the last element
    let mut result = Vec::with_capacity(tails.len());
    let mut idx = Some(*tails.last().expect("tails is non-empty"));
    while let Some(i) = idx {
        result.push(matches[i]);
        idx = prev[i];
    }

    result.reverse();
    result
}

/// Simple LCS-based diff as fallback when no unique anchors exist.
#[doc(hidden)]
pub fn lcs_diff(hashes1: &[u64], hashes2: &[u64], offset1: usize, offset2: usize) -> Vec<LineRange> {
    let lcs = compute_lcs(hashes1, hashes2);

    if lcs.is_empty() {
        // No common elements - everything is a diff
        let mut runs = Vec::new();
        if !hashes1.is_empty() || !hashes2.is_empty() {
            runs.push(LineRange::Diff {
                r1: Range::new(offset1, offset1 + hashes1.len()),
                r2: Range::new(offset2, offset2 + hashes2.len()),
            });
        }
        return runs;
    }

    let mut runs = Vec::new();
    let mut pos1 = 0usize;
    let mut pos2 = 0usize;

    for (idx1, idx2) in lcs {
        // Add diff for gap before this match
        if pos1 < idx1 || pos2 < idx2 {
            runs.push(LineRange::Diff {
                r1: Range::new(offset1 + pos1, offset1 + idx1),
                r2: Range::new(offset2 + pos2, offset2 + idx2),
            });
        }

        // Add the matching line
        runs.push(LineRange::Same {
            r1: Range::new(offset1 + idx1, offset1 + idx1 + 1),
            r2: Range::new(offset2 + idx2, offset2 + idx2 + 1),
        });

        pos1 = idx1 + 1;
        pos2 = idx2 + 1;
    }

    // Add remaining diff
    if pos1 < hashes1.len() || pos2 < hashes2.len() {
        runs.push(LineRange::Diff {
            r1: Range::new(offset1 + pos1, offset1 + hashes1.len()),
            r2: Range::new(offset2 + pos2, offset2 + hashes2.len()),
        });
    }

    merge_runs(runs)
}

/// Compute LCS and return matching index pairs using Hirschberg's algorithm.
/// Uses O(min(m, n)) space instead of O(m * n).
#[doc(hidden)]
pub fn compute_lcs(hashes1: &[u64], hashes2: &[u64]) -> Vec<(usize, usize)> {
    let m = hashes1.len();
    let n = hashes2.len();

    if m == 0 || n == 0 {
        return vec![];
    }

    hirschberg(hashes1, hashes2, 0, 0)
}

/// Compute the last row of the LCS DP table using O(n) space.
/// Returns a vector where result[j] = LCS length of hashes1[..] and hashes2[..j].
fn lcs_lengths(hashes1: &[u64], hashes2: &[u64]) -> Vec<usize> {
    let n = hashes2.len();
    let mut prev = vec![0usize; n + 1];
    let mut curr = vec![0usize; n + 1];

    for &h1 in hashes1 {
        for j in 1..=n {
            if h1 == hashes2[j - 1] {
                curr[j] = prev[j - 1] + 1;
            } else {
                curr[j] = prev[j].max(curr[j - 1]);
            }
        }
        std::mem::swap(&mut prev, &mut curr);
        curr.fill(0);
    }

    prev
}

/// Hirschberg's divide-and-conquer LCS algorithm.
/// offset1 and offset2 track the original indices for the returned pairs.
fn hirschberg(
    hashes1: &[u64],
    hashes2: &[u64],
    offset1: usize,
    offset2: usize,
) -> Vec<(usize, usize)> {
    let m = hashes1.len();
    let n = hashes2.len();

    if m == 0 || n == 0 {
        return vec![];
    }

    if m == 1 {
        // Base case: single element in hashes1, find first match in hashes2
        if let Some(j) = hashes2.iter().position(|&h| h == hashes1[0]) {
            return vec![(offset1, offset2 + j)];
        }
        return vec![];
    }

    let mid = m / 2;

    // Score forward: LCS lengths of hashes1[..mid] vs hashes2
    let score_left = lcs_lengths(&hashes1[..mid], hashes2);

    // Score backward: LCS lengths of reversed hashes1[mid..] vs reversed hashes2
    let right1: Vec<u64> = hashes1[mid..].iter().rev().copied().collect();
    let right2: Vec<u64> = hashes2.iter().rev().copied().collect();
    let score_right = lcs_lengths(&right1, &right2);

    // Find the split point in hashes2 that maximizes the combined LCS length
    let mut best_k = 0;
    let mut best_score = 0;
    for k in 0..=n {
        let score = score_left[k] + score_right[n - k];
        if score > best_score {
            best_score = score;
            best_k = k;
        }
    }

    // Recurse on each half
    let mut result = hirschberg(
        &hashes1[..mid],
        &hashes2[..best_k],
        offset1,
        offset2,
    );
    result.extend(hirschberg(
        &hashes1[mid..],
        &hashes2[best_k..],
        offset1 + mid,
        offset2 + best_k,
    ));

    result
}

/// Merge adjacent Same ranges into single ranges.
#[doc(hidden)]
pub fn merge_runs(runs: Vec<LineRange>) -> Vec<LineRange> {
    if runs.is_empty() {
        return runs;
    }

    let mut merged = Vec::with_capacity(runs.len());
    let mut iter = runs.into_iter();
    let mut current = iter.next().expect("runs is non-empty after is_empty check");

    for next in iter {
        match (&current, &next) {
            (
                LineRange::Same { r1: r1a, r2: r2a },
                LineRange::Same { r1: r1b, r2: r2b },
            ) if r1a.end == r1b.start && r2a.end == r2b.start => {
                // Merge adjacent Same ranges
                current = LineRange::Same {
                    r1: Range::new(r1a.start, r1b.end),
                    r2: Range::new(r2a.start, r2b.end),
                };
            }
            (
                LineRange::Diff { r1: r1a, r2: r2a },
                LineRange::Diff { r1: r1b, r2: r2b },
            ) if r1a.end == r1b.start && r2a.end == r2b.start => {
                // Merge adjacent Diff ranges
                current = LineRange::Diff {
                    r1: Range::new(r1a.start, r1b.end),
                    r2: Range::new(r2a.start, r2b.end),
                };
            }
            _ => {
                merged.push(current);
                current = next;
            }
        }
    }

    merged.push(current);
    merged
}
