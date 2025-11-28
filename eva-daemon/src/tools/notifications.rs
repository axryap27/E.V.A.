use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;
use std::process::Command;
use tracing::info;

use super::ToolResult;

#[derive(Deserialize)]
struct NotifyArgs {
    title: String,
    #[serde(default)]
    message: String,
}

/// Send a macOS notification
pub async fn send_notification(arguments: &Value) -> Result<ToolResult> {
    let args: NotifyArgs = serde_json::from_value(arguments.clone())?;

    info!("🔔 Sending notification: {}", args.title);

    let script = format!(
        r#"display notification "{}" with title "E.V.A." subtitle "{}""#,
        args.message.replace('"', "\\\""),
        args.title.replace('"', "\\\"")
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()?;

    if output.status.success() {
        Ok(ToolResult {
            success: true,
            output: format!("Notification sent: {}", args.title),
        })
    } else {
        Ok(ToolResult {
            success: false,
            output: "Failed to send notification".to_string(),
        })
    }
}

/// Show a dialog box (blocking)
#[derive(Deserialize)]
struct DialogArgs {
    message: String,
    #[serde(default = "default_dialog_title")]
    title: String,
}

fn default_dialog_title() -> String {
    "E.V.A.".to_string()
}

pub async fn show_dialog(arguments: &Value) -> Result<ToolResult> {
    let args: DialogArgs = serde_json::from_value(arguments.clone())?;

    info!("💬 Showing dialog: {}", args.title);

    let script = format!(
        r#"display dialog "{}" with title "{}" buttons {{"OK"}} default button "OK""#,
        args.message.replace('"', "\\\""),
        args.title.replace('"', "\\\"")
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()?;

    if output.status.success() {
        Ok(ToolResult {
            success: true,
            output: "Dialog shown".to_string(),
        })
    } else {
        Ok(ToolResult {
            success: false,
            output: "Failed to show dialog".to_string(),
        })
    }
}
