use anyhow::Result;
use chrono::NaiveDate;
use clap::{Parser, ValueEnum};
use colored;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use regex::Regex;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::SystemTime;

use stree::{
    formatters::{
        ai::AiFormatter, classic::ClassicFormatter, csv::CsvFormatter, hex::HexFormatter,
        json::JsonFormatter, stats::StatsFormatter, tsv::TsvFormatter, Formatter,
    },
    parse_size, Scanner, ScannerConfig,
};

#[derive(Parser, Debug)]
#[command(
    name = "stree",
    about = "Smart Tree - An intelligent directory visualization tool",
    version,
    author
)]
struct Args {
    /// Path to analyze
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output mode
    #[arg(short, long, value_enum, default_value = "classic")]
    mode: OutputMode,

    /// Find files/directories matching pattern
    #[arg(long)]
    find: Option<String>,

    /// Filter by file extension
    #[arg(long = "type")]
    filter_type: Option<String>,

    /// Minimum file size (e.g., "1M", "500K")
    #[arg(long)]
    min_size: Option<String>,

    /// Maximum file size
    #[arg(long)]
    max_size: Option<String>,

    /// Show files newer than date (YYYY-MM-DD)
    #[arg(long)]
    newer_than: Option<String>,

    /// Show files older than date (YYYY-MM-DD)
    #[arg(long)]
    older_than: Option<String>,

    /// Maximum depth to traverse
    #[arg(short, long, default_value = "10")]
    depth: usize,

    /// Don't respect .gitignore files
    #[arg(long)]
    no_ignore: bool,

    /// Show ignored directories in brackets
    #[arg(long)]
    show_ignored: bool,

    /// Disable emoji in output
    #[arg(long)]
    no_emoji: bool,

    /// Compress output with zlib
    #[arg(short = 'z', long)]
    compress: bool,

    /// Compact JSON output (no pretty printing)
    #[arg(long)]
    compact: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputMode {
    /// Classic tree format with metadata
    Classic,
    /// Hexadecimal format with fixed-width fields
    Hex,
    /// JSON output
    Json,
    /// AI-optimized format (hex + stats)
    Ai,
    /// Directory statistics only
    Stats,
    /// CSV format
    Csv,
    /// TSV format
    Tsv,
}

fn parse_date(date_str: &str) -> Result<SystemTime> {
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
    let datetime = date.and_hms_opt(0, 0, 0).unwrap();
    Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(datetime.and_utc().timestamp() as u64))
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Check for AI_TOOLS environment variable
    let (mode, compress) = if std::env::var("AI_TOOLS").is_ok() {
        (OutputMode::Ai, true)
    } else {
        (args.mode, args.compress)
    };

    // Check if NO_COLOR is set
    let use_color = std::env::var("NO_COLOR").is_err() && atty::is(atty::Stream::Stdout);
    if !use_color {
        colored::control::set_override(false);
    }

    // Build scanner configuration
    let config = ScannerConfig {
        max_depth: args.depth,
        follow_symlinks: false,
        respect_gitignore: !args.no_ignore,
        show_hidden: true, // Always scan hidden files, formatter decides whether to show
        find_pattern: args.find.as_ref().map(|p| Regex::new(p)).transpose()?,
        file_type_filter: args.filter_type,
        min_size: args.min_size.as_ref().map(|s| parse_size(s)).transpose()?,
        max_size: args.max_size.as_ref().map(|s| parse_size(s)).transpose()?,
        newer_than: args.newer_than.as_ref().map(|d| parse_date(d)).transpose()?,
        older_than: args.older_than.as_ref().map(|d| parse_date(d)).transpose()?,
    };

    // Create scanner and scan directory
    let scanner = Scanner::new(&args.path, config)?;
    let (nodes, stats) = scanner.scan()?;

    // Create appropriate formatter
    let formatter: Box<dyn Formatter> = match mode {
        OutputMode::Classic => Box::new(ClassicFormatter::new(args.no_emoji, use_color)),
        OutputMode::Hex => Box::new(HexFormatter::new(use_color, args.no_emoji, args.show_ignored)),
        OutputMode::Json => Box::new(JsonFormatter::new(args.compact)),
        OutputMode::Ai => Box::new(AiFormatter::new(args.no_emoji)),
        OutputMode::Stats => Box::new(StatsFormatter::new()),
        OutputMode::Csv => Box::new(CsvFormatter::new()),
        OutputMode::Tsv => Box::new(TsvFormatter::new()),
    };

    // Format output
    let mut output = Vec::new();
    formatter.format(&mut output, &nodes, &stats, &args.path)?;

    // Handle compression if requested
    if compress {
        let compressed = compress_output(&output)?;
        println!("COMPRESSED_V1:{}", hex::encode(&compressed));
    } else {
        io::stdout().write_all(&output)?;
    }

    Ok(())
}

fn compress_output(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}