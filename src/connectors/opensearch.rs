//! OpenSearch connector for legal research.
//!
//! This module provides functionality for connecting to and interacting with
//! OpenSearch instances. It handles authentication and connection setup using
//! environment-based configuration.

use crate::constants::OpenSearchConfig;
use opensearch::{
    OpenSearch, auth::Credentials, http::transport::SingleNodeConnectionPool,
    http::transport::TransportBuilder,
};

/// Creates a new OpenSearch client with authentication.
///
/// This function initializes a connection to an OpenSearch instance using
/// credentials and configuration from environment variables. It sets up
/// a single-node connection pool with basic authentication.
///
/// # Returns
///
/// An OpenSearch client instance ready for use
///
/// # Panics
///
/// Panics if:
/// * The OpenSearch URL from environment variables is invalid
/// * The transport cannot be created with the provided credentials
pub fn create_client() -> OpenSearch {
    let config = OpenSearchConfig::from_env();
    let conn_pool = SingleNodeConnectionPool::new(config.url.parse().expect("Invalid URL"));
    let transport = TransportBuilder::new(conn_pool)
        .auth(Credentials::Basic(
            config.username.clone(),
            config.password.clone(),
        ))
        .build()
        .expect("Failed to create transport");
    OpenSearch::new(transport)
}
