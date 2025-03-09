use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct HuggingFaceRequest {
    inputs: String,
}

#[derive(Deserialize)]
struct HuggingFaceResponse {
    generated_text: String,
}

pub struct LLMClient {
    client: Client,
    api_key: String,
    api_url: String,
}

impl LLMClient {
    pub fn new(api_key: &str, model: &str) -> Self {
        LLMClient {
            client: Client::new(),
            api_key: api_key.to_string(),
            api_url: format!("https://api-inference.huggingface.co/models/{}", model),
        }
    }

    pub async fn get_recommendation(&self, game: &str) -> Result<String, reqwest::Error> {
        let request = HuggingFaceRequest {
            inputs: format!("Recommend me three games that have a similar playstyle and genre as {}", game),
        };

        let response = self.client.post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request).send().await?
            .json::<Vec<HuggingFaceResponse>>().await?;

        Ok(response[0].generated_text.clone())
    }
}