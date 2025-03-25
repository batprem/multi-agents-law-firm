use lawfirm_agents::connectors::llm::{LLMConnector, Message};
use std::collections::HashMap;

const SYSTEM_PROMPT: &str = "We have an Opensearch instant storing documents about code of conduct.
You are a data engineer who expertise Opensearch query.
Please suggest text query based on user's question return your answer only  and do not include prologue, prefix or suffix";

pub struct TextQueryBuilder {
    llm_connector: LLMConnector,
}

impl TextQueryBuilder {
    pub fn new(url: String,
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

    pub async fn request_llm(&self, user_prompt: &str) -> Option<String> {
        self
            .llm_connector
            .request_llm(user_prompt)
            .await
            .ok()
    }
    pub fn construct_prompt(&self, question: &str) -> String {
        format!("# Instruction:

Based on the following question, what keywords should be queried in Opensearch

# Question:
{}", question)
    }
    pub async fn build(&self, question: &str) -> String {
        let prompt = self.construct_prompt(question);
        let response = self.request_llm(&prompt).await.unwrap();
        response
    }
}

#[tokio::main]
async fn main() {
    let text_query_builder = TextQueryBuilder::new(
        "http://localhost:11434/api/chat".to_string(),
        "gemma2:latest".to_string(),
        "Not use".to_string(),
        None,
    );
    let query = text_query_builder.build("I want to invest in real estates. What detail should I know?").await;
    println!("{}", query);
}