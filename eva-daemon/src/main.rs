mod ai;
mod tools;
// mod audio;  // TODO: Fix cpal threading issues

use ai::LlamaEngine;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, error, Level};

#[derive(Deserialize)]
struct CommandRequest {
    text: String,
}

#[derive(Serialize)]
struct CommandResponse {
    response: String,
}

#[derive(Clone)]
struct AppState {
    llama: Arc<LlamaEngine>,
}

async fn health() -> &'static str {
    "E.V.A. is online ✓"
}

async fn process_command(
    State(state): State<AppState>,
    Json(payload): Json<CommandRequest>,
) -> Json<CommandResponse> {
    info!("📥 Command: {}", payload.text);
    let text = payload.text.to_lowercase();

    // Check for tool-requiring commands FIRST (before LLM)
    let response = if text.contains("search") && (text.contains("web") || text.contains("for") || text.contains("find")) {
        // Extract search query
        let query = extract_search_query(&payload.text);
        info!("🔍 Detected web search request: {}", query);

        use crate::tools::{ToolCall, execute_tool};
        let tool_call = ToolCall {
            name: "web_search".to_string(),
            arguments: serde_json::json!({"query": query}),
        };

        match execute_tool(&tool_call).await {
            Ok(result) => {
                if result.success {
                    format!("I found this: {}", result.output)
                } else {
                    result.output
                }
            }
            Err(e) => format!("Search failed: {}", e),
        }
    } else if text.starts_with("run ") || text.starts_with("execute ") {
        // Extract command
        let cmd = payload.text.split_whitespace().skip(1).collect::<Vec<_>>().join(" ");
        info!("⚡ Detected command execution: {}", cmd);

        use crate::tools::{ToolCall, execute_tool};
        let tool_call = ToolCall {
            name: "run_command".to_string(),
            arguments: serde_json::json!({"command": cmd}),
        };

        match execute_tool(&tool_call).await {
            Ok(result) => result.output,
            Err(e) => format!("Command failed: {}", e),
        }
    } else if text.contains("list") && (text.contains("files") || text.contains("directory") || text.contains("folder")) {
        // Extract path or use home directory
        let path = extract_path(&payload.text).unwrap_or_else(|| std::env::var("HOME").unwrap_or_else(|_| "/Users/aarya".to_string()));
        info!("📁 Listing directory: {}", path);

        use crate::tools::{ToolCall, execute_tool};
        let tool_call = ToolCall {
            name: "list_directory".to_string(),
            arguments: serde_json::json!({"path": path}),
        };

        match execute_tool(&tool_call).await {
            Ok(result) => result.output,
            Err(e) => format!("Failed to list directory: {}", e),
        }
    } else if text.contains("screenshot") || text.contains("screen capture") {
        info!("📸 Taking screenshot");
        use crate::tools::{ToolCall, execute_tool};
        let tool_call = ToolCall {
            name: "screenshot".to_string(),
            arguments: serde_json::json!({}),
        };
        match execute_tool(&tool_call).await {
            Ok(result) => result.output,
            Err(e) => format!("Screenshot failed: {}", e),
        }
    } else if text.contains("clipboard") || text.contains("what's copied") {
        info!("📋 Reading clipboard");
        use crate::tools::{ToolCall, execute_tool};
        let tool_call = ToolCall {
            name: "get_clipboard".to_string(),
            arguments: serde_json::json!({}),
        };
        match execute_tool(&tool_call).await {
            Ok(result) => result.output,
            Err(e) => format!("Clipboard read failed: {}", e),
        }
    } else if text.contains("active window") || text.contains("current app") || text.contains("focused window") {
        info!("🪟 Getting active window");
        use crate::tools::{ToolCall, execute_tool};
        let tool_call = ToolCall {
            name: "active_window".to_string(),
            arguments: serde_json::json!({}),
        };
        match execute_tool(&tool_call).await {
            Ok(result) => result.output,
            Err(e) => format!("Failed: {}", e),
        }
    } else if text.contains("running apps") || text.contains("open apps") {
        info!("📱 Getting running apps");
        use crate::tools::{ToolCall, execute_tool};
        let tool_call = ToolCall {
            name: "running_apps".to_string(),
            arguments: serde_json::json!({}),
        };
        match execute_tool(&tool_call).await {
            Ok(result) => result.output,
            Err(e) => format!("Failed: {}", e),
        }
    } else if text.contains("time") {
        format!("The current time is {}", chrono::Local::now().format("%I:%M %p"))
    } else if text.contains("date") {
        format!("Today is {}", chrono::Local::now().format("%A, %B %d, %Y"))
    } else {
        // Use Llama for general conversation (NO tool definitions in prompt)
        match state.llama.generate(&payload.text).await {
            Ok(response) => response,
            Err(e) => {
                error!("❌ Llama error: {}", e);
                "Sorry, I encountered an error processing your request.".to_string()
            }
        }
    };

    Json(CommandResponse { response })
}

fn extract_search_query(text: &str) -> String {
    let lower = text.to_lowercase();

    // Try to extract after "search for", "search", "find", etc.
    if let Some(pos) = lower.find("search for ") {
        return text[pos + 11..].trim().to_string();
    }
    if let Some(pos) = lower.find("search ") {
        return text[pos + 7..].trim().to_string();
    }
    if let Some(pos) = lower.find("find ") {
        return text[pos + 5..].trim().to_string();
    }

    // Fallback: use the whole text
    text.trim().to_string()
}

fn extract_path(text: &str) -> Option<String> {
    // Look for paths starting with / or ~
    for word in text.split_whitespace() {
        if word.starts_with('/') || word.starts_with('~') {
            return Some(word.replace('~', &std::env::var("HOME").unwrap_or_else(|_| "/Users/aarya".to_string())));
        }
    }
    None
}

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    println!("\n╔═══════════════════════════════════════╗");
    println!("║                                       ║");
    println!("║     E.V.A. Daemon Starting...         ║");
    println!("║     Embedded Virtual Assistant        ║");
    println!("║                                       ║");
    println!("╚═══════════════════════════════════════╝\n");

    // Connect to Ollama
    let llama = match LlamaEngine::new("llama3.1:8b") {
        Ok(engine) => {
            info!("✓ Connected to Ollama");
            Arc::new(engine)
        }
        Err(e) => {
            error!("❌ Failed to connect to Ollama: {}", e);
            error!("   Make sure Ollama is installed and running:");
            error!("   brew install ollama && ollama serve");
            error!("   ollama pull llama3.1:8b");
            std::process::exit(1);
        }
    };

    // TODO: Implement wake word detection
    info!("⚠️  Voice activity detection disabled (coming soon)");

    let state = AppState { llama };

    let app = Router::new()
        .route("/health", get(health))
        .route("/command", post(process_command))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8765));

    println!("║     http://127.0.0.1:8765      ║");
    println!("║                                ║");
    println!("╚════════════════════════════════╝\n");

    info!("🚀 Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();
}