// 🎸 The Cheet's Marqant CLI - "Compress your docs like a rockstar!" 🤘

use anyhow::Result;
use clap::{Parser, Subcommand};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use st::formatters::marqant::MarqantFormatter;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name = "mq")]
#[command(about = "Marqant (.mq) compression tool for markdown files")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Compress a markdown file to .mq format
    Compress {
        /// Input markdown file (or - for stdin)
        input: String,
        /// Output .mq file (or - for stdout)
        #[arg(short, long)]
        output: Option<String>,
        /// Enable zlib compression for extra size reduction
        #[arg(long)]
        zlib: bool,
        /// Add semantic section tags
        #[arg(long)]
        semantic: bool,
    },
    /// Decompress a .mq file back to markdown
    Decompress {
        /// Input .mq file (or - for stdin)
        input: String,
        /// Output markdown file (or - for stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Show compression statistics
    Stats {
        /// Input markdown file
        input: String,
    },
    /// Inspect a .mq file with visual diagnostics
    Inspect {
        /// Input .mq file
        input: String,
    },
    /// Aggregate multiple markdown files into a single .mq
    Aggregate {
        /// Root directory to search for markdown files
        #[arg(default_value = ".")]
        path: String,
        /// Output .mq file (defaults to [project-name].mq)
        #[arg(short, long)]
        output: Option<String>,
        /// Maximum depth to search
        #[arg(long, default_value = "10")]
        max_depth: usize,
        /// Pattern to match (default: *.md)
        #[arg(long, default_value = "*.md")]
        pattern: String,
        /// Paths to exclude
        #[arg(long)]
        exclude: Vec<String>,
        /// Enable zlib compression
        #[arg(long)]
        zlib: bool,
        /// Add semantic section tags
        #[arg(long)]
        semantic: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compress {
            input,
            output,
            zlib,
            semantic,
        } => {
            let content = if input == "-" {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                buffer
            } else {
                fs::read_to_string(&input)?
            };

            // Build flags string
            let mut flags = Vec::new();
            if zlib {
                flags.push("-zlib");
            }
            if semantic {
                flags.push("-semantic");
            }
            let flags_str = if flags.is_empty() {
                None
            } else {
                Some(flags.join(" "))
            };

            let compressed = if let Some(ref f) = flags_str {
                MarqantFormatter::compress_markdown_with_flags(&content, Some(f))?
            } else {
                MarqantFormatter::compress_markdown(&content)?
            };

            if let Some(output_path) = output {
                if output_path == "-" {
                    io::stdout().write_all(compressed.as_bytes())?;
                } else {
                    fs::write(&output_path, compressed)?;
                    println!("✅ Compressed {} -> {}", input, output_path);
                }
            } else {
                // Default output filename
                let output_path = if input == "-" {
                    "output.mq".to_string()
                } else {
                    input.replace(".md", ".mq")
                };
                fs::write(&output_path, compressed)?;
                println!("✅ Compressed {} -> {}", input, output_path);
            }
        }

        Commands::Decompress { input, output } => {
            let compressed = if input == "-" {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                buffer
            } else {
                fs::read_to_string(&input)?
            };

            let decompressed = MarqantFormatter::decompress_marqant(&compressed)?;

            if let Some(output_path) = output {
                if output_path == "-" {
                    io::stdout().write_all(decompressed.as_bytes())?;
                } else {
                    fs::write(&output_path, decompressed)?;
                    println!("✅ Decompressed {} -> {}", input, output_path);
                }
            } else {
                // Default output filename
                let output_path = if input == "-" {
                    "output.md".to_string()
                } else {
                    input.replace(".mq", ".md")
                };
                fs::write(&output_path, decompressed)?;
                println!("✅ Decompressed {} -> {}", input, output_path);
            }
        }

        Commands::Stats { input } => {
            let content = fs::read_to_string(&input)?;
            let compressed = MarqantFormatter::compress_markdown(&content)?;

            let original_size = content.len();
            let compressed_size = compressed.len();
            let ratio =
                (original_size as f64 - compressed_size as f64) / original_size as f64 * 100.0;

            println!("📊 Marqant Compression Statistics");
            println!("──────────────────────────────────");
            println!("Original size:    {:>8} bytes", original_size);
            println!("Compressed size:  {:>8} bytes", compressed_size);
            println!("Compression:      {:>8.1}%", ratio);
            println!(
                "Reduction factor: {:>8.1}x",
                original_size as f64 / compressed_size as f64
            );

            // Count tokens
            let token_count = compressed
                .lines()
                .filter(|line| line.contains('=') && !line.starts_with("MARQANT"))
                .count();
            println!("Tokens used:      {:>8}", token_count);
        }

        Commands::Inspect { input } => {
            let content = fs::read_to_string(&input)?;
            let lines: Vec<&str> = content.lines().collect();

            if lines.is_empty() || !lines[0].starts_with("MARQANT_V1") {
                return Err(anyhow::anyhow!("Not a valid marqant file"));
            }

            // Parse header
            let header_parts: Vec<&str> = lines[0].split_whitespace().collect();
            if header_parts.len() < 4 {
                return Err(anyhow::anyhow!("Invalid marqant header"));
            }

            let timestamp = header_parts[1];
            let original_size: usize = header_parts[2].parse()?;
            let compressed_size: usize = header_parts[3].parse()?;
            let flags = if header_parts.len() > 4 {
                header_parts[4..].join(" ")
            } else {
                "none".to_string()
            };

            // Count tokens and find sections
            let mut token_count = 0;
            let mut most_frequent_token = String::new();
            let mut most_frequent_pattern = String::new();
            let mut sections = Vec::new();

            for line in &lines[1..] {
                if line == &"---" {
                    break;
                }
                if line.contains('=') {
                    token_count += 1;
                    if let Some((token, pattern)) = line.split_once('=') {
                        if token.len() > most_frequent_token.len() {
                            most_frequent_token = token.to_string();
                            most_frequent_pattern = pattern.replace("\\n", "↵").to_string();
                        }
                    }
                }
            }

            // Look for section tags in content
            for line in &lines {
                if line.contains("::section:") && line.ends_with("::") {
                    if let Some(section) = line
                        .strip_prefix("::section:")
                        .and_then(|s| s.strip_suffix("::"))
                    {
                        sections.push(section);
                    }
                }
            }

            // Calculate compression
            let ratio =
                (original_size as f64 - compressed_size as f64) / original_size as f64 * 100.0;

            // Display visual diagnostics
            println!("📄 File: {}", input);
            println!("📆 Modified: {}", timestamp);
            println!("📦 Compression: {:.1}%", ratio);
            println!("🔤 Dictionary Size: {} entries", token_count);
            if !most_frequent_token.is_empty() {
                println!(
                    "🧠 High-frequency token: {} (\"{}\")",
                    most_frequent_token, most_frequent_pattern
                );
            }
            if !sections.is_empty() {
                println!("📊 Sections: {}", sections.join(", "));
            }
            println!("🏳️  Flags: {}", flags);
        }

        Commands::Aggregate {
            path,
            output,
            max_depth,
            pattern: _,
            exclude,
            zlib,
            semantic,
        } => {
            let root_path = Path::new(&path);
            let project_name = root_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project");

            // Find all markdown files
            let mut markdown_files = Vec::new();
            let walker = WalkDir::new(&path).max_depth(max_depth);

            for entry in walker {
                let entry = entry?;
                let path = entry.path();

                // Skip if in exclude list
                let relative_path = path.strip_prefix(root_path).unwrap_or(path);
                let relative_str = relative_path.to_string_lossy();

                // Check each exclusion pattern
                let should_exclude = exclude.iter().any(|ex| {
                    // Simple glob-like matching: "vendor/*" matches "vendor/anything"
                    if ex.ends_with("/*") {
                        let prefix = &ex[..ex.len() - 2];
                        relative_str.starts_with(prefix)
                    } else {
                        relative_str.contains(ex)
                    }
                });

                if should_exclude {
                    continue;
                }

                // Check if it's a markdown file
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "md" {
                            markdown_files.push(path.to_path_buf());
                        }
                    }
                }
            }

            if markdown_files.is_empty() {
                println!("❌ No markdown files found in {}", path);
                return Ok(());
            }

            println!(
                "📚 Found {} markdown files to aggregate",
                markdown_files.len()
            );

            // Read all files and build combined content
            let mut all_content = String::new();
            let mut file_manifest = Vec::new();
            let mut total_original_size = 0;

            // Build combined content for tokenization
            let mut combined_for_analysis = String::new();
            for file_path in &markdown_files {
                let content = fs::read_to_string(file_path)?;
                combined_for_analysis.push_str(&content);
                combined_for_analysis.push_str("\n\n");
            }

            // Build flags
            let mut flags = vec!["-aggregate"];
            if zlib {
                flags.push("-zlib");
            }
            if semantic {
                flags.push("-semantic");
            }
            let flags_str = flags.join(" ");

            // Tokenize with combined dictionary
            let (tokens, _) = MarqantFormatter::tokenize_content(&combined_for_analysis);

            // Build header
            let timestamp = chrono::Utc::now().to_rfc3339();
            all_content.push_str(&format!(
                "MARQANT_V2 {} {} ",
                timestamp,
                combined_for_analysis.len()
            ));

            // Add manifest
            all_content.push_str("0 "); // Placeholder for compressed size
            all_content.push_str(&flags_str);
            all_content.push_str("\n::manifest::\n");

            let mut compressed_content = String::new();
            let mut current_offset = 0;

            // Process each file
            for file_path in &markdown_files {
                let content = fs::read_to_string(file_path)?;
                let relative_path = file_path
                    .strip_prefix(root_path)
                    .unwrap_or(file_path)
                    .to_string_lossy();

                total_original_size += content.len();

                // Add file marker
                compressed_content.push_str(&format!("::file:{}::\n", relative_path));

                // Tokenize this file with shared dictionary
                let mut tokenized = content.clone();
                for (token, pattern) in &tokens {
                    tokenized = tokenized.replace(pattern, token);
                }

                let start = current_offset;
                let length = tokenized.len();
                current_offset += length + relative_path.len() + 12; // Account for markers

                file_manifest.push(format!("{}:{}:{}", relative_path, start, length));
                compressed_content.push_str(&tokenized);
                compressed_content.push('\n');
            }

            // Write manifest
            for entry in &file_manifest {
                all_content.push_str(entry);
                all_content.push('\n');
            }
            all_content.push_str("::end-manifest::\n");

            // Write token dictionary
            for (token, pattern) in &tokens {
                let escaped_pattern = pattern.replace('\n', "\\n");
                all_content.push_str(&format!("{}={}\n", token, escaped_pattern));
            }
            all_content.push_str("---\n");

            // Add compressed content
            if zlib {
                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
                encoder.write_all(compressed_content.as_bytes())?;
                let compressed = encoder.finish()?;
                let encoded =
                    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &compressed);
                all_content.push_str(&encoded);
            } else {
                all_content.push_str(&compressed_content);
            }

            // Update compressed size in header
            let compressed_size = all_content.len();
            let size_placeholder = format!("{} {} 0", timestamp, combined_for_analysis.len());
            let size_actual = format!(
                "{} {} {}",
                timestamp,
                combined_for_analysis.len(),
                compressed_size
            );
            all_content = all_content.replace(&size_placeholder, &size_actual);

            // Determine output filename
            let output_path = output.unwrap_or_else(|| format!("{}.mq", project_name));

            // Write the aggregate file
            fs::write(&output_path, all_content)?;

            // Print summary
            let compression_ratio = (total_original_size as f64 - compressed_size as f64)
                / total_original_size as f64
                * 100.0;

            println!("✅ Created aggregate: {}", output_path);
            println!("📊 Aggregation Statistics:");
            println!("───────────────────────────");
            println!("Files aggregated: {}", markdown_files.len());
            println!("Original size:    {} bytes", total_original_size);
            println!("Compressed size:  {} bytes", compressed_size);
            println!("Compression:      {:.1}%", compression_ratio);
            println!("Shared tokens:    {}", tokens.len());

            // List included files
            println!("\n📄 Included files:");
            for (i, file) in markdown_files.iter().enumerate() {
                let relative = file.strip_prefix(root_path).unwrap_or(file);
                println!("  {}. {}", i + 1, relative.display());
                if i >= 9 && markdown_files.len() > 10 {
                    println!("  ... and {} more", markdown_files.len() - 10);
                    break;
                }
            }
        }
    }

    Ok(())
}

// 🎸 "Compression so good, even your README will sing!" - The Cheet
