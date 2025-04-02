//! User interaction agent for legal research.
//! 
//! This module implements an agent that generates human-readable responses to legal
//! questions using summarized document content. It provides streaming responses with
//! proper citations and encourages users to explore the source materials.

use crate::connectors::llm::{LLMConnector, Message};
use std::collections::HashMap;
use futures_core::stream::Stream;

/// System prompt that instructs the LLM to act as a legal expert providing
/// well-referenced answers based on prepared document summaries
const SYSTEM_PROMPT: &str = "You are an humble expert in lawfirm, and your secretary already
prepared gists from the related document for you to answer user's question 
Your duty is to answer the question with confidence using the prepared data source as a reference.
Please also add the reference of data source with URL to PDF file with page number and encourage user to find out more information with it";

/// Agent responsible for generating human-readable responses to legal questions.
/// 
/// This agent uses an LLM to synthesize information from summarized legal documents
/// into coherent, well-referenced answers that are streamed to the user.
pub struct UserInteractive {
    /// The LLM connector used for making requests to the language model
    llm_connector: LLMConnector,
}

impl UserInteractive {
    /// Creates a new UserInteractive instance.
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
        UserInteractive { llm_connector }
    }

    /// Constructs a prompt for generating a response.
    /// 
    /// # Arguments
    /// 
    /// * `question` - The user's question to answer
    /// * `topic` - The legal topic being discussed
    /// * `contexts` - Vector of summarized document content
    /// * `source_url` - URL to the source document
    /// 
    /// # Returns
    /// 
    /// A formatted prompt string ready to be sent to the LLM
    fn construct_prompt(&self, question: &str, topic: &str, contexts: Vec<String>, source_url: &str) -> String {
        let context_prompt = contexts.join("\n- ");
        format!(
            "
# question:
{}

# Prepared data source:
Document: {}
{}
URL: {}
",
            question, topic, context_prompt, source_url
        )
    }

    /// Generates a streaming response to the user's question.
    /// 
    /// # Arguments
    /// 
    /// * `question` - The user's question to answer
    /// * `topic` - The legal topic being discussed
    /// * `contexts` - Vector of summarized document content
    /// * `source_url` - URL to the source document
    /// 
    /// # Returns
    /// 
    /// A Result containing either a Stream of response chunks or a reqwest error
    pub async fn answer(&self, question: &str, topic: &str, contexts: Vec<String>, source_url: &str) -> Result<impl Stream<Item = String>, reqwest::Error> {
        self.llm_connector.request_streaming_llm(self.construct_prompt(question, topic, contexts, source_url)).await
    }
}

#[allow(dead_code)]
#[tokio::main]
async fn main() {
    use futures_util::StreamExt;
    use futures_util::pin_mut;  
    use std::io::Write;


    let user_interactive = UserInteractive::new(
        "http://localhost:11434/api/chat".to_string(),
        "gemma2:latest".to_string(),
        "Not use".to_string(),
        None,
    );

    let question = "I want to invest in real estates. What detail should I know?";
    let topic = "Code on Real Estate Investment Trusts";
    let contexts = vec![
        "You should know the transaction history of properties, renovation plans with costs and financing, operating data like occupancy rate, tenant mix, lease details, borrowing policy, risk mitigation measures, dividend policy, insurance arrangements, and exit strategy in case of divestment.".to_string(),
        "When making property investments, you should consider factors like demographics, economic risks, political risks, legal risks and tax considerations, policies affecting property investments, the overall property market overview, competitive dynamics in the rental market, operational requirements, and rules governing property ownership and tenancy matters.".to_string(),
        "at least 75% of the gross asset value of a scheme shall be invested in real estate that generates recurrent rental income at all times.".to_string(),
        "The offering document of the scheme shall clearly include a discussion of the business plan for property investment and management covering the scope and type of investments made or intended to be made by the scheme, including the type(s) of real estate (e.g. residential/commercial/industrial).".to_string()
    ];
    let stream = user_interactive.answer(question, topic, contexts, "https://www.investinginrealestates.com").await.unwrap();
    pin_mut!(stream);
    while let Some(chunk) = stream.next().await {
        print!("{}", chunk);
        std::io::stdout().flush().unwrap();
    };
}
