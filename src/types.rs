#[derive(Debug, Clone)]
pub struct SearchResult {
    pub topic: String,
    pub url: String,
    pub page: u32,
    pub text: String,
}
