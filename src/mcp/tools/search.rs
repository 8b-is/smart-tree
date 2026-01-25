//! Search and find tools
//!
//! Contains find_files, find_code_files, find_config_files, find_documentation,
//! find_large_files, find_recent_changes, find_in_timespan, find_tests, find_build_files,
//! find_duplicates, find_empty_directories, find_projects, search_in_files handlers.

use super::definitions::FindFilesArgs;
use crate::formatters::projects::ProjectsFormatter;
use crate::formatters::Formatter;
use crate::mcp::helpers::{
    scan_with_config, should_use_default_ignores, validate_and_convert_path, ScannerConfigBuilder,
};
use crate::mcp::{fmt_num, fmt_num64, is_path_allowed, McpContext};
use crate::parse_size;
use anyhow::Result;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

/// Powerful file search with regex patterns, size filters, and date ranges
pub async fn find_files(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let args: FindFilesArgs = serde_json::from_value(args)?;
    let path = validate_and_convert_path(&args.path, &ctx)?;

    // Parse dates - use local timezone (no panics on invalid time!)
    let parse_date = |date_str: &str| -> Result<SystemTime> {
        use chrono::{Local, NaiveDate, TimeZone};
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
        let naive_time = date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid time 00:00:00"))?;
        let datetime = Local
            .from_local_datetime(&naive_time)
            .single()
            .ok_or_else(|| anyhow::anyhow!("Invalid local datetime"))?;
        Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(datetime.timestamp() as u64))
    };

    // Parse end date as end of day (23:59:59) for inclusive range
    let parse_end_date = |date_str: &str| -> Result<SystemTime> {
        use chrono::{Local, NaiveDate, TimeZone};
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
        let naive_time = date
            .and_hms_opt(23, 59, 59)
            .ok_or_else(|| anyhow::anyhow!("Invalid time 23:59:59"))?;
        let datetime = Local
            .from_local_datetime(&naive_time)
            .single()
            .ok_or_else(|| anyhow::anyhow!("Invalid local datetime"))?;
        Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(datetime.timestamp() as u64))
    };

    // Build scanner configuration using builder
    let config = ScannerConfigBuilder::new()
        .max_depth(args.max_depth)
        .show_hidden(true)
        .find_pattern(args.pattern.as_ref().map(|p| Regex::new(p)).transpose()?)
        .file_type_filter(args.file_type)
        .entry_type_filter(args.entry_type)
        .min_size(args.min_size.as_ref().map(|s| parse_size(s)).transpose()?)
        .max_size(args.max_size.as_ref().map(|s| parse_size(s)).transpose()?)
        .newer_than(
            args.newer_than
                .as_ref()
                .map(|d| parse_date(d))
                .transpose()?,
        )
        .older_than(
            args.older_than
                .as_ref()
                .map(|d| parse_end_date(d))
                .transpose()?,
        )
        .use_default_ignores(should_use_default_ignores(&path))
        .build();

    // Scan directory
    let (nodes, _stats) = scan_with_config(&path, config)?;

    // Format results as JSON list
    let mut results = Vec::new();
    for node in &nodes {
        // Skip the root directory itself
        if node.path == path {
            continue;
        }

        // Use hex formatting for token efficiency!
        let use_hex = ctx.config.hex_numbers;
        let modified_secs = node
            .modified
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs();

        results.push(json!({
            "path": node.path.display().to_string(),
            "name": node.path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
            "size": fmt_num64(node.size, use_hex),
            "modified": fmt_num64(modified_secs, use_hex),
            "permissions": format!("{:o}", node.permissions),
            "is_directory": node.is_dir,
        }));
    }

    let use_hex = ctx.config.hex_numbers;
    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&json!({
                "found": fmt_num(results.len(), use_hex),
                "files": results
            }))?
        }]
    }))
}

/// Find all source code files by programming language
pub async fn find_code_files(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let languages = args["languages"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_else(|| vec!["all"]);

    let extensions = if languages.contains(&"all") {
        vec![
            "py", "js", "ts", "tsx", "jsx", "rs", "go", "java", "cpp", "c", "h", "hpp", "rb",
            "php", "swift", "kt", "scala", "r", "jl", "cs", "vb", "lua", "pl", "sh", "bash", "zsh",
            "ps1", "dart", "elm", "ex", "exs", "clj", "cljs", "ml", "mli",
        ]
    } else {
        let mut exts = Vec::new();
        for lang in languages {
            match lang {
                "python" => exts.extend(&["py", "pyw", "pyx"]),
                "javascript" => exts.extend(&["js", "mjs", "cjs"]),
                "typescript" => exts.extend(&["ts", "tsx"]),
                "rust" => exts.push("rs"),
                "go" => exts.push("go"),
                "java" => exts.push("java"),
                "cpp" => exts.extend(&["cpp", "cxx", "cc", "c++", "hpp", "h", "hxx"]),
                "c" => exts.extend(&["c", "h"]),
                "ruby" => exts.push("rb"),
                "php" => exts.push("php"),
                "swift" => exts.push("swift"),
                "kotlin" => exts.extend(&["kt", "kts"]),
                "scala" => exts.extend(&["scala", "sc"]),
                "r" => exts.push("r"),
                "julia" => exts.push("jl"),
                _ => {}
            }
        }
        exts
    };

    let pattern = format!(r"\.({})$", extensions.join("|"));
    find_files(
        json!({
            "path": path,
            "pattern": pattern,
            "max_depth": 20
        }),
        ctx,
    )
    .await
}

/// Locate all configuration files
pub async fn find_config_files(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;

    let pattern =
        r"\.(json|yaml|yml|toml|ini|cfg|conf|config|env|properties|xml)$|^\..*rc$|^.*config.*$";
    find_files(
        json!({
            "path": path,
            "pattern": pattern,
            "max_depth": 10
        }),
        ctx,
    )
    .await
}

/// Discover all projects across a filesystem
pub async fn find_projects(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let depth = args["depth"].as_i64().unwrap_or(10) as usize;

    // Check permissions
    if !is_path_allowed(&path, &ctx.config) {
        return Ok(json!({
            "error": "Path not allowed by security settings"
        }));
    }

    // Create scanner config with projects mode depth - limit to 3 for testing
    let config = ScannerConfigBuilder::new()
        .max_depth(depth.min(3))
        .use_default_ignores(true)
        .show_hidden(false)
        .respect_gitignore(false)
        .build();

    // Scan for all files
    let (nodes, stats) = scan_with_config(&path, config)?;

    // Use the ProjectsFormatter to find and format projects
    let formatter = ProjectsFormatter::new();
    let mut buffer = Vec::new();
    formatter.format(&mut buffer, &nodes, &stats, &path)?;

    // Parse the output and convert to JSON
    let output = String::from_utf8_lossy(&buffer);

    // Extract project info from the formatted output
    let mut projects = Vec::new();
    let mut current_project = None;

    for line in output.lines() {
        if line.starts_with("[") && line.contains("] ") {
            // New project line starts with [HASH]
            if let Some(proj) = current_project.take() {
                projects.push(proj);
            }

            // Parse project line
            if let Some(idx) = line.find("] ") {
                let after_hash = &line[idx + 2..];
                let name_start = after_hash
                    .chars()
                    .position(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                    .unwrap_or(0);
                let name = after_hash[name_start..].trim().to_string();

                current_project = Some(json!({
                    "name": name,
                    "hash": line[1..idx].to_string(),
                    "details": Vec::<String>::new()
                }));
            }
        } else if line.starts_with("  ") && current_project.is_some() {
            // Project detail
            if let Some(proj) = current_project.as_mut() {
                if let Some(details) = proj.get_mut("details") {
                    if let Some(arr) = details.as_array_mut() {
                        arr.push(json!(line.trim()));
                    }
                }
            }
        }
    }

    // Add the last project
    if let Some(proj) = current_project {
        projects.push(proj);
    }

    let use_hex = ctx.config.hex_numbers;
    Ok(json!({
        "projects": projects,
        "count": fmt_num(projects.len(), use_hex),
        "search_path": path.display().to_string(),
        "max_depth": fmt_num(depth, use_hex)
    }))
}

/// Find all documentation files
pub async fn find_documentation(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;

    let pattern = r"(README|readme|CHANGELOG|changelog|LICENSE|license|CONTRIBUTING|contributing|TODO|todo|INSTALL|install|AUTHORS|authors|NOTICE|notice|HISTORY|history)(\.(md|markdown|rst|txt|adoc|org))?$|\.(md|markdown|rst|txt|adoc|org)$";
    find_files(
        json!({
            "path": path,
            "pattern": pattern,
            "max_depth": 10
        }),
        ctx,
    )
    .await
}

/// Search for keywords within files
pub async fn search_in_files(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path_str = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let path = validate_and_convert_path(path_str, &ctx)?;

    let keyword = args["keyword"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing keyword"))?;
    let file_type = args["file_type"].as_str();
    let _case_sensitive = args["case_sensitive"].as_bool().unwrap_or(false);
    let include_content = args["include_content"].as_bool().unwrap_or(true);
    let context_lines = args["context_lines"].as_u64().map(|n| n as usize);
    let max_matches_per_file = args["max_matches_per_file"].as_u64().unwrap_or(20) as usize;

    // Build scanner configuration using builder
    let config = ScannerConfigBuilder::for_search(&path)
        .file_type_filter(file_type.map(String::from))
        .search_keyword(Some(keyword.to_string()))
        .include_line_content(include_content)
        .build();

    let (nodes, _) = scan_with_config(&path, config)?;

    // Format results showing files with matches
    let use_hex = ctx.config.hex_numbers;
    let mut results = Vec::new();
    for node in &nodes {
        if let Some(matches) = &node.search_matches {
            let mut file_result = json!({
                "path": node.path.display().to_string(),
                "matches": fmt_num(matches.total_count, use_hex),
                "truncated": matches.truncated
            });

            // Include line content if available
            if let Some(ref lines) = matches.line_content {
                let mut line_results = Vec::new();
                for (line_num, content, column) in lines.iter().take(max_matches_per_file) {
                    let line_obj = json!({
                        "line": fmt_num(*line_num, use_hex),
                        "content": content,
                        "col": fmt_num(*column, use_hex)
                    });

                    if let Some(_ctx_lines) = context_lines {
                        // TODO: Add context lines before and after
                    }

                    line_results.push(line_obj);
                }
                file_result["lines"] = json!(line_results);
            }

            results.push(file_result);
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&json!({
                "keyword": keyword,
                "files_with_matches": fmt_num(results.len(), use_hex),
                "include_content": include_content,
                "max_per_file": fmt_num(max_matches_per_file, use_hex),
                "results": results
            }))?
        }]
    }))
}

/// Find files larger than a threshold
pub async fn find_large_files(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let min_size = args["min_size"].as_str().unwrap_or("10M");

    find_files(
        json!({
            "path": path,
            "min_size": min_size,
            "max_depth": 20
        }),
        ctx,
    )
    .await
}

/// Find files modified within the last N days
pub async fn find_recent_changes(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let days = args["days"].as_u64().unwrap_or(7);

    // Calculate date N days ago
    use chrono::{Duration, Utc};
    let date = Utc::now() - Duration::days(days as i64);
    let date_str = date.format("%Y-%m-%d").to_string();

    find_files(
        json!({
            "path": path,
            "newer_than": date_str,
            "max_depth": 20
        }),
        ctx,
    )
    .await
}

/// Find files modified within a specific time range
pub async fn find_in_timespan(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let start_date = args["start_date"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing start_date"))?;

    // Build the find_files request
    let mut find_args = json!({
        "path": path,
        "newer_than": start_date,
        "max_depth": 20
    });

    if let Some(end_date) = args["end_date"].as_str() {
        find_args["older_than"] = json!(end_date);
    }

    if let Some(file_type) = args["file_type"].as_str() {
        find_args["file_type"] = json!(file_type);
    }

    find_files(find_args, ctx.clone()).await
}

/// Find potential duplicate files
pub async fn find_duplicates(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path_str = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let path = validate_and_convert_path(path_str, &ctx)?;

    // Get all files using builder
    let config = ScannerConfigBuilder::new()
        .max_depth(20)
        .use_default_ignores(should_use_default_ignores(&path))
        .build();

    let (nodes, _) = scan_with_config(&path, config)?;

    // Group files by size and name
    let mut size_groups: HashMap<u64, Vec<&crate::scanner::FileNode>> = HashMap::new();

    for node in &nodes {
        if !node.is_dir {
            size_groups.entry(node.size).or_default().push(node);
        }
    }

    // Find potential duplicates with hex formatting
    let use_hex = ctx.config.hex_numbers;
    let mut duplicates = Vec::new();
    for (size, files) in size_groups.iter() {
        if files.len() > 1 && *size > 0 {
            duplicates.push(json!({
                "sz": fmt_num64(*size, use_hex),
                "n": fmt_num(files.len(), use_hex),
                "files": files.iter().map(|f| f.path.display().to_string()).collect::<Vec<_>>()
            }));
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&json!({
                "groups": fmt_num(duplicates.len(), use_hex),
                "dups": duplicates
            }))?
        }]
    }))
}

/// Locate all test files
pub async fn find_tests(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;

    let pattern = r"(test_|_test\.|\.test\.|tests?\.|spec\.|\.spec\.|_spec\.)|(/tests?/|/specs?/)";
    find_files(
        json!({
            "path": path,
            "pattern": pattern,
            "max_depth": 20
        }),
        ctx,
    )
    .await
}

/// Find all build configuration files
pub async fn find_build_files(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;

    let pattern = r"^(Makefile|makefile|CMakeLists\.txt|Cargo\.toml|package\.json|pom\.xml|build\.gradle|build\.sbt|setup\.py|requirements\.txt|Gemfile|go\.mod|composer\.json|Dockerfile|docker-compose\.yml)$";
    find_files(
        json!({
            "path": path,
            "pattern": pattern,
            "max_depth": 10
        }),
        ctx,
    )
    .await
}

/// Find all empty directories
pub async fn find_empty_directories(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path_str = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let path = validate_and_convert_path(path_str, &ctx)?;

    let config = ScannerConfigBuilder::new()
        .max_depth(20)
        .use_default_ignores(should_use_default_ignores(&path))
        .build();

    let (nodes, _) = scan_with_config(&path, config)?;

    // Find directories with no children
    let mut empty_dirs = Vec::new();
    let mut dir_children: std::collections::HashMap<PathBuf, usize> =
        std::collections::HashMap::new();

    // Count children for each directory
    for node in &nodes {
        if let Some(parent) = node.path.parent() {
            *dir_children.entry(parent.to_path_buf()).or_insert(0) += 1;
        }
    }

    // Find empty directories
    for node in &nodes {
        if node.is_dir {
            let child_count = dir_children.get(&node.path).unwrap_or(&0);
            if *child_count == 0 {
                empty_dirs.push(node.path.display().to_string());
            }
        }
    }

    let use_hex = ctx.config.hex_numbers;
    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&json!({
                "count": fmt_num(empty_dirs.len(), use_hex),
                "dirs": empty_dirs
            }))?
        }]
    }))
}
