use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::{info, Level};

#[derive(Deserialize)]
struct CommandRequest {
    text: String,
}

#[derive(Serialize)]
struct CommandResponse {
    response: String,
}

async fn health() -> &'static str {
    "E.V.A. is online ✓"
}

async fn process_command(
    Json(payload): Json<CommandRequest>,
) -> Json<CommandResponse> {
    info!("📥 Command: {}", payload.text);
    
    // Smart responses based on command
    let response = match payload.text.to_lowercase().as_str() {
        text if text.contains("hello") || text.contains("hi") => {
            "Hello! I'm E.V.A. How can I help you today?".to_string()
        }
        text if text.contains("time") => {
            format!("The current time is {}", chrono::Local::now().format("%I:%M %p"))
        }
        text if text.contains("date") => {
            format!("Today is {}", chrono::Local::now().format("%A, %B %d, %Y"))
        }
        text if text.contains("weather") => {
            "I don't have access to weather data yet, but I'm learning!".to_string()
        }
        text if text.contains("help") || text.contains("what can you do") => {
            "I can help with:\n• Opening apps\n• Answering questions\n• System commands\n• And much more soon!".to_string()
        }
        _ => {
            format!("I heard you say: '{}'. I'm still learning how to respond to that!", payload.text)
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

    let app = Router::new()
        .route("/health", get(health))
        .route("/command", post(process_command))
        .layer(tower_http::cors::CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8765));
    
    println!("\n╔════════════════════════════════════════╗");
    println!("║                                        ║");
    println!("║     E.V.A. Daemon Starting...          ║");
    println!("║     Embedded Virtual Assistant         ║");
    println!("║                                        ║");
    println!("║     http://127.0.0.1:8765              ║");
    println!("║                                        ║");
    println!("╚════════════════════════════════════════╝\n");

    info!("🚀 Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap();
    
    axum::serve(listener, app)
        .await
        .unwrap();
}