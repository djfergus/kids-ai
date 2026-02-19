use anyhow::{Context, Result};

const DEFAULT_MODEL: &str = "meta-llama/llama-3.3-70b-instruct:free";
const DEFAULT_MAX_HISTORY: usize = 20;

pub struct Config {
    pub openrouter_api_key: String,
    pub openrouter_model: String,
    pub telegram_bot_token: String,
    pub telegram_chat_id: String,
    pub child_name: Option<String>,
    pub max_history: usize,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();

        let openrouter_api_key = std::env::var("OPENROUTER_API_KEY")
            .context("OPENROUTER_API_KEY is required. Set it in .env or environment.")?;

        let telegram_bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
            .context("TELEGRAM_BOT_TOKEN is required. Set it in .env or environment.")?;

        let telegram_chat_id = std::env::var("TELEGRAM_CHAT_ID")
            .context("TELEGRAM_CHAT_ID is required. Set it in .env or environment.")?;

        let openrouter_model = std::env::var("OPENROUTER_MODEL")
            .unwrap_or_else(|_| DEFAULT_MODEL.to_string());

        let child_name = std::env::var("CHILD_NAME").ok().filter(|s| !s.is_empty());

        let max_history = std::env::var("MAX_HISTORY")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_MAX_HISTORY);

        Ok(Config {
            openrouter_api_key,
            openrouter_model,
            telegram_bot_token,
            telegram_chat_id,
            child_name,
            max_history,
        })
    }
}
