use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::error::Error;
use itertools::Itertools;
use crate::database::db::get_user_games;

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,

    #[serde(rename = "generationConfig")]
    generation_config: GeminiGenerationConfig,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiCandidateContent,
}

#[derive(Deserialize)]
struct GeminiCandidateContent {
    parts: Vec<GeminiPartResponse>,
}

#[derive(Deserialize)]
struct GeminiPartResponse {
    text: String,
}

#[derive(Serialize)]
struct GeminiGenerationConfig {
    temperature: f32,
    top_p: f32,
    max_output_tokens: u32,
}

pub struct LLMClient {
    pub client: Client,
    pub api_key: String,
    pub api_url: String,
}

impl LLMClient {
    pub fn new(api_key: &str) -> Self {
        let model = "gemini-1.5-pro";
        LLMClient {
            client: Client::new(),
            api_key: api_key.to_string(),
            api_url: format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", model),
        }
    }

    pub async fn get_recommendation(
        &self,
        pool: &PgPool,
        steam_id: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let user_games = get_user_games(pool, steam_id).await?;
        if user_games.is_empty() {
            return Ok("No games found for this user.".to_string());
        }

        // Sort by playtime and take top 20
        let top_games: Vec<String> = user_games
            .iter()
            .filter(|g| g.playtime_forever >= 60)
            .sorted_by_key(|g| -(g.playtime_forever as i32))
            .take(20)
            .map(|g| format!("{} ({}h)", g.name, g.playtime_forever / 60))
            .collect();

        // Get *all* owned games for exclusion
        let owned_games: Vec<String> = user_games.iter().map(|g| g.name.clone()).collect();

        let prompt = format!(
            "The user has played the following games the most:\n{}\n\n\
                They also own these games and should not be recommended again:\n{}\n\n\
                Based on the top-played games, recommend three new games the user might enjoy. \
                Do not include any already owned games. Keep the total under 300 characters. \
                Vary the suggestions each time. Format: 1: Game Name - explanation.",
            top_games.join(", "),
            owned_games.join(", ")
        );

        let body = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart { text: prompt }],
            }],
            generation_config: GeminiGenerationConfig {
                temperature: 1.0,
                top_p: 0.9,
                max_output_tokens: 300,
            },
        };

        // Gemini's response
        let raw_response = self
            .client
            .post(&format!("{}?key={}", self.api_url, self.api_key))
            .json(&body)
            .send()
            .await?
            .text()
            .await?;

        // Used for debugging purposes
        println!("Raw response from Gemini: {}", raw_response);

        let response: GeminiResponse = serde_json::from_str(&raw_response)?;

        let generated = response
            .candidates
            .get(0)
            .and_then(|c| c.content.parts.get(0))
            .map(|p| p.text.clone())
            .unwrap_or_else(|| "⚠️ No recommendation.".to_string());

        Ok(generated)
    }
}
