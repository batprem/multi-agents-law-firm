use lawfirm_agents::connectors::text_embedder::get_embedding;
use lawfirm_agents::constants::EMBEDDING_MODEL;
use opensearch::{OpenSearch, SearchParts};
use serde_json::{Value, json};
use std::collections::HashMap;
use tokio;

pub trait Searcher {
    fn get_available_topics(&self) -> impl std::future::Future<Output = Option<(Vec<String>, String)>> + Send;
    fn search_data_in_opensearch(
        &self,
        query: &str,
        search_method: SearchMethod,
        topic_title: Option<&str>,
    ) -> impl std::future::Future<Output = Result<Value, reqwest::Error>> + Send;
}

pub struct OpenSearcher {
    client: OpenSearch,
}

impl OpenSearcher {
    pub fn new(client: OpenSearch) -> Self {
        OpenSearcher { client }
    }
}

pub enum SearchMethod {
    Text,
    Vector,
}

impl Searcher for OpenSearcher {
    async fn get_available_topics(&self) -> Option<(Vec<String>, String)> {
        let response = self
            .client
            .search(SearchParts::Index(&["sfc_code_preprocess"]))
            .body(json!({
                "size": 0,
                "aggs": {
                    "distinct_sources": {
                        "composite": {
                            "sources": [
                                {"topic_title": {"terms": {"field": "topic_title.keyword"}}},
                                {"file_url": {"terms": {"field": "file_url.keyword"}}}
                            ],
                            "size": 10000
                        }
                    }
                }
            }))
            .send()
            .await
            .expect("Failed to execute search query");

        let response_body: serde_json::Value =
            response.json().await.expect("Failed to parse response");

        if let Some(buckets) =
            response_body["aggregations"]["distinct_sources"]["buckets"].as_array()
        {
            let mut buckets_topic_to_url: HashMap<String, String> = HashMap::new();

            for bucket in buckets {
                if let (Some(topic_title), Some(file_url)) = (
                    bucket["key"]["topic_title"].as_str(),
                    bucket["key"]["file_url"].as_str(),
                ) {
                    buckets_topic_to_url.insert(topic_title.to_string(), file_url.to_string());
                }
            }

            let mut topic_list: Vec<String> = buckets_topic_to_url.keys().cloned().collect();
            topic_list.sort();
            let topic_choices = topic_list
                .iter()
                .enumerate()
                .map(|(i, topic)| format!("{}. {}", i + 1, topic))
                .collect::<Vec<String>>()
                .join("\n");

            return Some((topic_list, topic_choices));
        }
        return None;
    }
    async fn search_data_in_opensearch(
        &self,
        query: &str,
        search_method: SearchMethod,
        topic_title: Option<&str>,
    ) -> Result<Value, reqwest::Error> {
        let mut must: Vec<Value> = vec![];
        match search_method {
            SearchMethod::Text => {
                let _ = must.push(json!({
                        "match": {
                            "text": {
                                "query": query,
                            },
                        },
                    }
                ));
            }
            SearchMethod::Vector => {
                let query_embedding = get_embedding(query, EMBEDDING_MODEL).await.ok().unwrap();
                must.push(json!({"knn": {"embedding": {"vector": query_embedding, "k": 5}}}))
            }
        }
        must.push(json!({
            "match": {
                "topic_title": {
                    "query": topic_title,
                },
            }
        }));
        let query_body = json!({
            "query": {"bool": {"must": must}},
            "_source": false,
            "fields": ["id", "topic_title", "text", "file_url", "page_number"],
        });
        let response = self
            .client
            .search(SearchParts::Index(&["sfc_code_preprocess"]))
            .body(query_body)
            .send()
            .await
            .ok()
            .unwrap();
        let query_result: Value = response.json().await.expect("Failed to parse response");

        return Ok(query_result);
    }
}

#[allow(dead_code)]
#[tokio::main]
async fn main() {
    use lawfirm_agents::connectors::opensearch::create_client;

    let opensearch_client = create_client();
    let searcher = OpenSearcher::new(opensearch_client);
    let query_result = searcher
        .search_data_in_opensearch(
            "What is the law on real estate?",
            SearchMethod::Vector,
            Some("Real Estate"),
        )
        .await;
    println!("{:?}", query_result);

    let query_result = searcher
        .search_data_in_opensearch(
            "What is the law on real estate?",
            SearchMethod::Text,
            Some("Real Estate"),
        )
        .await;
    println!("{:?}", query_result);
}
