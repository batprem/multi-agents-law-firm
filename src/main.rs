pub mod agents;
pub mod connectors;
pub mod constants;
pub mod tools;
pub mod types;
use connectors::opensearch::create_client;
use constants::{LLM_HOST, MODEL};
use futures_util::{StreamExt, FutureExt};
use futures_util::pin_mut;
use std::io::Write;
use serde_json::Value;
use tokio;
use tools::searcher::{OpenSearcher, Searcher, SearchMethod};



#[tokio::main]
async fn main() {
    let opensearch_client = create_client();
    let searcher = OpenSearcher::new(opensearch_client);
    let (topic_list, topics) = searcher.get_available_topics().await.unwrap();
    println!("Topics:\n{}", topics);
    

    let llm = connectors::llm::LLMConnector::new(
        LLM_HOST.to_string(),
        MODEL.to_string(),
        "Not use".to_string(),
        None,
        None,
    );

    let topic_selector = agents::topic_selector::TopicSelector::new(
        LLM_HOST.to_string(),
        MODEL.to_string(),
        "Not use".to_string(),
        None,
    );
    let text_query_builder = agents::text_query_builder::TextQueryBuilder::new(
        LLM_HOST.to_string(),
        MODEL.to_string(),
        "Not use".to_string(),
        None,
    );
    let question = "What is the legal framework for investing in real estates?";
    // Select the topic
    let selected_topic_index = topic_selector
        .select_topic(question, &topics)
        .await.unwrap();
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
    let search_text_result_raw = search_text_result.unwrap();
    let search_vector_result_raw = search_vector_result.unwrap();
    println!("{:?}", search_text_result_raw);
    // Summarize
    let summarizer = agents::source_summarizer::SourceSummarizer::new(
        LLM_HOST.to_string(),
        MODEL.to_string(),
        "Not use".to_string(),
        None,
    );
    let summary = summarizer.summarize_into_context_bulk(&search_text_result_raw[0..3], question).await.unwrap();

    // Response to user
    let user_interactive_agent = agents::user_interactive::UserInteractive::new(
        LLM_HOST.to_string(),
        MODEL.to_string(),
        "Not use".to_string(),
        None,
    );
    // Summarize

    // let selected_topic_index = topic_selector
    //     .select_topic("How to cook fried chicken?", &topics)
    //     .await;
    // println!("{}", selected_topic_index);
    // let stream = llm
    //     .request_streaming_llm("Write essay about the history of machine learning?")
    //     .await
    //     .ok()
    //     .unwrap();
    // pin_mut!(stream);

    // while let Some(chunk) = stream.next().await {
    //     print!("{}", chunk);
    //     std::io::stdout().flush().unwrap();
    // }
}
