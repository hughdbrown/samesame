# Open Questions

## Resolved Questions

These questions were clarified during planning:

| Question | Resolution |
|----------|------------|
| File input method | Both: explicit paths AND directory scanning with globs |
| Comparison mode | All pairs (N×N) - every file compared against every other |
| Line normalization | Strip whitespace from left and right before hashing |
| Output format | Both text and JSON, selectable via `--format` flag |
| LCS Algorithm | Use patience diff algorithm |
| Async runtime | Omit tokio; use rayon for parallelism only |
| Hash verification | Hash-only comparison is acceptable (no content verification) |
| Binary files | Skip binary files; only process text files |
| Match threshold | `-m` applies to each contiguous block (not total) |
| Large files | No size limits or warnings; process all files |
| Search behavior | Find ALL duplicates (don't stop on first match) |
| Exit codes | 0=no duplicates, 1=duplicates found, 2=error |
| Self-comparison | Skip matching a file against itself |
| Encoding | Lossy UTF-8 conversion for non-UTF-8 bytes |
| Symlinks | Skip them (avoid infinite loops) |
| Redundant pairs | Only compare (i, j) where i < j |
| Memory model | Keep all FileDescriptions in memory |
| Empty files | Skip them |
| Line length | No limits |

---

## Pending Questions

*All questions resolved - proceeding with implementation.*

### Algorithm Design

1. ~~**LCS Algorithm Choice**~~ **RESOLVED**: Use patience diff algorithm

2. ~~**Hash Collision Handling**~~ **RESOLVED**: Hash-only comparison is acceptable

3. ~~**Minimum Match Threshold Semantics**~~ **RESOLVED**: `-m 5` means each contiguous block must be >= 5 lines to be reported

### File Handling

4. ~~**Binary File Detection**~~ **RESOLVED**: Skip binary files; only process text files (detect via null bytes in first 8KB)

5. ~~**Large File Handling**~~ **RESOLVED**: No size limits or warnings; process all files regardless of size

6. **File Encoding**
   - The PRD doesn't specify encoding handling
   - **Recommendation**: Assume UTF-8, use `from_utf8_lossy` for non-UTF-8 bytes
   - **Decision needed**: Is lossy UTF-8 conversion acceptable?

7. **Symlink Handling**
   - Should symlinks be followed or skipped?
   - **Recommendation**: Skip symlinks to avoid infinite loops
   - **Decision needed**: Follow symlinks?

### Output & UX

8. ~~**Exit Codes & Search Behavior**~~ **RESOLVED**: Find ALL duplicates in files (don't stop on first match). Exit codes: 0=success/no duplicates, 1=duplicates found, 2=error

9. ~~**Self-Comparison**~~ **RESOLVED**: Skip matching a file against itself

10. **Duplicate Pair Reporting**
    - If A matches B, we have (A, B) in our pair list
    - Should we also check (B, A)? (Result would be symmetric)
    - **Recommendation**: Only compare (i, j) where i < j, skip redundant pairs
    - **Decision needed**: Confirm this optimization?

### Performance

11. ~~**Async vs Sync I/O**~~ **RESOLVED**: Omit tokio; use sync I/O with rayon for parallelism

12. **Memory vs Disk Tradeoff**
    - Should all FileDescriptions be kept in memory?
    - Or process pairs streaming and discard intermediate results?
    - **Recommendation**: Keep all in memory (typical codebases fit easily)
    - **Decision needed**: Memory-resident or streaming?

### Edge Cases

13. **Empty Files**
    - How to handle comparison with empty files?
    - **Recommendation**: Skip empty files in comparison (nothing to match)
    - **Decision needed**: Skip or include empty files?

14. **Single Line Files**
    - A file with one line can only have matches of length 0 or 1
    - With default `-m 5`, these would never report matches
    - **Recommendation**: This is correct behavior, no special handling
    - **Decision needed**: Confirm expected behavior?

15. **Very Long Lines**
    - Lines longer than 10KB? 1MB?
    - **Recommendation**: No line length limit, hash the entire line
    - **Decision needed**: Any line length restrictions?

---

## Questions for Future Versions

These are out of scope for v1.0.0 but worth tracking:

1. **Intra-file Duplicates**: Find duplicate blocks within the same file?
2. **Semantic Matching**: Ignore variable name differences?
3. **Threshold Tuning**: Suggest optimal `-m` value based on codebase?
4. **Visualization**: HTML report with side-by-side diff view?
5. **IDE Integration**: LSP server for live duplicate detection?
6. **Git Integration**: Ignore files in .gitignore?
7. **Language-Aware Parsing**: Use tree-sitter for AST-based comparison?
