use std::collections::VecDeque;

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

pub struct ChatHistory {
    system_prompt: String,
    messages: VecDeque<Message>,
    max_history: usize,
}

impl ChatHistory {
    pub fn new(system_prompt: String, max_history: usize) -> Self {
        Self {
            system_prompt,
            messages: VecDeque::new(),
            max_history,
        }
    }

    pub fn add_user_message(&mut self, content: &str) {
        self.messages.push_back(Message {
            role: "user".to_string(),
            content: content.to_string(),
        });
        self.trim();
    }

    pub fn add_assistant_message(&mut self, content: &str) {
        self.messages.push_back(Message {
            role: "assistant".to_string(),
            content: content.to_string(),
        });
        self.trim();
    }

    /// Build the full message list for the API: system prompt + conversation history.
    pub fn build_api_messages(&self) -> Vec<Message> {
        let mut msgs = Vec::with_capacity(self.messages.len() + 1);
        msgs.push(Message {
            role: "system".to_string(),
            content: self.system_prompt.clone(),
        });
        msgs.extend(self.messages.iter().cloned());
        msgs
    }

    fn trim(&mut self) {
        while self.messages.len() > self.max_history {
            self.messages.pop_front();
        }
        // Ensure history never starts with an assistant message (no orphaned response).
        if self.messages.front().map(|m| m.role == "assistant").unwrap_or(false) {
            self.messages.pop_front();
        }
    }
}
