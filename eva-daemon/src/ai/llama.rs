use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    system: String,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: i32,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

pub struct LlamaEngine {
    client: reqwest::Client,
    model: String,
    ollama_url: String,
}

impl LlamaEngine {
    pub fn new(model_name: &str) -> Result<Self> {
        info!("🧠 Connecting to Ollama...");

        Ok(Self {
            client: reqwest::Client::new(),
            model: model_name.to_string(),
            ollama_url: "http://localhost:11434/api/generate".to_string(),
        })
    }

    pub async fn generate(&self, prompt: &str) -> Result<String> {
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            system: "You are E.V.A. (Embedded Virtual Assistant), a helpful AI assistant. Be concise, friendly, and helpful. Keep responses under 3 sentences unless asked for detail.".to_string(),
            options: OllamaOptions {
                temperature: 0.7,
                num_predict: 150,
            },
        };

        let response = self
            .client
            .post(&self.ollama_url)
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Ollama")?;

        if !response.status().is_success() {
            anyhow::bail!("Ollama returned error: {}", response.status());
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        Ok(ollama_response.response.trim().to_string())
    }

    pub async fn check_health(&self) -> bool {
        matches!(
            self.client
                .get("http://localhost:11434/api/version")
                .send()
                .await,
            Ok(resp) if resp.status().is_success()
        )
    }
}
