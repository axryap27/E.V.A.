mod ai;

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

    // Handle special commands that don't need AI
    let response = match payload.text.to_lowercase().as_str() {
        text if text.contains("time") => {
            format!("The current time is {}", chrono::Local::now().format("%I:%M %p"))
        }
        text if text.contains("date") => {
            format!("Today is {}", chrono::Local::now().format("%A, %B %d, %Y"))
        }
        _ => {
            // Use Llama for general queries
            match state.llama.generate(&payload.text).await {
                Ok(response) => response,
                Err(e) => {
                    error!("❌ Llama error: {}", e);
                    "Sorry, I encountered an error processing your request.".to_string()
                }
            }
        }
    };

    Json(CommandResponse { response })
}

#[tokio::main]
async fn main() {
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
    let llama = match LlamaEngine::new("llama3.2:3b") {
        Ok(engine) => {
            info!("✓ Connected to Ollama");
            Arc::new(engine)
        }
        Err(e) => {
            error!("❌ Failed to connect to Ollama: {}", e);
            error!("   Make sure Ollama is installed and running:");
            error!("   brew install ollama && ollama serve");
            error!("   ollama pull llama3.2:3b");
            std::process::exit(1);
        }
    };

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