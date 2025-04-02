//! Flow control and orchestration module for the Law Firm application.
//!
//! This module implements the main workflow of the legal research system, coordinating
//! between different components including:
//! - Topic selection
//! - Query building
//! - Search execution (both text and vector-based)
//! - Result summarization
//! - User interaction and response generation

use crate::connectors::opensearch::create_client;
use crate::constants::{LLM_HOST, MODEL};
use crate::tools::searcher::{OpenSearcher, SearchMethod, Searcher};
use futures_util::StreamExt;
use futures_util::pin_mut;
use std::io::Write;
use tokio;

/// Main workflow function that orchestrates the legal research process.
///
/// This function implements the complete workflow for processing a legal question:
/// 1. Initializes necessary components (OpenSearch client, LLM connector)
/// 2. Selects relevant legal topic
/// 3. Builds search query
/// 4. Performs both text and vector-based searches
/// 5. Summarizes search results
/// 6. Generates and streams response to the user
///
/// # Arguments
///
/// * `question` - The legal question to research
///
/// # Example
///
/// ```
/// let question = "What is the legal framework for investing in real estates?";
/// flow(question).await;
/// ```
pub async fn flow(question: &str) {
    let opensearch_client = create_client();
    let searcher = OpenSearcher::new(opensearch_client);
    let (topic_list, topics) = searcher.get_available_topics().await.unwrap();
    println!("Topics:\n{}", topics);

    let topic_selector = crate::agents::topic_selector::TopicSelector::new(
        LLM_HOST.to_string(),
        MODEL.to_string(),
        "Not use".to_string(),
        None,
    );
    let text_query_builder = crate::agents::text_query_builder::TextQueryBuilder::new(
        LLM_HOST.to_string(),
        MODEL.to_string(),
        "Not use".to_string(),
        None,
    );
    // let question = "What is the legal framework for investing in real estates?";
    // Select the topic
    let selected_topic_index = topic_selector
        .select_topic(question, &topics)
        .await
        .unwrap();
    let topic_title = topic_list[selected_topic_index - 1].clone();
    println!("Selected topic {}", topic_list[selected_topic_index - 1]);

    // Build query
    let query_text = text_query_builder.build(question).await;
    println!("Query text: {}", query_text);
    println!("{query_text}");

    // Query
    let (search_text_result, search_vector_result) = tokio::join!(
        searcher.search_data_in_opensearch(&query_text, SearchMethod::Text, Some(&topic_title)),
        searcher.search_data_in_opensearch(&query_text, SearchMethod::Vector, Some(&topic_title))
    );
    let search_results = [
        &(search_text_result.unwrap())[..],
        &(search_vector_result.unwrap())[..],
    ]
    .concat();

    // Summarize
    let summarizer = crate::agents::source_summarizer::SourceSummarizer::new(
        LLM_HOST.to_string(),
        MODEL.to_string(),
        "Not use".to_string(),
        None,
    );
    let summary = summarizer
        .summarize_into_context_bulk(&search_results, question)
        .await;
    let summary_texts: Vec<String> = summary
        .into_iter()
        .filter(|s| s.is_useful)
        .map(|s| format!("{} - Page: {}", s.summarize, s.page))
        .collect();

    // Response to user
    let user_interactive_agent = crate::agents::user_interactive::UserInteractive::new(
        LLM_HOST.to_string(),
        MODEL.to_string(),
        "Not use".to_string(),
        None,
    );
    let stream = user_interactive_agent
        .answer(
            question,
            &topic_title,
            summary_texts,
            &search_results[0].url,
        )
        .await
        .unwrap();
    pin_mut!(stream);
    while let Some(chunk) = stream.next().await {
        print!("{}", chunk);
        std::io::stdout().flush().unwrap();
    }
}
