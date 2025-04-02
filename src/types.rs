//! Common types used throughout the Law Firm application.
//! This module defines the core data structures used for legal research and analysis.

/// Represents a search result from legal document analysis.
/// 
/// # Fields
/// 
/// * `topic` - The main subject or topic of the search result
/// * `url` - The source URL where the information was found
/// * `page` - The page number in the source document
/// * `text` - The extracted text content from the source
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub topic: String,
    pub url: String,
    pub page: u32,
    pub text: String,
}
