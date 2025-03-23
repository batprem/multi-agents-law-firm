use lawfirm_agents::constants::OpenSearchConfig;
use opensearch::{OpenSearch, http::transport::SingleNodeConnectionPool, auth::Credentials, http::transport::TransportBuilder};


pub fn create_client() -> OpenSearch {
    let config = OpenSearchConfig::from_env();
    let conn_pool = SingleNodeConnectionPool::new(config.url.parse().expect("Invalid URL"));
    let transport = TransportBuilder::new(conn_pool)
        .auth(Credentials::Basic(config.username.clone(), config.password.clone()))
        .build()
        .expect("Failed to create transport");
    OpenSearch::new(transport)
}