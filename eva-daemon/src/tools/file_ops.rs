use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::Path;
use tracing::info;

use super::ToolResult;

#[derive(Deserialize)]
struct ReadFileArgs {
    path: String,
}

#[derive(Deserialize)]
struct WriteFileArgs {
    path: String,
    content: String,
}

#[derive(Deserialize)]
struct ListDirArgs {
    path: String,
}

pub async fn read_file(arguments: &Value) -> Result<ToolResult> {
    let args: ReadFileArgs = serde_json::from_value(arguments.clone())
        .context("Invalid read_file arguments")?;

    info!("📄 Reading file: {}", args.path);

    match fs::read_to_string(&args.path) {
        Ok(content) => {
            // Limit output to prevent overwhelming the LLM
            let truncated = if content.len() > 5000 {
                format!("{}... (truncated, {} total chars)", &content[..5000], content.len())
            } else {
                content
            };

            Ok(ToolResult {
                success: true,
                output: truncated,
            })
        }
        Err(e) => {
            Ok(ToolResult {
                success: false,
                output: format!("Failed to read file: {}", e),
            })
        }
    }
}

pub async fn write_file(arguments: &Value) -> Result<ToolResult> {
    let args: WriteFileArgs = serde_json::from_value(arguments.clone())
        .context("Invalid write_file arguments")?;

    info!("✍️ Writing file: {}", args.path);

    // Create parent directories if they don't exist
    if let Some(parent) = Path::new(&args.path).parent() {
        fs::create_dir_all(parent)?;
    }

    match fs::write(&args.path, &args.content) {
        Ok(_) => {
            Ok(ToolResult {
                success: true,
                output: format!("Successfully wrote {} bytes to {}", args.content.len(), args.path),
            })
        }
        Err(e) => {
            Ok(ToolResult {
                success: false,
                output: format!("Failed to write file: {}", e),
            })
        }
    }
}

pub async fn list_directory(arguments: &Value) -> Result<ToolResult> {
    let args: ListDirArgs = serde_json::from_value(arguments.clone())
        .context("Invalid list_directory arguments")?;

    info!("📁 Listing directory: {}", args.path);

    match fs::read_dir(&args.path) {
        Ok(entries) => {
            let mut files = Vec::new();
            let mut dirs = Vec::new();

            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if entry.path().is_dir() {
                    dirs.push(format!("📁 {}/", name));
                } else {
                    // Get file size
                    let size = entry.metadata()
                        .ok()
                        .map(|m| format_bytes(m.len()))
                        .unwrap_or_else(|| "?".to_string());
                    files.push(format!("📄 {} ({})", name, size));
                }
            }

            let mut output = String::new();
            if !dirs.is_empty() {
                output.push_str("Directories:\n");
                for dir in dirs {
                    output.push_str(&format!("  {}\n", dir));
                }
            }
            if !files.is_empty() {
                output.push_str("Files:\n");
                for file in files {
                    output.push_str(&format!("  {}\n", file));
                }
            }

            if output.is_empty() {
                output = "Directory is empty".to_string();
            }

            Ok(ToolResult {
                success: true,
                output: output.trim().to_string(),
            })
        }
        Err(e) => {
            Ok(ToolResult {
                success: false,
                output: format!("Failed to list directory: {}", e),
            })
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.1}{}", size, UNITS[unit_idx])
}
