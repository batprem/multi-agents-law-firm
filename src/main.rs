//! Main entry point for the Law Firm application.
//! This module initializes the application and handles the main execution flow.

use lawfirm_agents::flow::flow;
use tokio;

/// Main function that serves as the entry point for the application.
/// It initializes the async runtime and executes the main flow with a sample legal question.
#[tokio::main]
async fn main() {
    let question = "What is the legal framework for investing in real estates?";
    flow(question).await;
}
