# Program Architecture

## Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         CLI Interface                           │
│                    (clap argument parsing)                       │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      File Discovery                              │
│           (glob patterns, directory scanning)                    │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                   File Processing (parallel)                     │
│    ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│    │ Read File 1  │  │ Read File 2  │  │ Read File N  │         │
│    └──────┬───────┘  └──────┬───────┘  └──────┬───────┘         │
│           │                 │                 │                  │
│    ┌──────▼───────┐  ┌──────▼───────┐  ┌──────▼───────┐         │
│    │ Normalize &  │  │ Normalize &  │  │ Normalize &  │         │
│    │    Hash      │  │    Hash      │  │    Hash      │         │
│    └──────┬───────┘  └──────┬───────┘  └──────┬───────┘         │
│           │                 │                 │                  │
│    ┌──────▼───────┐  ┌──────▼───────┐  ┌──────▼───────┐         │
│    │FileDescriptn │  │FileDescriptn │  │FileDescriptn │         │
│    └──────────────┘  └──────────────┘  └──────────────┘         │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                 Pair Comparison (parallel)                       │
│                                                                  │
│    For each pair (i, j) where i < j:                            │
│    ┌────────────────────────────────────────────────┐           │
│    │  1. Match head lines (Same ranges)             │           │
│    │  2. Match tail lines (Same ranges)             │           │
│    │  3. LCS for middle section                     │           │
│    │  4. Generate Diff ranges for non-matches       │           │
│    │  5. Build LineRanges result                    │           │
│    └────────────────────────────────────────────────┘           │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Result Processing                           │
│                                                                  │
│    1. Filter by minimum match length                            │
│    2. Sort by significance                                      │
│    3. Aggregate common patterns                                 │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Output Formatter                          │
│                                                                  │
│    ┌─────────────────┐          ┌─────────────────┐             │
│    │  Text Output    │    OR    │  JSON Output    │             │
│    │  (human-read)   │          │  (structured)   │             │
│    └─────────────────┘          └─────────────────┘             │
└─────────────────────────────────────────────────────────────────┘
```

## Module Structure

```
src/
├── main.rs              # Entry point, CLI setup, orchestration
├── cli.rs               # Clap argument definitions
├── discovery.rs         # File discovery and filtering
├── file.rs              # FileDescription, file reading, hashing
├── diff.rs              # LineRange, Range, comparison algorithm
├── output/
│   ├── mod.rs           # Output trait and common types
│   ├── text.rs          # Human-readable formatter
│   └── json.rs          # JSON formatter
└── error.rs             # Custom error types
```

## Core Data Structures

### Range
```rust
/// Represents a contiguous range of lines in a file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    /// Starting line index (0-based, inclusive)
    pub start: usize,
    /// Ending line index (0-based, exclusive)
    pub end: usize,
}

impl Range {
    /// Number of lines in this range
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}
```

### LineRange
```rust
/// Represents a comparison result between two files
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineRange {
    /// Lines that are identical between both files
    Same { r1: Range, r2: Range },
    /// Lines that differ between files
    Diff { r1: Range, r2: Range },
}
```

### FileDescription
```rust
use std::path::PathBuf;

/// Represents a file's content as a sequence of line hashes
#[derive(Debug)]
pub struct FileDescription {
    /// Path to the source file
    pub filename: PathBuf,
    /// Hash of each line (after normalization)
    pub hashes: Vec<u64>,
    /// Original line content (for output display)
    pub lines: Vec<String>,
}
```

### LineRanges
```rust
/// Complete comparison result between two files
#[derive(Debug)]
pub struct LineRanges<'a> {
    /// Reference to first file
    pub f1: &'a FileDescription,
    /// Reference to second file
    pub f2: &'a FileDescription,
    /// Sequence of matching/differing line ranges
    pub runs: Vec<LineRange>,
}
```

## Algorithm Details

### Line Normalization
1. Read file as UTF-8 (with fallback handling)
2. Split into lines
3. For each line: `line.trim()` (strips leading/trailing whitespace)
4. Hash the normalized line

### Comparison Algorithm
```
function compare(f1: FileDescription, f2: FileDescription) -> LineRanges:
    runs = []

    // Phase 1: Match head
    head_match = 0
    while head_match < min(f1.len, f2.len)
          AND f1.hashes[head_match] == f2.hashes[head_match]:
        head_match++

    if head_match > 0:
        runs.push(Same(Range(0, head_match), Range(0, head_match)))

    // Phase 2: Match tail
    tail_match = 0
    while tail_match < min(f1.len - head_match, f2.len - head_match)
          AND f1.hashes[f1.len - 1 - tail_match] == f2.hashes[f2.len - 1 - tail_match]:
        tail_match++

    if tail_match > 0:
        runs.push(Same(
            Range(f1.len - tail_match, f1.len),
            Range(f2.len - tail_match, f2.len)
        ))

    // Phase 3: Patience diff for middle section
    middle1 = f1.hashes[head_match..f1.len - tail_match]
    middle2 = f2.hashes[head_match..f2.len - tail_match]

    // Patience diff: find unique lines, use as anchors, recurse on gaps
    patience_runs = patience_diff(middle1, middle2, head_match)
    runs.extend(patience_runs)

    // Phase 4: Fill gaps with Diff entries
    runs = fill_diff_gaps(runs, f1.len, f2.len)

    // Sort by position in file 1
    runs.sort_by(|a, b| a.r1.start.cmp(&b.r1.start))

    return LineRanges { f1, f2, runs }
```

### Parallel Processing Strategy
- **File I/O**: Use rayon's `par_iter` to read and hash files in parallel
- **Comparison**: Use rayon to compare file pairs in parallel
- **Thread pool**: Global rayon thread pool (default: num_cpus)

## CLI Interface

```
samesame [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Files to compare (can also use --dir)

Options:
  -m, --match <N>       Minimum matching lines to report [default: 5]
  -d, --dir <PATH>      Directory to scan for files
  -g, --glob <PATTERN>  Glob pattern for file matching [default: "**/*.rs"]
  -f, --format <FMT>    Output format: text, json [default: text]
  -v, --verbose         Show detailed output
  -q, --quiet           Suppress warnings
  -h, --help            Print help
  -V, --version         Print version
```

## Output Formats

### Text Format
```
=== Duplicate Code Found ===

Files: src/foo.rs <-> src/bar.rs
Match: 12 lines (foo.rs:45-56, bar.rs:102-113)

  45 │ fn calculate_hash(data: &[u8]) -> u64 {
  46 │     let mut hasher = DefaultHasher::new();
  ...

---

Files: src/foo.rs <-> src/baz.rs
Match: 8 lines (foo.rs:10-17, baz.rs:25-32)
...
```

### JSON Format
```json
{
  "version": "1.0.0",
  "summary": {
    "files_analyzed": 10,
    "pairs_compared": 45,
    "duplicates_found": 3
  },
  "duplicates": [
    {
      "file1": "src/foo.rs",
      "file2": "src/bar.rs",
      "matches": [
        {
          "lines": 12,
          "file1_range": { "start": 45, "end": 56 },
          "file2_range": { "start": 102, "end": 113 },
          "content": ["fn calculate_hash...", "..."]
        }
      ]
    }
  ]
}
```

## Error Handling Strategy

- Use `thiserror` for custom error types
- Graceful degradation: skip unreadable files with warning
- Collect all errors, report at end (don't fail fast)
- Exit code: 0 = success, 1 = duplicates found, 2 = error
