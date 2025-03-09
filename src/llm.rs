use sqlx::PgPool;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::database::db::get_user_games;

#[derive(Serialize)]
struct HuggingFaceRequest {
    inputs: Vec<String>,
}

#[derive(Deserialize)]
struct HuggingFaceResponse {
    generated_text: String,
}

#[derive(Debug, Serialize)]
struct GameHistory {
    name: String,
    playtime_hours: u32,
}

pub struct LLMClient {
    client: Client,
    api_key: String,
    api_url: String,
}

impl LLMClient {
    pub fn new(api_key: &str) -> Self {
        let model = "mistralai/Mistral-7B-Instruct-v0.1";
        LLMClient {
            client: Client::new(),
            api_key: api_key.to_string(),
            api_url: format!("https://api-inference.huggingface.co/models/{}", model),
        }
    }

    pub async fn get_recommendation(&self, pool: &PgPool, steam_id: &str)
        -> Result<String, Box<dyn std::error::Error>>  {
        // Fetch the user's games from the database
        let user_games = get_user_games(pool, steam_id).await?;

        if user_games.is_empty() {
            return Ok("No games found for this user.".to_string());
        }

        // Format games to be fed into LLM
        let game_history: Vec<String> = user_games
            .iter()
            .map(|game| format!("- {} ({} hours played)", game.name, game.playtime_forever / 60))
            .collect();

        let prompt = format!(
            "The user has played these games:\n{}\n\nBased on this history, recommend three new games they might enjoy.",
            game_history.join("\n")
        );

        let request = HuggingFaceRequest {
            inputs: vec![prompt],
        };

        let response: HuggingFaceResponse = self.client.post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request).send().await?.json().await?;
        Ok(response.generated_text.clone())
    }
}