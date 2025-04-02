//! Main entry point for the Law Firm application.
//! This module initializes the application and handles the main execution flow.

use clap::Parser;
use env_logger;
use lawfirm_agents::flow::flow;
use tokio;

/// Command line arguments for the Law Firm application
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The legal question to be answered
    #[arg(short, long)]
    question: String,
}

/// Main function that serves as the entry point for the application.
/// It initializes the async runtime and executes the main flow with the provided legal question.
#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    flow(&args.question).await;
}
