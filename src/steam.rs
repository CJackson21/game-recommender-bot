use std::fmt::format;
use tokio::time::{sleep, Duration};
use reqwest::{Client};
use serde::Deserialize;

const RETRY_COOLDOWN: u64 = 5;

#[derive(Deserialize, Debug, Clone)]
pub struct SteamGame {
    pub appid: u32,
    pub name: String,
    pub playtime_forever: u32,
}

#[derive(Deserialize)]
struct SteamResponse {
    response: SteamOwnedGames,
}

#[derive(Deserialize)]
pub struct SteamOwnedGames {
    pub games: Vec<SteamGame>,
}

#[derive(Deserialize, Debug)]
pub struct SteamProfile {
    pub steam_id: String,
    pub persona_name: String,
    pub avatar: String,
}

#[derive(Deserialize)]
struct SteamProfileData {
    players: Vec<SteamProfile>,
}

#[derive(Deserialize)]
struct SteamProfileResponse {
    response: SteamProfileData,
}

pub async fn fetch_steam_games(steam_id: &str, api_key: &str) -> anyhow::Result<Vec<SteamGame>> {
    let mut attempts = 0;
    let max_retries = 5;

    while attempts < max_retries {
        let url = format!(
            "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?key={}&steamid={}&format=json&include_appinfo=true",
            api_key, steam_id
        );

        let client = Client::new();
        let response = client.get(url).send().await?;

        if response.status().is_success() {
            let steam_data = response.json::<SteamResponse>().await?.response;
            return Ok(steam_data.games);
        }
        else if response.status().as_u16() == 429 {
            eprintln!("Steam has limited the rate limit. Retrying in 5 seconds...");
            sleep(Duration::from_secs(RETRY_COOLDOWN)).await;
        }
        else {
            eprintln!("Failed to fetch Steam games, Status: {}. Retrying...",
                      response.status().as_u16());
            sleep(Duration::from_secs(2_u64.pow(attempts))).await;
        }
        attempts += 1;
    }
    Err(anyhow::anyhow!("Max retries reached"))

}

pub async fn fetch_steam_profile(steam_id: &str, api_key: &str) -> anyhow::Result<SteamProfile> {
    let url = format!(
        "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/?key={}&steamids={}",
        api_key, steam_id
    );

    let client = Client::new();
    let response = client.get(&url).send().await?;
    if response.status().is_success() {
        let profile_data = response.json::<SteamProfileResponse>().await?;
        if let Some(profile) = profile_data.response.players.into_iter().next() {
            Ok(profile)
        }
        else {
            Err(anyhow::anyhow!("No Steam profile found for the provided Steam ID!"))
        }
    }
    else {
        Err(anyhow::anyhow!("Failed to fetch Steam profile for steam ID: {}; Status: {}",
            steam_id, response.status().as_u16()))
    }
}