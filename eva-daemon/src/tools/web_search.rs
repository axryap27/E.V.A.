use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use tracing::{info, error};

use super::ToolResult;

#[derive(Deserialize)]
struct SearchArgs {
    query: String,
}

#[derive(Deserialize)]
struct DuckDuckGoResult {
    #[serde(rename = "AbstractText")]
    abstract_text: String,
    #[serde(rename = "AbstractURL")]
    abstract_url: String,
    #[serde(rename = "RelatedTopics")]
    related_topics: Vec<Value>,
}

pub async fn execute(arguments: &Value) -> Result<ToolResult> {
    let args: SearchArgs = serde_json::from_value(arguments.clone())
        .context("Invalid search arguments")?;

    info!("🔍 Searching web for: {}", args.query);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    // Use DuckDuckGo Instant Answer API (no key required)
    let url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
        urlencoding::encode(&args.query)
    );

    match client.get(&url).send().await {
        Ok(response) => {
            match response.json::<DuckDuckGoResult>().await {
                Ok(result) => {
                    let mut output = String::new();

                    if !result.abstract_text.is_empty() {
                        output.push_str(&format!("{}\n", result.abstract_text));
                        if !result.abstract_url.is_empty() {
                            output.push_str(&format!("Source: {}\n", result.abstract_url));
                        }
                    }

                    // Add related topics if available
                    if output.is_empty() && !result.related_topics.is_empty() {
                        output.push_str("Related information:\n");
                        for (i, topic) in result.related_topics.iter().take(3).enumerate() {
                            if let Some(text) = topic.get("Text").and_then(|v| v.as_str()) {
                                output.push_str(&format!("{}. {}\n", i + 1, text));
                            }
                        }
                    }

                    if output.is_empty() {
                        // Fallback: scrape search results page
                        output = scrape_search_results(&args.query).await
                            .unwrap_or_else(|_| "No results found.".to_string());
                    }

                    info!("✓ Search complete");
                    Ok(ToolResult {
                        success: true,
                        output: output.trim().to_string(),
                    })
                }
                Err(e) => {
                    error!("Failed to parse DuckDuckGo response: {}", e);
                    // Fallback to scraping
                    let output = scrape_search_results(&args.query).await?;
                    Ok(ToolResult {
                        success: true,
                        output,
                    })
                }
            }
        }
        Err(e) => {
            error!("Search request failed: {}", e);
            Ok(ToolResult {
                success: false,
                output: format!("Search failed: {}", e),
            })
        }
    }
}

async fn scrape_search_results(query: &str) -> Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .build()?;

    let url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query));
    let html = client.get(&url).send().await?.text().await?;

    // Simple HTML parsing - extract text snippets
    let snippets: Vec<&str> = html
        .split("<a class=\"result__snippet\"")
        .skip(1)
        .take(3)
        .filter_map(|chunk| {
            chunk
                .split("</a>")
                .next()
                .and_then(|s| s.split('>').nth(1))
        })
        .collect();

    if snippets.is_empty() {
        Ok("No results found.".to_string())
    } else {
        Ok(snippets.join("\n\n"))
    }
}
