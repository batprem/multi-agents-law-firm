//! Application-wide constants and configuration.
//! This module contains all the constant values and configuration structures used throughout the application.

use std::env;

/// Configuration structure for OpenSearch connection.
///
/// # Fields
///
/// * `username` - The username for OpenSearch authentication
/// * `password` - The password for OpenSearch authentication
/// * `url` - The URL of the OpenSearch instance
pub struct OpenSearchConfig {
    pub username: String,
    pub password: String,
    pub url: String,
}

impl OpenSearchConfig {
    /// Creates a new OpenSearchConfig instance from environment variables.
    ///
    /// # Returns
    ///
    /// A new OpenSearchConfig instance with values from environment variables.
    ///
    /// # Panics
    ///
    /// Panics if any of the required environment variables are not set:
    /// - OPENSEARCH_USERNAME
    /// - OPENSEARCH_PASSWORD
    /// - OPENSEARCH_URL
    pub fn from_env() -> Self {
        OpenSearchConfig {
            username: env::var("OPENSEARCH_USERNAME").expect("OPENSEARCH_USERNAME not set"),
            password: env::var("OPENSEARCH_PASSWORD").expect("OPENSEARCH_PASSWORD not set"),
            url: env::var("OPENSEARCH_URL").expect("OPENSEARCH_URL not set"),
        }
    }
}

/// The LLM model to use for text generation
pub const MODEL: &str = "gemma2:latest";

/// The model to use for text embeddings
pub const EMBEDDING_MODEL: &str = "all-minilm:latest";

/// The URL endpoint for the embedding service
pub const EMBEDDING_URL: &str = "http://localhost:11434/api/embed";

/// The URL endpoint for the LLM chat service
pub const LLM_HOST: &str = "http://localhost:11434/api/chat";

/// Maximum number of retry attempts for failed operations
pub const RETRY_COUNT: u16 = 5;

/// Number of top results to select in search operations
pub const SELECT_TOP_RESULTS: u16 = 3;

/// Name of the OpenSearch index for code preprocessing
pub const INDEX_NAME: &str = "sfc_code_preprocess";

/// Number of top results to return in search operations
pub const SEARCH_TOP_RESULTS: usize = 3;
