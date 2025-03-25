use lawfirm_agents::connectors::llm::{LLMConnector, Message};
use std::collections::HashMap;
use tokio;

const SYSTEM_PROMPT: &str = "Pick a choice, please answer only a number and do not include prologue, prefix or suffix";


pub struct TopicSelector {
    llm_connector: LLMConnector,
}

impl TopicSelector {
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
        TopicSelector { llm_connector }
    }

    pub async fn request_llm(&self, user_prompt: &str) -> Option<String> {
        self
            .llm_connector
            .request_llm(user_prompt)
            .await
            .ok()
    }
    pub fn construct_prompt(&self, user_prompt: &str, topics_prompt: &str) -> String {
        format!("# Instruction:
Pick an index of document that you think that it can help answer the following question or pick 0 if you think they are not helpful. Please answer only as a number and do not include prologue, prefix or suffix.

# Available source:
{}

# Question:
{}", topics_prompt, user_prompt)
    }
    pub async fn select_topic(&self, user_prompt: &str,  topics_prompt: &str) -> usize {
        let prompt = self.construct_prompt(user_prompt, &topics_prompt);
        let response = self.request_llm(&prompt).await.unwrap();
        response.trim().parse::<usize>().unwrap()
    }
}

#[tokio::main]
#[allow(dead_code)]
async fn main() {
    let topic_list: Vec<String> = vec![
        "Code of Conduct for Persons Providing Credit Rating Services".to_string(),
        "Code of Conduct for Share Registrars".to_string(),
        "Code on Immigration-Linked Investment Schemes".to_string(),
        "Code on Open-ended Fund Companies".to_string(),
        "Code on Pooled Retirement Funds".to_string(),
        "Code on Real Estate Investment Trusts".to_string(),
        "Corporate Finance Adviser Code of Conduct".to_string(),
        "Fund Manager Code of Conduct".to_string(),
        "SFC Code on MPF Products".to_string(),
        "Section I - Overarching Principles Section".to_string(),
        "Section II - Code on Unit Trusts and Mutual Funds".to_string(),
        "Section III - Code on Investment-Linked Assurance Schemes".to_string(),
        "Section IV - Code on Unlisted Structured Investment Products".to_string(),
        "The Codes on Takeovers and Mergers and Share Buy-backs".to_string(),
    ];

    let topics = "Topics:
1. Code of Conduct for Persons Providing Credit Rating Services
2. Code of Conduct for Share Registrars
3. Code on Immigration-Linked Investment Schemes
4. Code on Open-ended Fund Companies
5. Code on Pooled Retirement Funds
6. Code on Real Estate Investment Trusts
7. Corporate Finance Adviser Code of Conduct
8. Fund Manager Code of Conduct
9. SFC Code on MPF Products
10. Section I - Overarching Principles Section
11. Section II - Code on Unit Trusts and Mutual Funds
12. Section III - Code on Investment-Linked Assurance Schemes
13. Section IV - Code on Unlisted Structured Investment Products
14. The Codes on Takeovers and Mergers and Share Buy-backs".to_string();

    let topic_selector = TopicSelector::new(
        "http://localhost:11434/api/chat".to_string(),
        "gemma2:latest".to_string(),
        "Not use".to_string(),
        None,
    );
    let question = "I want to invest in real estates. What detail should I know?";
    let selected_topic_index = topic_selector.select_topic(question, &topics).await;
    println!("Question:
{}
Selected topic: {}", question, topic_list[selected_topic_index - 1]
);

    let selected_topic_index = topic_selector.select_topic("How to cook fried chicken?", &topics).await;
    println!("{}", selected_topic_index);
}
