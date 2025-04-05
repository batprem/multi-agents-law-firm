//! Rejection message generation agent for legal research.
//!
//! This module implements an agent that generates polite and professional rejection messages
//! when a user's question is either not legal-related or when we don't have sufficient
//! information to answer it.

use crate::connectors::llm::{LLMConnector, Message};
use std::collections::HashMap;

/// System prompt that instructs the LLM to generate polite rejection messages
const SYSTEM_PROMPT: &str = "You are a professional legal assistant who needs to politely reject questions that are either not legal-related or when we don't have sufficient information to answer them. Your responses should be:
1. Professional and courteous
2. Clear about why we cannot assist
3. Maintain a helpful tone while setting appropriate boundaries
4. Keep the message concise but informative

Return only the rejection message without any additional text or formatting.";

/// Agent responsible for generating rejection messages.
///
/// This agent uses an LLM to generate appropriate rejection messages when a user's
/// question cannot be answered, either because it's not legal-related or because
/// we don't have sufficient information.
pub struct RejectorAgent {
    /// The LLM connector used for making requests to the language model
    llm_connector: LLMConnector,
}

impl RejectorAgent {
    /// Creates a new RejectorAgent instance.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the LLM service
    /// * `model` - The name of the LLM model to use
    /// * `api_key` - The API key for authentication
    /// * `config` - Optional configuration parameters for the LLM connector
    pub fn new(
        url: String,
        model: String,
        api_key: String,
        config: Option<HashMap<String, serde_json::Value>>,
    ) -> Self {
        let mut initial_message: Vec<Message> = Vec::new();
        initial_message.push(Message {
            role: "system".to_string(),
            content: SYSTEM_PROMPT.to_string(),
        });
        let llm_connector = LLMConnector::new(url, model, api_key, config, Some(initial_message));
        RejectorAgent { llm_connector }
    }

    /// Constructs a prompt for rejection message generation.
    ///
    /// # Arguments
    ///
    /// * `question` - The user's question that needs to be rejected
    /// * `reason` - The reason for rejection ("not_legal" or "no_info")
    ///
    /// # Returns
    ///
    /// A formatted prompt string ready to be sent to the LLM
    fn construct_prompt(&self, question: &str, reason: &str) -> String {
        match reason {
            "not_legal" => format!(
                "Please generate a polite rejection message for the following non-legal question:\n\nQuestion: {}\n\nRejection message:",
                question
            ),
            "no_info" => format!(
                "Please generate a polite rejection message explaining that we don't have sufficient information to answer this legal question:\n\nQuestion: {}\n\nRejection message:",
                question
            ),
            _ => panic!("Invalid rejection reason: {}", reason),
        }
    }

    /// Generates a rejection message based on the reason for rejection.
    ///
    /// # Arguments
    ///
    /// * `question` - The user's question that needs to be rejected
    /// * `reason` - The reason for rejection ("not_legal" or "no_info")
    ///
    /// # Returns
    ///
    /// A Result containing either a stream of message chunks or an error
    pub async fn generate_rejection_message(
        &self,
        question: &str,
        reason: &str,
    ) -> Result<impl futures_util::Stream<Item = String>, Box<dyn std::error::Error>> {
        let prompt = self.construct_prompt(question, reason);
        let stream = self.llm_connector.request_streaming_llm(prompt).await?;
        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::{StreamExt, pin_mut};

    #[tokio::test]
    async fn test_generate_rejection_message() {
        let agent = RejectorAgent::new(
            "http://localhost:11434/api/chat".to_string(),
            "gemma2:latest".to_string(),
            "Not use".to_string(),
            None,
        );

        let question = "What's the weather like today?";
        let stream = agent
            .generate_rejection_message(question, "not_legal")
            .await
            .unwrap();
        pin_mut!(stream);
        let mut message = String::new();
        while let Some(chunk) = stream.next().await {
            message.push_str(&chunk);
        }
        assert!(!message.is_empty());
    }
}
