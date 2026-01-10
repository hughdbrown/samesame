# Implementation Tasks

## Stage 1: Foundation

### 1.1 Project Setup
- [ ] Add dependencies to Cargo.toml (clap, rayon, serde, serde_json, glob)
- [ ] Configure clap for command line argument parsing
- [ ] Set up basic error handling with `thiserror` or `anyhow`

### 1.2 Core Data Structures
- [ ] Implement `Range` struct with `start` and `end` fields
- [ ] Implement `LineRange` enum with `Same` and `Diff` variants
- [ ] Implement `FileDescription` struct (filename, hashes vector)
- [ ] Implement `LineRanges` struct (file refs, runs vector)
- [ ] Add unit tests for all data structures

### 1.3 File Reading & Hashing
- [ ] Implement file reading with error handling
- [ ] Implement line normalization (strip leading/trailing whitespace)
- [ ] Implement line hashing (use `std::hash` or `xxhash`)
- [ ] Create `FileDescription::from_path()` constructor
- [ ] Add unit tests for file reading and hashing

## Stage 2: Core Algorithm

### 2.1 Initial Matching (Head/Tail)
- [ ] Implement matching of identical lines at file start
- [ ] Implement matching of identical lines at file end
- [ ] Create initial `LineRange::Same` entries for head/tail matches
- [ ] Add unit tests for head/tail matching

### 2.2 Patience Diff Algorithm
- [ ] Implement patience diff: find unique lines as anchors
- [ ] Match anchors between files using longest increasing subsequence
- [ ] Recursively process gaps between matched anchors
- [ ] Build complete `runs` vector with all LineRange entries
- [ ] Add unit tests for patience diff algorithm

### 2.3 Diff Generation
- [ ] Handle lines present in only one file
- [ ] Create `LineRange::Diff` entries for non-matching sections
- [ ] Ensure all lines are accounted for in final result
- [ ] Add integration tests for complete diff generation

## Stage 3: File Discovery & Comparison

### 3.1 File Discovery
- [ ] Accept explicit file paths as CLI arguments
- [ ] Implement directory scanning with glob patterns
- [ ] Filter files by extension (configurable)
- [ ] Detect and skip binary files (check for null bytes in first 8KB)
- [ ] Exclude hidden files and directories (configurable)
- [ ] Add tests for file discovery

### 3.2 Pairwise Comparison
- [ ] Generate all unique file pairs (N choose 2)
- [ ] Implement comparison orchestration
- [ ] Track results per file pair
- [ ] Add tests for pair generation

### 3.3 Parallel Processing
- [ ] Use rayon for parallel file reading/hashing
- [ ] Use rayon for parallel pair comparison
- [ ] Ensure thread-safe result aggregation
- [ ] Add performance benchmarks

## Stage 4: Output & Reporting

### 4.1 Result Filtering
- [ ] Filter results by minimum match length (`-m` flag)
- [ ] Sort results by significance (match length, frequency)
- [ ] Aggregate duplicate patterns across file pairs

### 4.2 Human-Readable Output
- [ ] Format file names and line numbers clearly
- [ ] Show matching line content with context
- [ ] Color-code output for terminal (optional)
- [ ] Add verbose mode for detailed output

### 4.3 JSON Output
- [ ] Define JSON schema for results
- [ ] Implement serde serialization
- [ ] Add `--format` flag (text/json)
- [ ] Add tests for JSON output format

## Stage 5: Polish & Release

### 5.1 Error Handling
- [ ] Handle file read errors gracefully
- [ ] Handle encoding issues (UTF-8 assumption)
- [ ] Provide meaningful error messages
- [ ] Add `--quiet` flag to suppress warnings

### 5.2 Documentation
- [ ] Add rustdoc comments to all public items
- [ ] Write README with usage examples
- [ ] Add `--help` with clear descriptions
- [ ] Include example output in docs

### 5.3 Testing & CI
- [ ] Achieve complete unit test coverage
- [ ] Add integration tests with sample files
- [ ] Set up CI pipeline (GitHub Actions)
- [ ] Add code quality checks (clippy, fmt)

### 5.4 Release
- [ ] Finalize version 1.0.0
- [ ] Publish to crates.io
- [ ] Create GitHub release with binaries
