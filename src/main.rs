
mod connectors;
mod tools;
use connectors::opensearch::create_client;
use opensearch::{OpenSearch, SearchParts};
use tools::searcher::OpenSearcher;
use serde_json::json;
use tokio;
use crate::tools::searcher::Searcher;


#[tokio::main]
async fn main() {
    let opensearch_client = create_client();
    let searcher = OpenSearcher::new(opensearch_client);
    let topics = searcher.get_available_topics().await;
    println!("Topics:\n{}", topics.unwrap());
}
