use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use tokio::task::JoinHandle;

const MAX_MESSAGE_LEN: usize = 4096;

#[derive(Clone)]
pub struct TelegramNotifier {
    client: Client,
    bot_token: String,
    chat_id: String,
}

impl TelegramNotifier {
    pub fn new(bot_token: String, chat_id: String) -> Self {
        Self {
            client: Client::new(),
            bot_token,
            chat_id,
        }
    }

    /// Send a Q&A notification to Telegram. Returns a JoinHandle for the background task.
    pub fn notify(&self, question: &str, answer: &str) -> JoinHandle<()> {
        let notifier = self.clone();
        let question = question.to_string();
        let answer = answer.to_string();

        tokio::spawn(async move {
            if let Err(e) = notifier.send_qa(&question, &answer).await {
                eprintln!("Telegram notification failed: {e}");
            }
        })
    }

    async fn send_qa(&self, question: &str, answer: &str) -> Result<()> {
        let text = format!(
            "<b>Question:</b>\n{}\n\n<b>Answer:</b>\n{}",
            escape_html(question),
            escape_html(answer)
        );

        for chunk in split_message(&text, MAX_MESSAGE_LEN) {
            self.send_message(&chunk).await?;
        }

        Ok(())
    }

    async fn send_message(&self, text: &str) -> Result<()> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.bot_token
        );

        let response = self
            .client
            .post(&url)
            .json(&json!({
                "chat_id": self.chat_id,
                "text": text,
                "parse_mode": "HTML",
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Telegram API error {status}: {body}");
        }

        Ok(())
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn split_message(text: &str, max_len: usize) -> Vec<String> {
    if text.len() <= max_len {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        if remaining.len() <= max_len {
            chunks.push(remaining.to_string());
            break;
        }

        // Find a safe UTF-8 char boundary at or before max_len.
        let boundary = if remaining.is_char_boundary(max_len) {
            max_len
        } else {
            (0..max_len)
                .rev()
                .find(|&i| remaining.is_char_boundary(i))
                .unwrap_or(0)
        };

        // Prefer splitting at a newline within the boundary.
        let split_at = remaining[..boundary].rfind('\n').unwrap_or(boundary);

        chunks.push(remaining[..split_at].to_string());
        remaining = remaining[split_at..].trim_start_matches('\n');
    }

    chunks
}
