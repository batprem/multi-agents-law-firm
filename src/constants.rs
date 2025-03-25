use std::env;

pub struct OpenSearchConfig {
    pub username: String,
    pub password: String,
    pub url: String,
}

impl OpenSearchConfig {
    pub fn from_env() -> Self {
        OpenSearchConfig {
            username: env::var("OPENSEARCH_USERNAME").expect("OPENSEARCH_USERNAME not set"),
            password: env::var("OPENSEARCH_PASSWORD").expect("OPENSEARCH_PASSWORD not set"),
            url: env::var("OPENSEARCH_URL").expect("OPENSEARCH_URL not set"),
        }
    }
}

pub const MODEL: &str = "gemma2:latest";
pub const EMBEDDING_MODEL: &str = "all-minilm:latest";
pub const LLM_HOST: &str = "http://localhost:11434/api/chat";
pub const RETRY_COUNT: u16 = 5;
pub const SELECT_TOP_RESULTS: u16 = 3;
pub const INDEX_NAME: &str = "sfc_code_preprocess";
