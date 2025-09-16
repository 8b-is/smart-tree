// MCP Hook Management Tools - "Control your context flow!" 🎣
// Tools for managing Claude Code hooks via MCP
// "No more manual /hooks commands!" - Hue

use anyhow::Result;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Configuration for different hook types
#[derive(Debug, Clone)]
pub struct HookConfig {
    pub hook_type: String,
    pub command: String,
    pub enabled: bool,
    pub description: String,
}

/// List all configured hooks
pub async fn list_hooks(_params: Value) -> Result<Value> {
    let hooks = vec![
        HookConfig {
            hook_type: "UserPromptSubmit".to_string(),
            command: format!("{} --claude-user-prompt-submit", get_hook_path()),
            enabled: check_hook_enabled("UserPromptSubmit"),
            description: "Provides intelligent context based on user prompts".to_string(),
        },
        HookConfig {
            hook_type: "PreToolUse".to_string(),
            command: format!("{} --claude-pre-tool", get_hook_path()),
            enabled: check_hook_enabled("PreToolUse"),
            description: "Validates tool usage before execution".to_string(),
        },
        HookConfig {
            hook_type: "PostToolUse".to_string(),
            command: format!("{} --claude-post-tool", get_hook_path()),
            enabled: check_hook_enabled("PostToolUse"),
            description: "Processes tool results for better context".to_string(),
        },
        HookConfig {
            hook_type: "SessionStart".to_string(),
            command: format!("{} --claude-restore", get_hook_path()),
            enabled: check_hook_enabled("SessionStart"),
            description: "Restores consciousness from previous session".to_string(),
        },
    ];

    let mut result = Vec::new();
    for hook in hooks {
        result.push(json!({
            "type": hook.hook_type,
            "command": hook.command,
            "enabled": hook.enabled,
            "description": hook.description,
        }));
    }

    Ok(json!({
        "hooks": result,
        "hook_path": get_hook_path(),
        "hooks_file": get_hooks_file_path().display().to_string(),
    }))
}

/// Set or update a hook
pub async fn set_hook(params: Value) -> Result<Value> {
    let hook_type = params["hook_type"].as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing hook_type parameter"))?;

    // Map hook type to correct command flag
    let flag = match hook_type {
        "UserPromptSubmit" => "--claude-user-prompt-submit",
        "PreToolUse" => "--claude-pre-tool",
        "PostToolUse" => "--claude-post-tool",
        "SessionStart" => "--claude-restore",
        _ => "--claude-user-prompt-submit",
    };
    let default_command = format!("{} {}", get_hook_path(), flag);
    let command = params["command"].as_str()
        .unwrap_or(&default_command);

    let enabled = params["enabled"].as_bool().unwrap_or(true);

    // Read existing hooks configuration
    let hooks_file = get_hooks_file_path();
    let mut hooks_config = if hooks_file.exists() {
        let content = fs::read_to_string(&hooks_file)?;
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    // Update the specific hook
    hooks_config[hook_type] = json!({
        "command": command,
        "enabled": enabled,
        "updated_at": chrono::Utc::now().to_rfc3339(),
    });

    // Write back the configuration
    fs::write(&hooks_file, serde_json::to_string_pretty(&hooks_config)?)?;

    // If Claude Code is running, we might need to reload
    // For now, we'll just note that a restart might be needed

    Ok(json!({
        "success": true,
        "hook_type": hook_type,
        "command": command,
        "enabled": enabled,
        "message": if enabled {
            format!("Hook '{}' has been configured. You may need to restart Claude Code or use /hooks to apply.", hook_type)
        } else {
            format!("Hook '{}' has been disabled.", hook_type)
        }
    }))
}

/// Remove a hook
pub async fn remove_hook(params: Value) -> Result<Value> {
    let hook_type = params["hook_type"].as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing hook_type parameter"))?;

    // Read existing hooks configuration
    let hooks_file = get_hooks_file_path();
    if !hooks_file.exists() {
        return Ok(json!({
            "success": false,
            "message": "No hooks configuration found"
        }));
    }

    let mut hooks_config: Value = {
        let content = fs::read_to_string(&hooks_file)?;
        serde_json::from_str(&content)?
    };

    // Remove the hook
    if let Some(obj) = hooks_config.as_object_mut() {
        if obj.remove(hook_type).is_some() {
            // Write back the configuration
            fs::write(&hooks_file, serde_json::to_string_pretty(&hooks_config)?)?;

            return Ok(json!({
                "success": true,
                "message": format!("Hook '{}' has been removed. Use /hooks in Claude Code to update.", hook_type)
            }));
        }
    }

    Ok(json!({
        "success": false,
        "message": format!("Hook '{}' was not found", hook_type)
    }))
}

/// Test a hook with sample input
pub async fn test_hook(params: Value) -> Result<Value> {
    let hook_type = params["hook_type"].as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing hook_type parameter"))?;

    let test_input = params["input"].as_str()
        .unwrap_or("test input");

    // Determine the command based on hook type
    let command = match hook_type {
        "UserPromptSubmit" => format!("{} --claude-user-prompt-submit", get_hook_path()),
        "PreToolUse" => format!("{} --claude-pre-tool", get_hook_path()),
        "PostToolUse" => format!("{} --claude-post-tool", get_hook_path()),
        "SessionStart" => format!("{} --claude-restore", get_hook_path()),
        _ => {
            return Ok(json!({
                "success": false,
                "message": format!("Unknown hook type: {}", hook_type)
            }));
        }
    };

    // Prepare test input based on hook type
    let input_json = match hook_type {
        "UserPromptSubmit" => json!({
            "prompt": test_input
        }),
        "PreToolUse" => json!({
            "tool": "test_tool",
            "args": {"test": true}
        }),
        "PostToolUse" => json!({
            "tool": "test_tool",
            "result": {"success": true}
        }),
        _ => json!({})
    };

    // Execute the hook command with test input
    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .env("HOOK_TEST_MODE", "true")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let _ = stdin.write_all(input_json.to_string().as_bytes());
            }
            child.wait_with_output()
        });

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            Ok(json!({
                "success": output.status.success(),
                "exit_code": output.status.code(),
                "stdout": stdout.to_string(),
                "stderr": stderr.to_string(),
                "command": command,
                "input": input_json,
            }))
        }
        Err(e) => {
            Ok(json!({
                "success": false,
                "error": format!("Failed to execute hook: {}", e),
                "command": command,
            }))
        }
    }
}

/// Get hook commands for easy copying
pub async fn get_hook_commands(_params: Value) -> Result<Value> {
    let hook_path = get_hook_path();

    Ok(json!({
        "commands": {
            "UserPromptSubmit": format!("{} --claude-user-prompt-submit", hook_path),
            "PreToolUse": format!("{} --claude-pre-tool", hook_path),
            "PostToolUse": format!("{} --claude-post-tool", hook_path),
            "SessionStart": format!("{} --claude-restore", hook_path),
            "SessionEnd": format!("{} --claude-save", hook_path),
        },
        "instructions": "Copy the command for the hook you want and paste it in Claude Code's /hooks command",
        "hook_path": hook_path,
    }))
}

// Helper functions

fn get_hook_path() -> String {
    // Look for st-hook wrapper script first
    if let Ok(current_dir) = env::current_dir() {
        let st_hook = current_dir.join("st-hook");
        if st_hook.exists() {
            return st_hook.display().to_string();
        }
    }

    // Check in smart-tree directory
    let smart_tree_hook = PathBuf::from("/aidata/ayeverse/smart-tree/st-hook");
    if smart_tree_hook.exists() {
        return smart_tree_hook.display().to_string();
    }

    // Try installed location
    if let Ok(output) = Command::new("which").arg("st-hook").output() {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }

    // Fall back to st binary
    get_st_path()
}

fn get_st_path() -> String {
    // For now, use a dedicated hook binary path
    // In the future this should be integrated into st itself
    // Try to find st-hook binary first
    if let Ok(output) = Command::new("which").arg("st-hook").output() {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }

    // Try regular st in PATH
    if let Ok(output) = Command::new("which").arg("st").output() {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }

    // Otherwise use the full path
    if let Ok(current_dir) = env::current_dir() {
        let st_release = current_dir.join("target/release/st");
        if st_release.exists() {
            return st_release.display().to_string();
        }
    }

    // Fallback to ayeverse location
    "/aidata/ayeverse/smart-tree/target/release/st".to_string()
}

fn get_hooks_file_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("hooks.json")
}

fn check_hook_enabled(hook_type: &str) -> bool {
    let hooks_file = get_hooks_file_path();
    if !hooks_file.exists() {
        return false;
    }

    if let Ok(content) = fs::read_to_string(&hooks_file) {
        if let Ok(config) = serde_json::from_str::<Value>(&content) {
            return config[hook_type]["enabled"].as_bool().unwrap_or(false);
        }
    }

    false
}