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
        ai::AiFormatter, ai_json::AiJsonFormatter, classic::ClassicFormatter, csv::CsvFormatter, 
        digest::DigestFormatter, hex::HexFormatter, json::JsonFormatter, stats::StatsFormatter, 
        tsv::TsvFormatter, Formatter, PathDisplayMode,
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

    /// Don't use default ignore patterns (node_modules, __pycache__, etc.)
    #[arg(long)]
    no_default_ignore: bool,

    /// Show hidden files and directories (starting with .)
    #[arg(long, short = 'a')]
    all: bool,

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

    /// Path display mode
    #[arg(long = "path-mode", value_enum, default_value = "off")]
    path_mode: PathMode,

    /// When to use colors
    #[arg(long, value_enum, default_value = "auto")]
    color: ColorMode,

    /// Embed AI output in JSON structure (only applies to AI mode)
    #[arg(long)]
    ai_json: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ColorMode {
    /// Always use colors
    Always,
    /// Never use colors
    Never,
    /// Use colors if output is a terminal (default)
    Auto,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum PathMode {
    /// Show only filenames (default)
    Off,
    /// Show relative paths from scan root
    Relative,
    /// Show full absolute paths
    Full,
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
    /// Super compact digest format (hash + minimal stats)
    Digest,
}

fn parse_date(date_str: &str) -> Result<SystemTime> {
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
    let datetime = date.and_hms_opt(0, 0, 0).unwrap();
    Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(datetime.and_utc().timestamp() as u64))
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Check environment variables for defaults
    let default_mode = std::env::var("STREE_DEFAULT_MODE")
        .ok()
        .and_then(|m| match m.to_lowercase().as_str() {
            "classic" => Some(OutputMode::Classic),
            "hex" => Some(OutputMode::Hex),
            "json" => Some(OutputMode::Json),
            "ai" => Some(OutputMode::Ai),
            "stats" => Some(OutputMode::Stats),
            "csv" => Some(OutputMode::Csv),
            "tsv" => Some(OutputMode::Tsv),
            "digest" => Some(OutputMode::Digest),
            _ => None,
        });

    // Check for AI_TOOLS environment variable (highest priority)
    let (mode, compress) = if std::env::var("AI_TOOLS").is_ok() {
        (OutputMode::Ai, true)
    } else if let Some(default) = default_mode {
        (default, args.compress)
    } else {
        (args.mode, args.compress)
    };

    // Check color settings (command line takes precedence over env vars)
    let use_color = match args.color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => {
            if std::env::var("STREE_COLOR").as_deref() == Ok("always") {
                true
            } else if std::env::var("NO_COLOR").is_ok() || std::env::var("STREE_COLOR").as_deref() == Ok("never") {
                false
            } else {
                atty::is(atty::Stream::Stdout)
            }
        }
    };
    
    if !use_color {
        colored::control::set_override(false);
    }

    // Build scanner configuration
    // For AI mode, automatically enable show_ignored to give full context
    let show_ignored = args.show_ignored || matches!(mode, OutputMode::Ai);
    
    let config = ScannerConfig {
        max_depth: args.depth,
        follow_symlinks: false,
        respect_gitignore: !args.no_ignore,
        show_hidden: args.all,
        show_ignored,
        find_pattern: args.find.as_ref().map(|p| Regex::new(p)).transpose()?,
        file_type_filter: args.filter_type,
        min_size: args.min_size.as_ref().map(|s| parse_size(s)).transpose()?,
        max_size: args.max_size.as_ref().map(|s| parse_size(s)).transpose()?,
        newer_than: args.newer_than.as_ref().map(|d| parse_date(d)).transpose()?,
        older_than: args.older_than.as_ref().map(|d| parse_date(d)).transpose()?,
        use_default_ignores: !args.no_default_ignore,
    };

    // Create scanner and scan directory
    let scanner = Scanner::new(&args.path, config)?;
    let (nodes, stats) = scanner.scan()?;

    // Convert PathMode to PathDisplayMode
    let path_display_mode = match args.path_mode {
        PathMode::Off => PathDisplayMode::Off,
        PathMode::Relative => PathDisplayMode::Relative,
        PathMode::Full => PathDisplayMode::Full,
    };

    // Create appropriate formatter
    // For classic mode, auto-switch to relative paths when using find (unless user specified otherwise)
    let formatter: Box<dyn Formatter> = match mode {
        OutputMode::Classic => {
            let classic_path_mode = if args.find.is_some() && matches!(args.path_mode, PathMode::Off) {
                PathDisplayMode::Relative
            } else {
                path_display_mode
            };
            Box::new(ClassicFormatter::new(args.no_emoji, use_color, classic_path_mode))
        },
        OutputMode::Hex => Box::new(HexFormatter::new(use_color, args.no_emoji, args.show_ignored, path_display_mode)),
        OutputMode::Json => Box::new(JsonFormatter::new(args.compact)),
        OutputMode::Ai => {
            if args.ai_json {
                Box::new(AiJsonFormatter::new(args.no_emoji, path_display_mode))
            } else {
                Box::new(AiFormatter::new(args.no_emoji, path_display_mode))
            }
        },
        OutputMode::Stats => Box::new(StatsFormatter::new()),
        OutputMode::Csv => Box::new(CsvFormatter::new()),
        OutputMode::Tsv => Box::new(TsvFormatter::new()),
        OutputMode::Digest => Box::new(DigestFormatter::new()),
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