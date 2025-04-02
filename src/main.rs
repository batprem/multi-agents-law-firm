pub mod flow;
use flow::flow;
use tokio;

#[tokio::main]
async fn main() {
    let question = "What is the legal framework for investing in real estates?";
    flow(question).await;
}