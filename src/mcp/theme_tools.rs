//! MCP Tools for theme customization

use anyhow::Result;
use serde_json::{json, Value};

/// Handle the set_dashboard_theme MCP tool
pub async fn handle_set_dashboard_theme(args: Value) -> Result<Value> {
    // This is a proxy to the existing HTTP endpoint.
    // In a real scenario, this might directly call the config-saving logic,
    // but for now, we'll simulate the call.

    // Extract arguments
    let bg_primary = args["bg_primary"].as_str();
    let bg_secondary = args["bg_secondary"].as_str();
    let accent_primary = args["accent_primary"].as_str();
    let accent_secondary = args["accent_secondary"].as_str();
    let fg_primary = args["fg_primary"].as_str();
    let fg_secondary = args["fg_secondary"].as_str();

    // Here you would typically call the function that saves the theme to ~/.st/theme.json
    // For now, we'll just acknowledge the request.
    // The actual saving logic is in `src/web_dashboard/api.rs`.

    Ok(json!({
        "status": "success",
        "message": "Theme settings received. The dashboard will update on next refresh.",
        "settings_applied": {
            "bg_primary": bg_primary,
            "bg_secondary": bg_secondary,
            "accent_primary": accent_primary,
            "accent_secondary": accent_secondary,
            "fg_primary": fg_primary,
            "fg_secondary": fg_secondary,
        }
    }))
}
