//! samesame - A tool to identify repeated fragments of code across multiple files.

use std::process::ExitCode;

use rayon::prelude::*;

use samesame::cli::{Args, OutputFormat};
use samesame::diff::compare_files;
use samesame::discovery::{discover_files, generate_pairs};
use samesame::error::SameError;
use samesame::file::read_file_if_text;
use samesame::output::{format_json, format_text};
use samesame::types::{ComparisonResult, FileDescription};

fn main() -> ExitCode {
    let args = Args::parse_args();

    match run(&args) {
        Ok(has_duplicates) => {
            if has_duplicates {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::from(2)
        }
    }
}

fn run(args: &Args) -> Result<bool, SameError> {
    // Discover files
    let paths = discover_files(
        &args.files,
        args.directory.as_deref(),
        &args.glob_pattern,
    )?;

    if !args.quiet {
        eprintln!("Found {} files to analyze", paths.len());
    }

    // Read files in parallel, filtering out binary and empty files
    let files: Vec<FileDescription> = paths
        .par_iter()
        .filter_map(|path| {
            match read_file_if_text(path) {
                Ok(Some(desc)) => Some(desc),
                Ok(None) => {
                    // Binary or empty file - skip silently
                    None
                }
                Err(e) => {
                    if !args.quiet {
                        eprintln!("Warning: {}", e);
                    }
                    None
                }
            }
        })
        .collect();

    if files.is_empty() {
        return Err(SameError::NoFilesFound);
    }

    if !args.quiet {
        eprintln!("Loaded {} text files", files.len());
    }

    // Generate pairs for comparison
    let pairs = generate_pairs(files.len());
    let pairs_count = pairs.len();

    if !args.quiet {
        eprintln!("Comparing {} file pairs", pairs_count);
    }

    // Compare pairs in parallel
    let results: Vec<ComparisonResult<'_>> = pairs
        .par_iter()
        .map(|&(i, j)| compare_files(&files[i], &files[j]))
        .collect();

    // Check if any duplicates found
    let has_duplicates = results
        .iter()
        .any(|r| r.has_significant_matches(args.min_match));

    // Format and print output
    let output = match args.format {
        OutputFormat::Text => format_text(
            &results,
            args.min_match,
            args.verbose,
            files.len(),
            pairs_count,
            args.regex.as_ref(),
        ),
        OutputFormat::Json => format_json(
            &results,
            args.min_match,
            args.verbose,
            files.len(),
            pairs_count,
            args.regex.as_ref(),
        ),
    };

    println!("{}", output);

    Ok(has_duplicates)
}
