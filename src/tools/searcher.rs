use serde_json::json;
use opensearch::{OpenSearch, SearchParts};
use std::collections::HashMap;


pub trait Searcher {
    async fn get_available_topics(&self) -> Option<String>;
}

pub struct OpenSearcher {
    client: OpenSearch,
}


impl OpenSearcher {
    pub fn new(client: OpenSearch) -> Self {
        OpenSearcher { client }
    }
}

impl Searcher for OpenSearcher {    
    async fn get_available_topics(&self) -> Option<String> {
        let response = self.client
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

        let response_body: serde_json::Value = response
            .json()
            .await
            .expect("Failed to parse response");

        if let Some(buckets) = response_body["aggregations"]["distinct_sources"]["buckets"].as_array() {
            let mut buckets_topic_to_url: HashMap<String, String> = HashMap::new();
            
            for bucket in buckets {
                if let (Some(topic_title), Some(file_url)) = (
                    bucket["key"]["topic_title"].as_str(),
                    bucket["key"]["file_url"].as_str(),
                ) {
                    buckets_topic_to_url.insert(topic_title.to_string(), file_url.to_string());
                }
            }
    
            let topic_list: Vec<String> = buckets_topic_to_url.keys().cloned().collect();
            let topic_choices = topic_list.iter()
                .enumerate()
                .map(|(i, topic)| format!("{}. {}", i + 1, topic))
                .collect::<Vec<String>>()
                .join("\n");
            
            return Some(topic_choices);
        }
        return None
    }
}


pub async fn test_query(client: &OpenSearch) -> serde_json::Value {
    let response = client
        .search(SearchParts::Index(&["sfc_code_preprocess"]))
        .body(json!({
            "query": {
                "match_all": {}
            }
        }))
        .send()
        .await
        .expect("Failed to execute search query");

    let response_body: serde_json::Value = response.json().await.expect("Failed to parse response");
    println!("Search Response: {}", response_body);
    response_body
}
