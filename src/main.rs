mod agents;
mod connectors;
mod constants;
mod tools;
use connectors::opensearch::create_client;
use constants::{LLM_HOST, MODEL};
use futures_util::StreamExt;
use futures_util::pin_mut;
use std::io::Write;
use tokio;
use tools::searcher::{OpenSearcher, Searcher};

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
    let selected_topic_index = topic_selector
        .select_topic("Tell me about real estate law?", &topics)
        .await;
    println!("Selected topic {}", topic_list[selected_topic_index - 1]);

    let selected_topic_index = topic_selector
        .select_topic("How to cook fried chicken?", &topics)
        .await;
    println!("{}", selected_topic_index);
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
