# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**samesame** is a Rust CLI tool that identifies repeated fragments of code across multiple files using the patience diff algorithm. It's designed to support refactoring by finding duplicate code regions.

## Build and Test Commands

```bash
# Build
cargo build
cargo build --release

# Run tests
cargo test

# Run a single test
cargo test test_name

# Run tests in a specific module
cargo test --test test_diff

# Clippy and formatting
cargo clippy
cargo fmt

# Run the tool
cargo run -- [OPTIONS] [FILES]
cargo run -- -d . -g "**/*.rs" -m 5
```

## Architecture

The application follows a pipeline architecture:

1. **CLI Parsing** (`cli.rs`) → Parse arguments with clap derive macros
2. **File Discovery** (`discovery.rs`) → Find files via explicit paths or glob patterns
3. **File Processing** (`file.rs`) → Read files, normalize lines, compute BLAKE3 hashes
4. **Pairwise Comparison** (`diff.rs`) → Compare all file pairs using patience diff
5. **Output Formatting** (`output.rs`) → Render results as text or JSON

### Key Data Flow

```
Files → FileDescription (path + line hashes + line content)
     → generate_pairs() creates (i, j) pairs where i < j
     → compare_files() runs patience diff on hash vectors
     → ComparisonResult with Vec<LineRange>
     → format_text/format_json filters by min_match threshold
```

### Core Types

- `FileDescription`: Holds file path, per-line BLAKE3 hashes, and original line content
- `Range`: 0-based half-open range `[start, end)` for line indices
- `LineRange`: Either `Same { r1, r2 }` or `Diff { r1, r2 }` representing matching/differing regions
- `ComparisonResult`: References two files and their diff results

### Patience Diff Algorithm (`diff.rs`)

1. Match identical lines at head of both files
2. Match identical lines at tail
3. For the middle: find lines unique to each file as anchors
4. Compute longest increasing subsequence (LIS) on anchor positions
5. Recursively process gaps between anchors
6. Fall back to LCS when no unique anchors exist

### Parallelism

Uses `rayon` for parallel file reading/hashing and parallel pair comparison. No async runtime.

## CLI Options

- `-m, --match <N>`: Minimum matching lines to report (default: 5)
- `-d, --dir <PATH>`: Directory to scan
- `-g, --glob <PATTERN>`: Glob pattern (default: `**/*.rs`)
- `-f, --format <text|json>`: Output format
- `-v, --verbose`: Show matching line content
- `-q, --quiet`: Suppress warnings
- `-r, --regex <PATTERN>`: Only show matches where first line matches regex

## Exit Codes

- 0: No duplicates found
- 1: Duplicates found
- 2: Error occurred

## Design Decisions

- Binary files are skipped (detected via null bytes in first 8KB)
- Symlinks are skipped to avoid infinite loops
- Empty files are skipped
- Lines are trimmed before hashing but original indentation is preserved in output
- Non-UTF-8 bytes use lossy conversion
- Hash-only comparison (no content verification for hash collisions)
