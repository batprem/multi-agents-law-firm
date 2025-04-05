//! Search functionality for legal research.
//!
//! This module provides tools for searching legal documents using both text-based
//! and vector-based search methods. It supports topic-based filtering and
//! retrieves relevant document sections with their metadata.

use crate::connectors::text_embedder::get_embedding;
use crate::constants::{EMBEDDING_MODEL, SEARCH_TOP_RESULTS};
use crate::types::SearchResult;
use opensearch::{OpenSearch, SearchParts};
use serde_json::{Value, json};
use std::collections::HashMap;

/// Trait defining the interface for document search operations.
///
/// This trait provides methods for:
/// * Retrieving available legal topics
/// * Searching documents using different search methods
pub trait Searcher {
    /// Retrieves a list of available legal topics and their descriptions.
    ///
    /// # Returns
    ///
    /// An Option containing either:
    /// * A tuple of (topic list, formatted topic choices)
    /// * None if no topics are available
    fn get_available_topics(
        &self,
    ) -> impl std::future::Future<Output = Option<(Vec<String>, String)>> + Send;
    /// Searches for documents using the specified method and topic filter.
    ///
    /// This is an asynchronous function that returns a Future which resolves to the search results.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query string
    /// * `search_method` - The search method to use (text or vector)
    /// * `topic_title` - Optional topic to filter results
    ///
    /// # Returns
    ///
    /// A Future that resolves to a Result containing either:
    /// * `Ok(Vec<SearchResult>)` - A vector of search results on success
    /// * `Err(reqwest::Error)` - An HTTP client error if the search fails
    fn search_data_in_opensearch(
        &self,
        query: &str,
        search_method: SearchMethod,
        topic_title: Option<&str>,
    ) -> impl std::future::Future<Output = Result<Vec<SearchResult>, reqwest::Error>> + Send;
}

/// Implementation of the Searcher trait using OpenSearch.
///
/// This struct provides concrete implementations of the search operations
/// using an OpenSearch client.
pub struct OpenSearcher {
    /// The OpenSearch client used for making search requests
    client: OpenSearch,
}

impl OpenSearcher {
    /// Creates a new OpenSearcher instance.
    ///
    /// # Arguments
    ///
    /// * `client` - The OpenSearch client to use for searches
    pub fn new(client: OpenSearch) -> Self {
        OpenSearcher { client }
    }
}

/// Enum defining the available search methods.
///
/// # Variants
///
/// * `Text` - Traditional text-based search using keyword matching
/// * `Vector` - Semantic search using vector embeddings
pub enum SearchMethod {
    Text,
    Vector,
}

/// Extracts search results from the OpenSearch response.
///
/// # Arguments
///
/// * `json_result` - The JSON response from OpenSearch
///
/// # Returns
///
/// A vector of SearchResult structs containing the extracted information
fn extract_seaarch_results(json_result: Value) -> Vec<SearchResult> {
    let mut search_results: Vec<SearchResult> = vec![];

    if let Some(hits) = json_result["hits"]["hits"].as_array() {
        for hit in hits[0..SEARCH_TOP_RESULTS].iter() {
            if let (Some(topic), Some(url), Some(page), Some(text)) = (
                hit["fields"]["topic_title"][0].as_str(),
                hit["fields"]["file_url"][0].as_str(),
                hit["fields"]["page_number"][0].as_u64(),
                hit["fields"]["text"][0].as_str(),
            ) {
                search_results.push(SearchResult {
                    topic: topic.to_string(),
                    url: url.to_string(),
                    page: page as u32,
                    text: text.to_string(),
                });
            }
        }
    }
    search_results
}

impl Searcher for OpenSearcher {
    /// Retrieves available legal topics using OpenSearch aggregations.
    ///
    /// This implementation queries OpenSearch to get a list of unique topics
    /// and their associated URLs, then formats them for display.
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

    /// Performs a search in OpenSearch using the specified method.
    ///
    /// This implementation supports both text-based and vector-based search,
    /// with optional topic filtering. For vector search, it first converts
    /// the query into an embedding before searching.
    async fn search_data_in_opensearch(
        &self,
        query: &str,
        search_method: SearchMethod,
        topic_title: Option<&str>,
    ) -> Result<Vec<SearchResult>, reqwest::Error> {
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

        return Ok(extract_seaarch_results(query_result));
    }
}
