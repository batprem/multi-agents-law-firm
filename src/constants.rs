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
