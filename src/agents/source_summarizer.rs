//! Source summarization agent for legal research.
//!
//! This module implements an agent that analyzes and summarizes legal documents
//! to determine their relevance to a user's question. It uses LLM to evaluate
//! the usefulness of each source and generate concise summaries of relevant content.

use crate::connectors::llm::{LLMConnector, Message};
use crate::types::SearchResult;
use futures::future::join_all;
use log;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use tokio;

/// System prompt that instructs the LLM to evaluate and summarize legal documents
const SYSTEM_PROMPT: &str = "You are an expert in lawfirm who are assigned to consider whether a text data source is useful to answer a user question or not. If yes, you will summarize the text which corespond user's question for another expert to write answer the user , otherwise, do nothing. You answer must be in JSON format with field:
\"is_useful\": boolean determining whether the source is useful,
\"summarize\": string your summarization refering the part for the text or empty string if not useful

return your answer only and do not include prologue, prefix or suffix";

/// Agent responsible for analyzing and summarizing legal documents.
///
/// This agent uses an LLM to evaluate the relevance of legal documents to a user's
/// question and generate concise summaries of the relevant content.
pub struct SourceSummarizer {
    /// The LLM connector used for making requests to the language model
    llm_connector: LLMConnector,
}

/// Represents a summary of a legal document with its relevance and content.
///
/// # Fields
///
/// * `is_useful` - Whether the document contains relevant information
/// * `summarize` - A concise summary of the relevant content
/// * `page` - The page number in the source document
#[derive(Debug, Deserialize)]
pub struct Summary {
    pub is_useful: bool,
    pub summarize: String,
    pub page: u32,
}

impl SourceSummarizer {
    /// Creates a new SourceSummarizer instance.
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
        SourceSummarizer { llm_connector }
    }

    /// Constructs a prompt for document summarization.
    ///
    /// # Arguments
    ///
    /// * `text_source` - The text content to analyze
    /// * `question` - The user's question to evaluate relevance against
    ///
    /// # Returns
    ///
    /// A formatted prompt string ready to be sent to the LLM
    pub fn construct_prompt(&self, text_source: &str, question: &str) -> String {
        format!(
            "# Source:
{text_source}

# question:

{question}"
        )
    }

    /// Summarizes a single text source and evaluates its relevance.
    ///
    /// # Arguments
    ///
    /// * `text_source` - The text content to analyze
    /// * `question` - The user's question to evaluate relevance against
    /// * `page` - The page number in the source document
    ///
    /// # Returns
    ///
    /// A Summary struct containing the evaluation and summary
    ///
    /// # Panics
    ///
    /// Panics if the LLM response cannot be parsed as valid JSON
    pub async fn summarize(&self, text_source: &str, question: &str, page: u32) -> Summary {
        let prompt = self.construct_prompt(text_source, question);
        let response = self.llm_connector.request_llm(&prompt).await.unwrap();
        let llm_response = response
            .trim_matches(|c| "`json".chars().collect::<Vec<char>>().contains(&c))
            .to_string();
        let json_response: Value =
            serde_json::from_str(&llm_response).expect("Failed to parse response");
        Summary {
            is_useful: json_response["is_useful"].as_bool().unwrap(),
            summarize: json_response["summarize"].as_str().unwrap().to_string(),
            page: page,
        }
    }

    /// Summarizes a search result and evaluates its relevance.
    ///
    /// # Arguments
    ///
    /// * `search_result` - The search result to analyze
    /// * `question` - The user's question to evaluate relevance against
    ///
    /// # Returns
    ///
    /// A Summary struct containing the evaluation and summary
    pub async fn summarize_into_context(
        &self,
        search_result: &SearchResult,
        question: &str,
    ) -> Summary {
        self.summarize(&search_result.text, question, search_result.page)
            .await
    }

    /// Summarizes multiple search results in parallel.
    ///
    /// # Arguments
    ///
    /// * `search_results` - A slice of search results to analyze
    /// * `question` - The user's question to evaluate relevance against
    ///
    /// # Returns
    ///
    /// A vector of Summary structs containing evaluations and summaries
    pub async fn summarize_into_context_bulk(
        &self,
        search_results: &[SearchResult],
        question: &str,
    ) -> Vec<Summary> {
        let futures = search_results
            .iter()
            .map(|result| self.summarize_into_context(result, question));
        join_all(futures).await
    }
}

#[allow(dead_code)]
#[tokio::main]
async fn main() {
    let summarizer = SourceSummarizer::new(
        "http://localhost:11434/api/chat".to_string(),
        "gemma2:latest".to_string(),
        "Not use".to_string(),
        None,
    );
    let sample_text_1 = "In particular, connected party transactions in the \nnature of services provided relating to the real estate of the scheme in \nthe ordinary and usual course of estate management, such as \nrenovation and maintenance work, shall be contracted on normal \ncommercial terms subject to the prior approval of the trustee. \n \n    (b)  valued, in relation to a property transaction, by an independent valuer that meets \nthe requirements of Chapter 6; \n(c)  consistent with the investment objectives and strategy of the scheme; \n \n(d)  on terms that are fair and reasonable and in the best interests of holders; and \n \n(e)  properly disclosed to holders. \n \n8.7A Save as otherwise provided in this Code or the guidelines issued by the Commission \nfrom time to time, all connected party transactions will be regulated with reference to \nrequirements applicable to listed companies under Chapter 14A of the Listing Rules \n(modified as appropriate pursuant to 2.26) to the extent appropriate and practicable, \nincluding but not limited to".to_string();
    let search_result_1 = SearchResult {
        topic: "Investing in real estates".to_string(),
        url: "https://www.investinginrealestates.com".to_string(),
        page: 1,
        text: sample_text_1,
    };
    let sample_text_2 = r#"" 3.10 "pooled retirement fund" or "PRF" has the same meaning as "pooling agreement" in the Occupational Retirement Schemes Ordinance (Chapter 426 of Laws of Hong Kong)."#.to_string();
    let search_result_2 = SearchResult {
        topic: "Investing in real estates".to_string(),
        url: "https://www.investinginrealestates.com".to_string(),
        page: 1,
        text: sample_text_2,
    };
    let question = "Tell me about investing in real estates?";
    let summary: Vec<_> = summarizer
        .summarize_into_context_bulk(&[search_result_1, search_result_2], question)
        .await
        .into_iter()
        .filter(|result| result.is_useful)
        .collect();
    log::debug!("{:?}", summary);
}
