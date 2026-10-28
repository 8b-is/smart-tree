//! Smart file reading with AST compression
//!
//! Contains smart_read handler and AST helper functions.

use super::definitions::SmartReadArgs;
use crate::mcp::{fmt_line, is_path_allowed, McpContext};
use anyhow::Result;
use regex::Regex;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Format a line number - uses centralized mcp::fmt_line
fn format_line_number(line: usize, hex: bool) -> String {
    fmt_line(line, hex)
}

/// Represents a collapsed function with its signature and body
#[derive(Debug, Clone)]
pub struct CollapsedFunction {
    pub name: String,
    pub signature: String,
    pub body: String,
    pub start_line: usize,
    pub end_line: usize,
    pub importance: f32,
}

/// Detects programming language from file extension
pub fn detect_language(path: &Path) -> Option<&'static str> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext| match ext.to_lowercase().as_str() {
            "rs" => Some("rust"),
            "py" => Some("python"),
            "js" | "jsx" | "mjs" => Some("javascript"),
            "ts" | "tsx" => Some("typescript"),
            "go" => Some("go"),
            "java" => Some("java"),
            "c" | "h" => Some("c"),
            "cpp" | "cc" | "cxx" | "hpp" => Some("cpp"),
            "rb" => Some("ruby"),
            "php" => Some("php"),
            "swift" => Some("swift"),
            "kt" | "kts" => Some("kotlin"),
            "cs" => Some("csharp"),
            "sh" | "bash" | "zsh" => Some("shell"),
            _ => None,
        })
}

/// Check if a language supports function collapsing
pub fn supports_collapsing(lang: &str) -> bool {
    matches!(
        lang,
        "rust" | "python" | "javascript" | "typescript" | "go" | "java" | "c" | "cpp"
    )
}

/// Extract functions from source code with improved regex patterns
pub fn extract_functions(source: &str, language: &str) -> Vec<CollapsedFunction> {
    let mut functions = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    match language {
        "rust" => {
            // Rust function pattern - handles pub, async, const, unsafe, extern
            let fn_pattern = Regex::new(
                r#"(?m)^[\s]*((?:pub(?:\s*\([^)]*\))?\s+)?(?:async\s+)?(?:const\s+)?(?:unsafe\s+)?(?:extern\s+"[^"]+"\s+)?fn\s+(\w+))"#
            ).expect("Static regex pattern should compile");

            for cap in fn_pattern.captures_iter(source) {
                if let (Some(full_sig), Some(name)) = (cap.get(1), cap.get(2)) {
                    let start_byte = full_sig.start();
                    let start_line = source[..start_byte].matches('\n').count();

                    // Find the opening brace and then match the closing one
                    if let Some(body_start) = source[start_byte..].find('{') {
                        let body_start_abs = start_byte + body_start;
                        if let Some((end_byte, _)) = find_matching_brace(&source[body_start_abs..])
                        {
                            let end_byte_abs = body_start_abs + end_byte;
                            let end_line = source[..end_byte_abs].matches('\n').count();

                            // Extract signature (up to opening brace)
                            let sig_end = source[start_byte..body_start_abs]
                                .rfind(|c: char| c != ' ' && c != '\t' && c != '\n')
                                .map(|i| start_byte + i + 1)
                                .unwrap_or(body_start_abs);
                            let signature = source[start_byte..sig_end].trim().to_string();

                            // Extract body
                            let body = source[body_start_abs..=end_byte_abs].to_string();

                            // Calculate importance
                            let importance = if name.as_str() == "main" {
                                1.0
                            } else if full_sig.as_str().contains("pub") {
                                0.9
                            } else if name.as_str().starts_with("test") {
                                0.3
                            } else {
                                0.6
                            };

                            functions.push(CollapsedFunction {
                                name: name.as_str().to_string(),
                                signature,
                                body,
                                start_line: start_line + 1,
                                end_line: end_line + 1,
                                importance,
                            });
                        }
                    }
                }
            }
        }
        "python" => {
            // Python function pattern - handles async, decorators captured separately
            let fn_pattern = Regex::new(r"(?m)^(\s*)(async\s+)?def\s+(\w+)\s*\([^)]*\)")
                .expect("Static Python regex should compile");

            for cap in fn_pattern.captures_iter(source) {
                if let (Some(indent_match), Some(name)) = (cap.get(1), cap.get(3)) {
                    let start_byte = cap.get(0).unwrap().start();
                    let start_line = source[..start_byte].matches('\n').count();
                    let indent = indent_match.as_str();
                    let indent_len = indent.len();

                    // Find end of function by indentation
                    let mut end_line = start_line;
                    let mut in_docstring = false;
                    let mut docstring_delim = "";

                    for (i, line) in lines.iter().enumerate().skip(start_line + 1) {
                        let trimmed = line.trim();

                        // Handle docstrings
                        if !in_docstring {
                            if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
                                in_docstring = true;
                                docstring_delim = if trimmed.starts_with("\"\"\"") {
                                    "\"\"\""
                                } else {
                                    "'''"
                                };
                                if trimmed.len() > 3 && trimmed[3..].contains(docstring_delim) {
                                    in_docstring = false;
                                }
                                continue;
                            }
                        } else if trimmed.contains(docstring_delim) {
                            in_docstring = false;
                            continue;
                        }

                        if in_docstring {
                            continue;
                        }

                        // Empty lines don't end the function
                        if trimmed.is_empty() {
                            continue;
                        }

                        // Check indentation
                        let line_indent = line.len() - line.trim_start().len();
                        if line_indent <= indent_len && !trimmed.is_empty() {
                            end_line = i.saturating_sub(1);
                            break;
                        }
                        end_line = i;
                    }

                    // Extract signature
                    let sig_end = source[start_byte..]
                        .find(':')
                        .map(|i| start_byte + i + 1)
                        .unwrap_or(start_byte + cap.get(0).unwrap().len());
                    let signature = source[start_byte..sig_end].trim().to_string();

                    // Extract body
                    let body_lines: Vec<&str> = lines[start_line..=end_line].to_vec();
                    let body = body_lines.join("\n");

                    // Calculate importance
                    let importance = if name.as_str() == "main" || name.as_str() == "__main__" {
                        1.0
                    } else if name.as_str() == "__init__" {
                        0.9
                    } else if name.as_str().starts_with("_") {
                        0.4
                    } else if name.as_str().starts_with("test") {
                        0.3
                    } else {
                        0.6
                    };

                    functions.push(CollapsedFunction {
                        name: name.as_str().to_string(),
                        signature,
                        body,
                        start_line: start_line + 1,
                        end_line: end_line + 1,
                        importance,
                    });
                }
            }
        }
        "javascript" | "typescript" => {
            // JS/TS function patterns - handles function declarations, arrow functions, methods
            let fn_pattern = Regex::new(
                r"(?m)^[\s]*((?:export\s+)?(?:async\s+)?function\s+(\w+)|(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?\([^)]*\)\s*=>)"
            ).expect("Static JS/TS regex should compile");

            for cap in fn_pattern.captures_iter(source) {
                let name = cap.get(2).or(cap.get(3));
                if let Some(name_match) = name {
                    let start_byte = cap.get(0).unwrap().start();
                    let start_line = source[..start_byte].matches('\n').count();

                    // Find opening brace
                    if let Some(body_start) = source[start_byte..].find('{') {
                        let body_start_abs = start_byte + body_start;
                        if let Some((end_byte, _)) = find_matching_brace(&source[body_start_abs..])
                        {
                            let end_byte_abs = body_start_abs + end_byte;
                            let end_line = source[..end_byte_abs].matches('\n').count();

                            let signature = source[start_byte..body_start_abs].trim().to_string();
                            let body = source[body_start_abs..=end_byte_abs].to_string();

                            let importance = if cap.get(0).unwrap().as_str().contains("export") {
                                0.9
                            } else {
                                0.6
                            };

                            functions.push(CollapsedFunction {
                                name: name_match.as_str().to_string(),
                                signature,
                                body,
                                start_line: start_line + 1,
                                end_line: end_line + 1,
                                importance,
                            });
                        }
                    }
                }
            }
        }
        _ => {
            // Generic C-style function pattern for other languages
            let fn_pattern = Regex::new(
                r"(?m)^[\s]*((?:public|private|protected|static|async|)\s*)(\w+)\s+(\w+)\s*\([^)]*\)\s*\{"
            ).expect("Static C-style regex should compile");

            for cap in fn_pattern.captures_iter(source) {
                if let Some(name) = cap.get(3) {
                    let start_byte = cap.get(0).unwrap().start();
                    let start_line = source[..start_byte].matches('\n').count();

                    if let Some(body_start) = source[start_byte..].find('{') {
                        let body_start_abs = start_byte + body_start;
                        if let Some((end_byte, _)) = find_matching_brace(&source[body_start_abs..])
                        {
                            let end_byte_abs = body_start_abs + end_byte;
                            let end_line = source[..end_byte_abs].matches('\n').count();

                            let signature = source[start_byte..body_start_abs].trim().to_string();
                            let body = source[body_start_abs..=end_byte_abs].to_string();

                            functions.push(CollapsedFunction {
                                name: name.as_str().to_string(),
                                signature,
                                body,
                                start_line: start_line + 1,
                                end_line: end_line + 1,
                                importance: 0.6,
                            });
                        }
                    }
                }
            }
        }
    }

    // Sort by line number
    functions.sort_by_key(|f| f.start_line);
    functions
}

/// Find matching closing brace, handling nested braces
pub fn find_matching_brace(s: &str) -> Option<(usize, usize)> {
    let mut depth = 0;
    let mut in_string = false;
    let mut string_char = ' ';
    let mut escaped = false;

    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        if c == '\\' {
            escaped = true;
            continue;
        }

        if in_string {
            if c == string_char {
                in_string = false;
            }
            continue;
        }

        match c {
            '"' | '\'' | '`' => {
                in_string = true;
                string_char = c;
            }
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((i, depth));
                }
            }
            _ => {}
        }
    }
    None
}

/// Check if a function should be expanded based on context keywords
pub fn should_expand_for_context(func: &CollapsedFunction, context_keywords: &[String]) -> bool {
    if context_keywords.is_empty() {
        return false;
    }

    let name_lower = func.name.to_lowercase();
    let body_lower = func.body.to_lowercase();

    for keyword in context_keywords {
        let kw_lower = keyword.to_lowercase();
        if name_lower.contains(&kw_lower) || body_lower.contains(&kw_lower) {
            return true;
        }
    }
    false
}

/// Main smart read handler
pub async fn smart_read(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let args: SmartReadArgs = serde_json::from_value(args)?;
    let path = PathBuf::from(&args.file_path);

    // Security check
    if !is_path_allowed(&path, &ctx.config) {
        return Err(anyhow::anyhow!("Path not allowed: {}", path.display()));
    }

    // Check if file exists
    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", path.display()));
    }

    if !path.is_file() {
        return Err(anyhow::anyhow!("Path is not a file: {}", path.display()));
    }

    // Read file content
    let content = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;

    // Detect language for smart compression
    let language = detect_language(&path);

    // Determine if we should compress - requires a known language that supports collapsing
    let compressible_lang = language.filter(|l| supports_collapsing(l));
    let should_compress = args.compress && !args.expand_all && compressible_lang.is_some();

    let (output, metadata) = if should_compress {
        // Safe: compressible_lang.is_some() guarantees we have a language
        let lang = compressible_lang.expect("Checked above");
        let functions = extract_functions(&content, lang);

        // Determine which functions to expand
        let expand_set: std::collections::HashSet<&str> =
            args.expand_functions.iter().map(|s| s.as_str()).collect();

        let mut output = String::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut current_line = 0;
        let mut collapsed_count = 0;
        let mut expanded_count = 0;

        // Track function references for the summary
        let mut function_refs: Vec<serde_json::Value> = Vec::new();

        // Use hex line numbers - defaults to MCP config (true for AI mode!)
        let use_hex = args.hex_line_numbers.unwrap_or(ctx.config.hex_numbers);

        for func in &functions {
            // Output lines before this function
            while current_line < func.start_line.saturating_sub(1) {
                if args.show_line_numbers {
                    output.push_str(&format!(
                        "{}│ {}\n",
                        format_line_number(current_line + 1, use_hex),
                        lines[current_line]
                    ));
                } else {
                    output.push_str(lines[current_line]);
                    output.push('\n');
                }
                current_line += 1;
            }

            // Check if this function should be expanded
            let should_expand = args.expand_all
                || expand_set.contains(func.name.as_str())
                || should_expand_for_context(func, &args.expand_context);

            if should_expand {
                // Output full function
                for i in func.start_line - 1..func.end_line {
                    if i < lines.len() {
                        if args.show_line_numbers {
                            output.push_str(&format!(
                                "{}│ {}\n",
                                format_line_number(i + 1, use_hex),
                                lines[i]
                            ));
                        } else {
                            output.push_str(lines[i]);
                            output.push('\n');
                        }
                    }
                }
                expanded_count += 1;
            } else {
                // Output collapsed function
                let body_lines = func.body.matches('\n').count() + 1;

                if args.show_line_numbers {
                    output.push_str(&format!(
                        "{}│ {} {{ ... }} // [fn:{}] {} lines collapsed\n",
                        format_line_number(func.start_line, use_hex),
                        func.signature,
                        func.name,
                        body_lines
                    ));
                } else {
                    output.push_str(&format!(
                        "{} {{ ... }} // [fn:{}] {} lines collapsed\n",
                        func.signature, func.name, body_lines
                    ));
                }

                // Use hex for line references too if enabled
                let lines_ref = if use_hex {
                    format!("{:X}-{:X}", func.start_line, func.end_line)
                } else {
                    format!("{}-{}", func.start_line, func.end_line)
                };

                function_refs.push(json!({
                    "name": func.name,
                    "ref": format!("[fn:{}]", func.name),
                    "lines": lines_ref,
                    "importance": func.importance
                }));

                collapsed_count += 1;
            }

            current_line = func.end_line;
        }

        // Output remaining lines after last function
        while current_line < lines.len() {
            if args.show_line_numbers {
                output.push_str(&format!(
                    "{}│ {}\n",
                    format_line_number(current_line + 1, use_hex),
                    lines[current_line]
                ));
            } else {
                output.push_str(lines[current_line]);
                output.push('\n');
            }
            current_line += 1;
        }

        let metadata = json!({
            "file_path": path.to_string_lossy(),
            "language": language,
            "compression_enabled": true,
            "hex_line_numbers": use_hex,
            "total_lines": lines.len(),
            "functions_found": functions.len(),
            "functions_collapsed": collapsed_count,
            "functions_expanded": expanded_count,
            "collapsed_refs": function_refs,
            "expand_hint": "Use expand_functions: ['fn_name'] or expand_context: ['keyword'] to expand specific functions"
        });

        (output, metadata)
    } else {
        // No compression - output raw content
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        // Use hex line numbers - defaults to MCP config (true for AI mode!)
        let use_hex = args.hex_line_numbers.unwrap_or(ctx.config.hex_numbers);

        let start_idx = args.offset.saturating_sub(1);
        let end_idx = if args.max_lines > 0 {
            (start_idx + args.max_lines).min(lines.len())
        } else {
            lines.len()
        };

        let mut output = String::new();
        for (i, line) in lines[start_idx..end_idx].iter().enumerate() {
            let line_num = start_idx + i + 1;
            if args.show_line_numbers {
                output.push_str(&format!(
                    "{}│ {}\n",
                    format_line_number(line_num, use_hex),
                    line
                ));
            } else {
                output.push_str(line);
                output.push('\n');
            }
        }

        let metadata = json!({
            "file_path": path.to_string_lossy(),
            "language": language,
            "compression_enabled": false,
            "hex_line_numbers": use_hex,
            "total_lines": total_lines,
            "lines_shown": end_idx - start_idx,
            "offset": args.offset,
            "has_more": end_idx < total_lines
        });

        (output, metadata)
    };

    Ok(json!({
        "content": [{
            "type": "text",
            "text": output
        }],
        "metadata": metadata
    }))
}
