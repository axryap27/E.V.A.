use anyhow::Result;
use std::process::Command;
use tracing::info;

/// Represents types of operations that may require consent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    /// Execute arbitrary shell commands - requires consent
    Command,
    /// Execute AppleScript - requires consent
    AppleScript,
    /// Write files - requires consent
    WriteFile,
    /// Read files - requires consent (for non-public paths)
    ReadFile,
    /// Screenshots - auto-approved (user preference)
    Screenshot,
    /// Clipboard access - auto-approved (user preference)
    Clipboard,
    /// App switching - requires consent
    SwitchApp,
}

impl OperationType {
    /// Check if this operation requires user consent
    pub fn requires_consent(&self) -> bool {
        matches!(
            self,
            Self::Command | Self::AppleScript | Self::WriteFile | Self::ReadFile | Self::SwitchApp
        )
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Command => "Execute Command",
            Self::AppleScript => "Run AppleScript",
            Self::WriteFile => "Write File",
            Self::ReadFile => "Read File",
            Self::Screenshot => "Take Screenshot",
            Self::Clipboard => "Access Clipboard",
            Self::SwitchApp => "Switch Application",
        }
    }
}

/// Ask user for consent via macOS dialog
pub async fn request_consent(op_type: OperationType, details: &str) -> Result<bool> {
    if !op_type.requires_consent() {
        return Ok(true);
    }

    info!("🔐 Requesting user consent for: {}", op_type.display_name());

    // Escape quotes for AppleScript
    let safe_details = details.replace('"', "\\\"").replace('\n', " ");
    let op_name = op_type.display_name();

    let script = format!(
        r#"display dialog "E.V.A. wants to:

{op_name}

{details}" buttons {{"Deny", "Allow"}} default button "Deny" with icon caution"#,
        op_name = op_name,
        details = safe_details
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()?;

    // Check if user clicked "Allow" (button 2)
    let result = String::from_utf8_lossy(&output.stdout);
    let allowed = result.contains("Allow");

    if allowed {
        info!("✓ User approved: {}", op_type.display_name());
    } else {
        info!("✗ User denied: {}", op_type.display_name());
    }

    Ok(allowed)
}

/// Request consent with command details for shell commands
pub async fn request_command_consent(command: &str) -> Result<bool> {
    let truncated = if command.len() > 100 {
        format!("{}...", &command[..100])
    } else {
        command.to_string()
    };

    request_consent(
        OperationType::Command,
        &format!("$ {}", truncated),
    )
    .await
}

/// Request consent for AppleScript execution
pub async fn request_applescript_consent(script: &str) -> Result<bool> {
    let truncated = if script.len() > 80 {
        format!("{}...", &script[..80])
    } else {
        script.to_string()
    };

    request_consent(
        OperationType::AppleScript,
        &format!("{}", truncated),
    )
    .await
}

/// Request consent for file write operation
pub async fn request_write_file_consent(path: &str) -> Result<bool> {
    request_consent(
        OperationType::WriteFile,
        &format!("Write to: {}", path),
    )
    .await
}

/// Request consent for file read operation
pub async fn request_read_file_consent(path: &str) -> Result<bool> {
    request_consent(
        OperationType::ReadFile,
        &format!("Read from: {}", path),
    )
    .await
}

/// Request consent for app switching
pub async fn request_switch_app_consent(app_name: &str) -> Result<bool> {
    request_consent(
        OperationType::SwitchApp,
        &format!("Switch to: {}", app_name),
    )
    .await
}
