use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;
use std::process::Command;
use tracing::info;

use super::ToolResult;

/// Get current clipboard content
pub async fn get_clipboard(_arguments: &Value) -> Result<ToolResult> {
    info!("📋 Reading clipboard");

    let output = Command::new("pbpaste")
        .output()?;

    if output.status.success() {
        let content = String::from_utf8_lossy(&output.stdout).to_string();
        let preview = if content.len() > 200 {
            format!("{}... ({} chars total)", &content[..200], content.len())
        } else {
            content.clone()
        };

        Ok(ToolResult {
            success: true,
            output: format!("Clipboard content:\n{}", preview),
        })
    } else {
        Ok(ToolResult {
            success: false,
            output: "Failed to read clipboard".to_string(),
        })
    }
}

/// Set clipboard content
#[derive(Deserialize)]
struct SetClipboardArgs {
    content: String,
}

pub async fn set_clipboard(arguments: &Value) -> Result<ToolResult> {
    let args: SetClipboardArgs = serde_json::from_value(arguments.clone())?;

    info!("📋 Writing to clipboard");

    let output = Command::new("pbcopy")
        .arg(&args.content)
        .output()?;

    if output.status.success() {
        Ok(ToolResult {
            success: true,
            output: format!("Copied to clipboard: {} chars", args.content.len()),
        })
    } else {
        Ok(ToolResult {
            success: false,
            output: "Failed to write to clipboard".to_string(),
        })
    }
}
