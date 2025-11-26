use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, warn};

use crate::tools::{self, ToolCall};

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
        self.generate_with_context(prompt, None).await
    }

    pub async fn generate_with_context(&self, prompt: &str, context: Option<String>) -> Result<String> {
        let system_prompt = format!(
            "You are E.V.A. (Embedded Virtual Assistant), a helpful AI assistant with powerful capabilities.\n\n{}\n\nBe concise and helpful. If you need to use a tool, output ONLY the JSON tool call. Otherwise, respond naturally to the user.",
            tools::get_tools_definition()
        );

        let full_prompt = if let Some(ctx) = context {
            format!("{}\n\nUser: {}", ctx, prompt)
        } else {
            prompt.to_string()
        };

        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: full_prompt,
            stream: false,
            system: system_prompt,
            options: OllamaOptions {
                temperature: 0.7,
                num_predict: 300,
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

        let response_text = ollama_response.response.trim().to_string();

        // Check if the response contains a tool call
        if let Some(tool_call) = self.extract_tool_call(&response_text) {
            info!("🔧 Tool call detected: {}", tool_call.name);

            // Execute the tool
            match tools::execute_tool(&tool_call).await {
                Ok(result) => {
                    info!("✓ Tool executed: {}", result.output);

                    // Generate final response with tool result
                    let context = format!(
                        "Tool '{}' was executed.\nResult: {}\n\nProvide a natural response to the user based on this result.",
                        tool_call.name, result.output
                    );

                    Box::pin(self.generate_with_context(prompt, Some(context))).await
                }
                Err(e) => {
                    warn!("❌ Tool execution failed: {}", e);
                    Ok(format!("I tried to use a tool, but encountered an error: {}", e))
                }
            }
        } else {
            Ok(response_text)
        }
    }

    fn extract_tool_call(&self, text: &str) -> Option<ToolCall> {
        // Try to parse direct tool call format: {"name": "...", "arguments": {...}}
        if let Some(start) = text.find("{\"name\"") {
            // Find the end of the JSON object
            let mut brace_count = 0;
            let mut end_pos = start;
            for (i, ch) in text[start..].char_indices() {
                match ch {
                    '{' => brace_count += 1,
                    '}' => {
                        brace_count -= 1;
                        if brace_count == 0 {
                            end_pos = start + i + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if end_pos > start {
                let json_str = &text[start..end_pos];
                if let Ok(tool_call) = serde_json::from_str::<ToolCall>(json_str) {
                    return Some(tool_call);
                }
            }
        }

        // Try wrapped format: {"tool": {"name": "...", "arguments": {...}}}
        if let Some(start) = text.find("{\"tool\"") {
            if let Some(end) = text[start..].find("}}}") {
                let json_str = &text[start..start + end + 3];
                if let Ok(wrapper) = serde_json::from_str::<Value>(json_str) {
                    if let Some(tool_obj) = wrapper.get("tool") {
                        if let Ok(tool_call) = serde_json::from_value::<ToolCall>(tool_obj.clone()) {
                            return Some(tool_call);
                        }
                    }
                }
            }
        }

        None
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
