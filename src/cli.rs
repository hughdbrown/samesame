//! Command-line argument parsing.

use clap::{Parser, ValueEnum};
use regex::Regex;

/// Parse and validate min_match value (must be at least 1).
fn min_match_parser(s: &str) -> Result<usize, String> {
    let value: usize = s
        .parse()
        .map_err(|_| format!("'{s}' is not a valid number"))?;
    if value == 0 {
        Err("minimum match must be at least 1".to_string())
    } else {
        Ok(value)
    }
}

/// Parse and validate a regex pattern.
fn regex_parser(s: &str) -> Result<Regex, String> {
    Regex::new(s).map_err(|e| format!("invalid regex pattern: {e}"))
}

/// Output format options.
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum OutputFormat {
    /// Human-readable text output.
    #[default]
    Text,
    /// Structured JSON output.
    Json,
}

/// A tool to identify repeated fragments of code across multiple files.
#[derive(Parser, Debug)]
#[command(name = "samesame")]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Files to compare (can also use --dir or --glob).
    #[arg(value_name = "FILES")]
    pub files: Vec<String>,

    /// Minimum matching lines to report (must be at least 1).
    #[arg(short = 'm', long = "match", default_value = "5", value_parser = min_match_parser)]
    pub min_match: usize,

    /// Directory to scan for files.
    #[arg(short = 'd', long = "dir")]
    pub directory: Option<String>,

    /// Glob pattern for file matching (used with --dir).
    #[arg(short = 'g', long = "glob", default_value = "**/*.rs")]
    pub glob_pattern: String,

    /// Output format.
    #[arg(short = 'f', long = "format", value_enum, default_value = "text")]
    pub format: OutputFormat,

    /// Show detailed output.
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Suppress warnings.
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Only show matches where the first line matches this regex pattern.
    #[arg(short = 'r', long = "regex", value_parser = regex_parser)]
    pub regex: Option<Regex>,
}

impl Args {
    /// Parse command-line arguments.
    pub fn parse_args() -> Self {
        Args::parse()
    }
}
