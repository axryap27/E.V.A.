use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::process::Command;
use tracing::{info, warn};

use super::ToolResult;

#[derive(Deserialize)]
struct CommandArgs {
    command: String,
}

#[derive(Deserialize)]
struct AppleScriptArgs {
    script: String,
}

pub async fn execute_command(arguments: &Value) -> Result<ToolResult> {
    let args: CommandArgs = serde_json::from_value(arguments.clone())
        .context("Invalid command arguments")?;

    info!("⚡ Executing shell command: {}", args.command);

    // Safety check: block potentially dangerous commands
    let dangerous_patterns = ["rm -rf /", "sudo", "chmod -R", "mkfs", "> /dev/"];
    for pattern in &dangerous_patterns {
        if args.command.contains(pattern) {
            warn!("⚠️ Blocked dangerous command: {}", args.command);
            return Ok(ToolResult {
                success: false,
                output: "Command blocked for safety reasons.".to_string(),
            });
        }
    }

    match Command::new("sh")
        .arg("-c")
        .arg(&args.command)
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let result = if !stdout.is_empty() {
                stdout.to_string()
            } else if !stderr.is_empty() {
                stderr.to_string()
            } else {
                "Command executed successfully (no output)".to_string()
            };

            info!("✓ Command complete");
            Ok(ToolResult {
                success: output.status.success(),
                output: result.trim().to_string(),
            })
        }
        Err(e) => {
            Ok(ToolResult {
                success: false,
                output: format!("Failed to execute command: {}", e),
            })
        }
    }
}

pub async fn execute_applescript(arguments: &Value) -> Result<ToolResult> {
    let args: AppleScriptArgs = serde_json::from_value(arguments.clone())
        .context("Invalid AppleScript arguments")?;

    info!("🍎 Executing AppleScript");

    match Command::new("osascript")
        .arg("-e")
        .arg(&args.script)
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let result = if !stdout.is_empty() {
                stdout.to_string()
            } else if !stderr.is_empty() {
                format!("AppleScript error: {}", stderr)
            } else {
                "AppleScript executed successfully".to_string()
            };

            info!("✓ AppleScript complete");
            Ok(ToolResult {
                success: output.status.success(),
                output: result.trim().to_string(),
            })
        }
        Err(e) => {
            Ok(ToolResult {
                success: false,
                output: format!("Failed to execute AppleScript: {}", e),
            })
        }
    }
}
