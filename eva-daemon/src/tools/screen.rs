use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;
use std::process::Command;
use tracing::info;

use super::ToolResult;
use super::consent;

#[derive(Deserialize)]
struct ScreenshotArgs {
    #[serde(default = "default_path")]
    path: String,
}

fn default_path() -> String {
    format!("/tmp/eva_screenshot_{}.png", chrono::Utc::now().timestamp())
}

/// Take a screenshot of the entire screen or a specific window
pub async fn take_screenshot(arguments: &Value) -> Result<ToolResult> {
    let args: ScreenshotArgs = serde_json::from_value(arguments.clone())
        .unwrap_or_else(|_| ScreenshotArgs {
            path: default_path(),
        });

    info!("📸 Taking screenshot: {}", args.path);

    // Use macOS screencapture command
    let output = Command::new("screencapture")
        .arg("-x") // Don't play sound
        .arg(&args.path)
        .output()?;

    if output.status.success() {
        Ok(ToolResult {
            success: true,
            output: format!("Screenshot saved to: {}", args.path),
        })
    } else {
        Ok(ToolResult {
            success: false,
            output: format!("Failed to capture screenshot: {}", String::from_utf8_lossy(&output.stderr)),
        })
    }
}

/// Get information about currently focused window
pub async fn get_active_window(_arguments: &Value) -> Result<ToolResult> {
    info!("🪟 Getting active window info");

    let script = r#"
        tell application "System Events"
            set frontApp to name of first application process whose frontmost is true
            set frontWindow to name of first window of application process frontApp
            return frontApp & " - " & frontWindow
        end tell
    "#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()?;

    if output.status.success() {
        let window_info = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(ToolResult {
            success: true,
            output: format!("Active window: {}", window_info),
        })
    } else {
        Ok(ToolResult {
            success: false,
            output: "Failed to get active window info".to_string(),
        })
    }
}

/// Get list of all running applications
pub async fn get_running_apps(_arguments: &Value) -> Result<ToolResult> {
    info!("📱 Getting running applications");

    let script = r#"
        tell application "System Events"
            set appList to name of every application process whose visible is true
            return appList as string
        end tell
    "#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()?;

    if output.status.success() {
        let apps = String::from_utf8_lossy(&output.stdout)
            .trim()
            .replace(", ", "\n- ");
        Ok(ToolResult {
            success: true,
            output: format!("Running apps:\n- {}", apps),
        })
    } else {
        Ok(ToolResult {
            success: false,
            output: "Failed to get running applications".to_string(),
        })
    }
}

/// Switch to a specific application
#[derive(Deserialize)]
struct SwitchAppArgs {
    app_name: String,
}

pub async fn switch_to_app(arguments: &Value) -> Result<ToolResult> {
    let args: SwitchAppArgs = serde_json::from_value(arguments.clone())?;

    info!("🔄 Switching to app: {}", args.app_name);

    // Request user consent before switching apps
    match consent::request_switch_app_consent(&args.app_name).await {
        Ok(false) => {
            info!("⚠️ App switch denied by user");
            return Ok(ToolResult {
                success: false,
                output: "App switch denied by user.".to_string(),
            });
        }
        Err(e) => {
            tracing::warn!("Failed to show consent dialog: {}", e);
            return Ok(ToolResult {
                success: false,
                output: format!("Failed to request permission: {}", e),
            });
        }
        Ok(true) => {} // User approved, continue
    }

    let script = format!(
        r#"tell application "{}" to activate"#,
        args.app_name
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()?;

    if output.status.success() {
        Ok(ToolResult {
            success: true,
            output: format!("Switched to {}", args.app_name),
        })
    } else {
        Ok(ToolResult {
            success: false,
            output: format!("Failed to switch to {}", args.app_name),
        })
    }
}
