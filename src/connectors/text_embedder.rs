use crate::constants::{EMBEDDING_MODEL, EMBEDDING_URL};
use reqwest::Client;
use serde_json::Value;
use std::error::Error;
use std::io::Error as IOError;
use tokio;


pub async fn get_embedding(text: &str, embedding_model: &str) -> Result<Vec<f64>, Box<dyn Error>> {
    let client = Client::new();
    let response = client
        .post(EMBEDDING_URL)
        .json(&serde_json::json!({
            "model": embedding_model,
            "input": text
        }))
        .send()
        .await
        .ok()
        .unwrap();
    response.error_for_status_ref()?;
    let response_json: Value = response.json().await.ok().unwrap();
    // println!("{:?}", response_json);
    if let Some(embedding) = response_json["embeddings"].as_array() {
        if let Some(inner_array) = embedding.first().and_then(|v| v.as_array()) {
            return Ok(inner_array.iter().map(|v| v.as_f64().unwrap()).collect());
        }
    }
    Err(IOError::new(std::io::ErrorKind::InvalidInput, "Failed to get embedding").into())
}

#[allow(dead_code)]
#[tokio::main]
async fn main() {
    let embedded_vector = get_embedding(
        "I want to invest in real estates. What detail should I know?",
        EMBEDDING_MODEL
    )
    .await
    .ok()
    .unwrap();
    println!("{:?}", embedded_vector);
}
