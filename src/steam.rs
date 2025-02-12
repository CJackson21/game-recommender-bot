use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
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
struct SteamOwnedGames {
    games: Vec<SteamGame>,
}

pub async fn fetch_steam_games(steam_id: &str, api_key: &str) -> Result<Vec<SteamGame>, reqwest::Error> {
    let url = format!("https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?key={}&steamid={}&format=json&include_appinfo=true",
                              api_key, steam_id
    );

    let client = Client::new();
    let steam_data  = client.get(&url).send().await?.json::<SteamResponse>().await?.response;

    Ok(steam_data .games)
}