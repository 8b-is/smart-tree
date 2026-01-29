//! Directory analysis tools
//!
//! Contains analyze_directory, quick_tree, project_overview, project_context_dump,
//! semantic_analysis, and related helper functions.

use super::definitions::{AnalyzeDirectoryArgs, ProjectContextDumpArgs};
use super::git::get_git_context;
use super::statistics::get_statistics;
use crate::formatters::{
    ai::AiFormatter, classic::ClassicFormatter, csv::CsvFormatter, digest::DigestFormatter,
    hex::HexFormatter, json::JsonFormatter, quantum::QuantumFormatter,
    quantum_semantic::QuantumSemanticFormatter, semantic::SemanticFormatter,
    stats::StatsFormatter, summary::SummaryFormatter, summary_ai::SummaryAiFormatter,
    tsv::TsvFormatter, Formatter, PathDisplayMode,
};
use crate::mcp::helpers::{
    scan_with_config, should_use_default_ignores, validate_and_convert_path, ScannerConfigBuilder,
};
use crate::mcp::McpContext;
use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;

/// Main directory analysis tool
pub async fn analyze_directory(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let args: AnalyzeDirectoryArgs = serde_json::from_value(args)?;
    let path = validate_and_convert_path(&args.path, &ctx)?;

    // Check cache if enabled
    let cache_key = format!(
        "{}:{}:{}:{}:{}:{}",
        path.display(),
        args.mode,
        args.max_depth,
        args.show_hidden,
        args.show_ignored,
        args.path_mode
    );

    if ctx.config.cache_enabled {
        if let Some(cached) = ctx.cache.get(&cache_key).await {
            return Ok(json!({
                "content": [{
                    "type": "text",
                    "text": cached
                }]
            }));
        }
    }

    // Build scanner configuration using builder
    let config = ScannerConfigBuilder::new()
        .max_depth(args.max_depth)
        .show_hidden(args.show_hidden)
        .show_ignored(args.show_ignored || args.mode == "ai")
        .use_default_ignores(should_use_default_ignores(&path))
        .build();

    // Special handling for home directory in MCP context
    if path.as_os_str() == std::env::var("HOME").unwrap_or_default().as_str() {
        eprintln!("⚠️  Note: Scanning home directory with safety limits enabled");
        eprintln!("   Maximum 100k files, 1 minute timeout for MCP operations");
    }

    // Scan directory
    let (nodes, stats) = scan_with_config(&path, config)?;

    // Convert path mode
    let path_display_mode = match args.path_mode.as_str() {
        "relative" => PathDisplayMode::Relative,
        "full" => PathDisplayMode::Full,
        _ => PathDisplayMode::Off,
    };

    // MCP optimizations: no emoji for clean output
    let mcp_no_emoji = true;

    // Compression logic:
    // 1. If user explicitly sets compress parameter, use that
    // 2. Otherwise, check MCP_NO_COMPRESS env var
    // 3. Default: false for ALL modes (decompressed by default)
    let default_compress = false;

    let mcp_compress = match args.compress {
        Some(compress) => compress,
        None => {
            if std::env::var("MCP_NO_COMPRESS")
                .is_ok_and(|v| v == "1" || v.to_lowercase() == "true")
            {
                false
            } else {
                default_compress
            }
        }
    };

    // Handle summary mode - auto-switch to AI version in MCP context
    let effective_mode = match args.mode.as_str() {
        "summary" => "summary-ai",
        other => other,
    };

    // Create formatter
    let formatter: Box<dyn Formatter> = match effective_mode {
        "classic" => Box::new(ClassicFormatter::new(mcp_no_emoji, true, path_display_mode)),
        "hex" => Box::new(HexFormatter::new(
            true,
            mcp_no_emoji,
            args.show_ignored,
            path_display_mode,
            false,
        )),
        "json" => Box::new(JsonFormatter::new(false)),
        "ai" => Box::new(AiFormatter::new(mcp_no_emoji, path_display_mode)),
        "stats" => Box::new(StatsFormatter::new()),
        "csv" => Box::new(CsvFormatter::new()),
        "tsv" => Box::new(TsvFormatter::new()),
        "digest" => Box::new(DigestFormatter::new()),
        "quantum" => Box::new(QuantumFormatter::new()),
        "semantic" => Box::new(SemanticFormatter::new(path_display_mode, mcp_no_emoji)),
        "quantum-semantic" => Box::new(QuantumSemanticFormatter::new()),
        "summary" => Box::new(SummaryFormatter::new(!mcp_no_emoji)),
        "summary-ai" => Box::new(SummaryAiFormatter::new(mcp_compress)),
        _ => return Err(anyhow::anyhow!("Invalid mode: {}", args.mode)),
    };

    // Format output
    let mut output = Vec::new();
    formatter.format(&mut output, &nodes, &stats, &path)?;

    // Handle different output formats
    let final_output = if args.mode == "quantum" || args.mode == "quantum-semantic" {
        // Quantum formats contain binary data, so base64-encode it for JSON safety
        use base64::{engine::general_purpose, Engine as _};
        format!(
            "QUANTUM_BASE64:{}",
            general_purpose::STANDARD.encode(&output)
        )
    } else {
        // For other formats, convert to string first
        let output_str = String::from_utf8_lossy(&output).to_string();

        // Use global compression manager for smart compression
        if mcp_compress || crate::compression_manager::should_compress_response(&output_str) {
            if args.mode == "semantic" {
                eprintln!("💡 Tip: Use mode:'quantum-semantic' for even better compression!");
            }
            use flate2::write::ZlibEncoder;
            use flate2::Compression;
            use std::io::Write;

            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(output_str.as_bytes())?;
            let compressed = encoder.finish()?;

            let compressed_size = compressed.len();
            let compression_ratio =
                100.0 - (compressed_size as f64 / output_str.len() as f64 * 100.0);
            eprintln!(
                "✅ Compressed: {} → {} bytes ({:.1}% reduction)",
                output_str.len(),
                compressed_size,
                compression_ratio
            );

            format!("COMPRESSED_V1:{}", hex::encode(&compressed))
        } else {
            output_str
        }
    };

    // Cache result if enabled
    if ctx.config.cache_enabled {
        ctx.cache.set(cache_key, final_output.clone()).await;
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": final_output
        }]
    }))
}

/// Quick 3-level directory overview
pub async fn quick_tree(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"].as_str().unwrap_or(".");

    // Get git context if available
    let git_info = get_git_context(path).await.unwrap_or_default();

    let analyze_args = json!({
        "path": path,
        "mode": "summary-ai",
        "max_depth": args["depth"].as_u64().unwrap_or(3),
        "compress": false,
        "show_ignored": true
    });

    let mut result = analyze_directory(analyze_args, ctx.clone()).await?;

    // Prepend git info to the result if available
    if !git_info.is_empty() {
        if let Some(content) = result["content"][0]["text"].as_str() {
            let enhanced_content = format!("{}\n{}", git_info, content);
            result["content"][0]["text"] = json!(enhanced_content);
        }
    }

    Ok(result)
}

/// Get a comprehensive project analysis
pub async fn project_overview(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;

    // Get git context if available
    let git_info = get_git_context(path).await.unwrap_or_default();

    // First get the summary-ai format overview (10x compression!)
    let ai_result = analyze_directory(
        json!({
            "path": path,
            "mode": "summary-ai",
            "max_depth": 5,
            "show_ignored": true
        }),
        ctx.clone(),
    )
    .await?;

    // Then get statistics
    let stats_result = get_statistics(
        json!({
            "path": path,
            "show_hidden": false
        }),
        ctx.clone(),
    )
    .await?;

    // Combine results
    let ai_text = ai_result["content"][0]["text"].as_str().unwrap_or("");
    let stats_text = stats_result["content"][0]["text"].as_str().unwrap_or("");

    // Build the final output with git info at the top
    let overview_text = if !git_info.is_empty() {
        format!(
            "PROJECT OVERVIEW\n\n{}\n\n{}\n\nDETAILED STATISTICS:\n{}",
            git_info, ai_text, stats_text
        )
    } else {
        format!(
            "PROJECT OVERVIEW\n\n{}\n\nDETAILED STATISTICS:\n{}",
            ai_text, stats_text
        )
    };

    Ok(json!({
        "content": [{
            "type": "text",
            "text": overview_text
        }]
    }))
}

/// Full project context dump for AI assistants
pub async fn project_context_dump(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let dump_args: ProjectContextDumpArgs = serde_json::from_value(args)?;
    let path = std::path::Path::new(&dump_args.path);

    let mut output_sections: Vec<String> = Vec::new();

    // Header
    output_sections.push("PROJECT_CONTEXT_DUMP_V1:".to_string());
    output_sections.push(format!("PATH:{}", path.display()));

    // 1. Git context (if enabled)
    if dump_args.include_git {
        let git_info = get_git_context(&dump_args.path).await.unwrap_or_default();
        if !git_info.is_empty() {
            output_sections.push(format!("GIT:{}", git_info.replace('\n', " | ")));
        }
    }

    // 2. Scan directory with configured depth
    let structure_mode = match dump_args.compression.as_str() {
        "quantum" => "quantum",
        _ => "summary-ai",
    };
    let content_compression = dump_args.compression.as_str();

    let scan_result = analyze_directory(
        json!({
            "path": dump_args.path,
            "mode": structure_mode,
            "max_depth": dump_args.max_depth,
            "show_ignored": true
        }),
        ctx.clone(),
    )
    .await?;

    let structure_text = scan_result["content"][0]["text"].as_str().unwrap_or("");

    // 3. Identify key files
    let key_files = identify_project_key_files(&dump_args.path).await;
    if !key_files.is_empty() {
        output_sections.push(format!("KEY_FILES:{}", key_files.join(",")));
    }

    // 4. Detect project type
    let project_type = detect_project_type_simple(&dump_args.path).await;
    output_sections.push(format!("TYPE:{}", project_type));

    // 5. Add directory structure
    output_sections.push(format!("STRUCTURE:\n{}", structure_text));

    // 6. Optionally include key file contents
    if dump_args.include_content {
        let content_budget = dump_args.token_budget / 3;
        let contents = read_key_files_content(
            &dump_args.path,
            &key_files,
            content_budget,
            content_compression,
        )
        .await;
        if !contents.is_empty() {
            output_sections.push(format!("FILE_CONTENTS:\n{}", contents));
        }
    }

    // Combine all sections
    let full_output = output_sections.join("\n");

    // Token estimation (rough: 1 token ≈ 4 chars)
    let estimated_tokens = full_output.len() / 4;

    // Add footer with token estimate
    let mut final_output = full_output;
    final_output.push_str(&format!(
        "\nEND_PROJECT_CONTEXT_DUMP\nTOKENS_EST:{:x}",
        estimated_tokens
    ));

    // Build metadata with warning if over budget
    let mut metadata = json!({
        "estimated_tokens": estimated_tokens,
        "compression_mode": dump_args.compression,
        "max_depth": dump_args.max_depth,
        "max_files": dump_args.max_files,
    });

    if estimated_tokens > dump_args.token_budget {
        metadata["warning"] = json!(format!(
            "Estimated tokens ({}) exceeds budget ({}). Consider: reducing max_depth, using 'quantum' compression, or disabling include_content",
            estimated_tokens, dump_args.token_budget
        ));
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": final_output
        }],
        "metadata": metadata
    }))
}

/// Identify key project files (README, CLAUDE.md, config files, entry points)
pub async fn identify_project_key_files(path: &str) -> Vec<String> {
    let priority_files = [
        "README.md",
        "README",
        "readme.md",
        "CLAUDE.md",
        ".claude/CLAUDE.md",
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "go.mod",
        "Makefile",
        "docker-compose.yml",
        "Dockerfile",
        "src/main.rs",
        "src/lib.rs",
        "src/index.ts",
        "src/index.js",
        "main.py",
        "app.py",
        "main.go",
        "index.js",
        "index.ts",
        ".env.example",
        "requirements.txt",
        "setup.py",
    ];

    let mut found = Vec::new();
    let base_path = std::path::Path::new(path);

    for file in &priority_files {
        let full_path = base_path.join(file);
        if full_path.exists() {
            found.push(file.to_string());
        }
    }

    found
}

/// Simple project type detection
pub async fn detect_project_type_simple(path: &str) -> String {
    let base_path = std::path::Path::new(path);

    // Check for language-specific markers
    if base_path.join("Cargo.toml").exists() {
        return "CODE[Rust]".to_string();
    }
    if base_path.join("package.json").exists() {
        if base_path.join("tsconfig.json").exists() {
            return "CODE[TypeScript]".to_string();
        }
        return "CODE[JavaScript]".to_string();
    }
    if base_path.join("pyproject.toml").exists() || base_path.join("setup.py").exists() {
        return "CODE[Python]".to_string();
    }
    if base_path.join("go.mod").exists() {
        return "CODE[Go]".to_string();
    }
    if base_path.join("Gemfile").exists() {
        return "CODE[Ruby]".to_string();
    }
    if base_path.join("pom.xml").exists() || base_path.join("build.gradle").exists() {
        return "CODE[Java]".to_string();
    }

    "MIXED".to_string()
}

/// Read contents of key files with token budget and optional compression
pub async fn read_key_files_content(
    path: &str,
    key_files: &[String],
    max_tokens: usize,
    compression: &str,
) -> String {
    use crate::formatters::marqant::MarqantFormatter;

    let mut output = String::new();
    let mut tokens_used = 0;
    let base_path = std::path::Path::new(path);

    // Priority order for content inclusion
    let content_priority = [
        "CLAUDE.md",
        ".claude/CLAUDE.md",
        "README.md",
        "README",
        "Cargo.toml",
        "package.json",
    ];

    for priority_file in &content_priority {
        if tokens_used >= max_tokens {
            break;
        }

        // Check if this file is in our key_files list
        if key_files
            .iter()
            .any(|f| f == *priority_file || f.ends_with(priority_file))
        {
            let file_path = base_path.join(priority_file);
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                // Apply compression based on mode
                let compressed_content = match compression {
                    "marqant" => {
                        if priority_file.ends_with(".md") {
                            MarqantFormatter::compress_markdown(&content)
                                .unwrap_or_else(|_| content.clone())
                        } else {
                            content.clone()
                        }
                    }
                    "quantum" => compress_file_quantum(&content, priority_file),
                    _ => content.clone(),
                };

                let file_tokens = compressed_content.len() / 4;

                // Truncate if would exceed budget
                let content_to_add = if tokens_used + file_tokens > max_tokens {
                    let remaining_chars = (max_tokens - tokens_used) * 4;
                    let truncate_at = remaining_chars.min(compressed_content.len());
                    let safe_truncate = compressed_content
                        .char_indices()
                        .take_while(|(i, _)| *i < truncate_at)
                        .last()
                        .map(|(i, c)| i + c.len_utf8())
                        .unwrap_or(0);
                    format!("{}...[TRUNCATED]", &compressed_content[..safe_truncate])
                } else {
                    compressed_content
                };

                let compression_tag = match compression {
                    "marqant" if priority_file.ends_with(".md") => "[MQ]",
                    "quantum" => "[Q]",
                    _ => "",
                };
                output.push_str(&format!(
                    "---FILE:{}{}---\n{}\n",
                    priority_file, compression_tag, content_to_add
                ));
                tokens_used += content_to_add.len() / 4;
            }
        }
    }

    output
}

/// Quantum compression for file contents - structure only, maximum reduction
pub fn compress_file_quantum(content: &str, filename: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let line_count = lines.len();

    if filename.ends_with(".md") {
        // For markdown: extract headers and first line of each section
        let mut result = String::new();
        let mut in_code_block = false;

        for line in &lines {
            if line.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }
            if line.starts_with('#') {
                result.push_str(line);
                result.push('\n');
            }
        }

        format!("Q[{}L]:\n{}", line_count, result)
    } else if filename.ends_with(".toml") || filename.ends_with(".json") {
        // For config files: extract top-level keys
        let mut keys = Vec::new();
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                keys.push(trimmed.to_string());
            } else if trimmed.contains('=') && !trimmed.starts_with('#') {
                if let Some(key) = trimmed.split('=').next() {
                    let key = key.trim();
                    if !key.contains(' ') && keys.len() < 20 {
                        keys.push(key.to_string());
                    }
                }
            } else if trimmed.starts_with('"') && trimmed.contains(':') {
                // JSON key
                if let Some(key) = trimmed.split(':').next() {
                    let key = key.trim().trim_matches('"');
                    if keys.len() < 20 {
                        keys.push(key.to_string());
                    }
                }
            }
        }
        format!("Q[{}L]:KEYS:{}", line_count, keys.join(","))
    } else {
        // For other files: first 5 and last 2 lines
        let preview: Vec<&str> = if line_count <= 10 {
            lines.clone()
        } else {
            let mut p = lines[..5].to_vec();
            p.push("...");
            p.extend_from_slice(&lines[line_count.saturating_sub(2)..]);
            p
        };
        format!("Q[{}L]:\n{}", line_count, preview.join("\n"))
    }
}

/// Semantic analysis using wave-based grouping
pub async fn semantic_analysis(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let max_depth = args["max_depth"].as_u64().unwrap_or(10) as usize;

    // Simply use analyze_directory with semantic mode
    analyze_directory(
        json!({
            "path": path,
            "mode": "semantic",
            "max_depth": max_depth,
            "no_emoji": false,
            "path_mode": "off"
        }),
        ctx,
    )
    .await
}
