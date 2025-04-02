//! Text query builder agent for legal research.
//! 
//! This module implements an agent that converts natural language questions
//! into optimized OpenSearch queries. It uses LLM to understand the user's
//! intent and generate appropriate search terms for legal document retrieval.

use crate::connectors::llm::{LLMConnector, Message};
use std::collections::HashMap;

/// System prompt that instructs the LLM to act as an OpenSearch query expert
const SYSTEM_PROMPT: &str = "We have an Opensearch instant storing documents about code of conduct.
You are a data engineer who expertise Opensearch query.
Please suggest text query based on user's question return your answer only  and do not include prologue, prefix or suffix";

/// Agent responsible for building optimized OpenSearch queries from natural language questions.
/// 
/// This agent uses an LLM to analyze the user's question and generate appropriate
/// search terms that will yield relevant legal documents.
pub struct TextQueryBuilder {
    /// The LLM connector used for making requests to the language model
    llm_connector: LLMConnector,
}

impl TextQueryBuilder {
    /// Creates a new TextQueryBuilder instance.
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
        TextQueryBuilder { llm_connector }
    }

    /// Makes a request to the LLM with the given prompt.
    /// 
    /// # Arguments
    /// 
    /// * `user_prompt` - The prompt to send to the LLM
    /// 
    /// # Returns
    /// 
    /// An Option containing the LLM's response if successful, or None if the request fails
    pub async fn request_llm(&self, user_prompt: &str) -> Option<String> {
        self.llm_connector.request_llm(user_prompt).await.ok()
    }

    /// Constructs a prompt for query building.
    /// 
    /// # Arguments
    /// 
    /// * `question` - The user's question to convert into a search query
    /// 
    /// # Returns
    /// 
    /// A formatted prompt string ready to be sent to the LLM
    pub fn construct_prompt(&self, question: &str) -> String {
        format!(
            "# Instruction:

Based on the following question, what keywords should be queried in Opensearch

# Question:
{}",
            question
        )
    }

    /// Builds an OpenSearch query from a natural language question.
    /// 
    /// # Arguments
    /// 
    /// * `question` - The user's question to convert into a search query
    /// 
    /// # Returns
    /// 
    /// A string containing the generated OpenSearch query
    /// 
    /// # Panics
    /// 
    /// Panics if the LLM request fails or returns no response
    pub async fn build(&self, question: &str) -> String {
        let prompt = self.construct_prompt(question);
        let response = self.request_llm(&prompt).await.unwrap();
        response
    }
}

#[allow(dead_code)]
#[tokio::main]
async fn main() {
    let text_query_builder = TextQueryBuilder::new(
        "http://localhost:11434/api/chat".to_string(),
        "gemma2:latest".to_string(),
        "Not use".to_string(),
        None,
    );
    let query = text_query_builder
        .build("I want to invest in real estates. What detail should I know?")
        .await;
    println!("{}", query);
}
