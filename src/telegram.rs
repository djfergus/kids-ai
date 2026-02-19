use reqwest::Client;
use serde_json::json;

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

    /// Send a Q&A notification to Telegram. Fire-and-forget — spawns a background task.
    pub fn notify(&self, question: &str, answer: &str) {
        let notifier = self.clone();
        let question = question.to_string();
        let answer = answer.to_string();

        tokio::spawn(async move {
            if let Err(e) = notifier.send_qa(&question, &answer).await {
                eprintln!("Telegram notification failed: {e}");
            }
        });
    }

    async fn send_qa(&self, question: &str, answer: &str) -> Result<(), reqwest::Error> {
        let text = format!(
            "<b>Question:</b>\n{}\n\n<b>Answer:</b>\n{}",
            escape_html(question),
            escape_html(answer)
        );

        // Split long messages
        let chunks = split_message(&text, MAX_MESSAGE_LEN);
        for chunk in chunks {
            self.send_message(&chunk).await?;
        }

        Ok(())
    }

    async fn send_message(&self, text: &str) -> Result<(), reqwest::Error> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.bot_token
        );

        self.client
            .post(&url)
            .json(&json!({
                "chat_id": self.chat_id,
                "text": text,
                "parse_mode": "HTML",
            }))
            .send()
            .await?;

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

        // Try to split at a newline within the limit
        let split_at = remaining[..max_len]
            .rfind('\n')
            .unwrap_or(max_len);

        chunks.push(remaining[..split_at].to_string());
        remaining = &remaining[split_at..].trim_start_matches('\n');
    }

    chunks
}
