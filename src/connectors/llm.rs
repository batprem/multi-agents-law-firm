use reqwest::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

use async_stream::stream;

use futures_core::stream::Stream;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

pub struct LLMConnector {
    api_key: String,
    model: String,
    url: String,
    config: HashMap<String, serde_json::Value>,
    initial_message: Vec<Message>,
    client: Client,
}

impl LLMConnector {
    pub fn new(
        url: String,
        model: String,
        api_key: String,
        config: Option<HashMap<String, serde_json::Value>>,
        initial_message: Option<Vec<Message>>,
    ) -> Self {
        let default_config = HashMap::from([
            ("max_tokens".to_string(), json!(512)),
            ("temperature".to_string(), json!(0.6)),
            ("top_p".to_string(), json!(0.95)),
            ("repetition_penalty".to_string(), json!(1.05)),
        ]);

        let config = config.unwrap_or(default_config);
        let initial_message = initial_message.unwrap_or(Vec::new());

        LLMConnector {
            api_key,
            model,
            url,
            config,
            initial_message: initial_message,
            client: Client::new(),
        }
    }

    pub async fn request_llm(&self, prompt: &str) -> Result<String, reqwest::Error> {
        let mut headers = HeaderMap::new();

        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key)).unwrap(),
        );

        let mut messages = self.initial_message.clone();
        messages.push(Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        });

        let mut json_data = json!({
            "model": self.model,
            "messages": messages,
            "stream": false,
        });

        if let Some(map) = json_data.as_object_mut() {
            for (key, value) in &self.config {
                map.insert(key.clone(), value.clone());
            }
        }
        let response = self
            .client
            .post(&self.url)
            .headers(headers)
            .json(&json_data)
            .send()
            .await
            .ok()
            .unwrap();

        let result: serde_json::Value = response.json().await.ok().unwrap();
        let content = result["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        Ok(content)
    }

    pub async fn request_streaming_llm(
        &self,
        prompt: &str,
    ) -> Result<impl Stream<Item = String>, reqwest::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key)).unwrap(),
        );

        let mut messages = self.initial_message.clone();
        messages.push(Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        });

        let mut json_data = json!({
            "model": self.model,
            "messages": messages,
            "stream": true,
        });

        if let Some(map) = json_data.as_object_mut() {
            for (key, value) in &self.config {
                map.insert(key.clone(), value.clone());
            }
        }

        let response = self
            .client
            .post(&self.url)
            .headers(headers)
            .json(&json_data)
            .send()
            .await?;
        let mut stream = response.bytes_stream();
        Ok(stream! {
            while let Some(chunk) = tokio_stream::StreamExt::next(&mut stream).await {
                match chunk {
                    Ok(bytes) => {
                        let line = String::from_utf8(bytes.to_vec()).unwrap();
                        let chunk = serde_json::from_str::<serde_json::Value>(&line).unwrap();
                        yield chunk["message"]["content"].as_str().unwrap().to_string()
                    }
                    Err(e) => eprintln!("Stream error: {}", e),
                }
            }
        })
    }
}
