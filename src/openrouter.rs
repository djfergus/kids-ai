use anyhow::{Context, Result};
use eventsource_stream::Eventsource;
use futures::StreamExt;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

use crate::chat::Message;

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [Message],
    stream: bool,
}

pub struct OpenRouterClient {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenRouterClient {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }

    /// Stream a chat completion. Calls `on_token` for each content token received.
    /// Returns the full assembled response text.
    pub async fn stream_chat(
        &self,
        messages: &[Message],
        mut on_token: impl FnMut(&str),
    ) -> Result<String> {
        let body = ChatRequest {
            model: &self.model,
            messages,
            stream: true,
        };

        let response = self
            .client
            .post(OPENROUTER_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://github.com/kids-ai")
            .header("X-Title", "Kids AI")
            .json(&body)
            .send()
            .await
            .context("Failed to connect to OpenRouter")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenRouter API error {status}: {body}");
        }

        let mut stream = response.bytes_stream().eventsource();
        let mut full_response = String::new();

        while let Some(event) = stream.next().await {
            let event = match event {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("SSE error: {e}");
                    continue;
                }
            };

            if event.data == "[DONE]" {
                break;
            }

            let parsed: Value = match serde_json::from_str(&event.data) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if let Some(content) = parsed["choices"][0]["delta"]["content"].as_str() {
                if !content.is_empty() {
                    on_token(content);
                    full_response.push_str(content);
                }
            }
        }

        Ok(full_response)
    }
}
