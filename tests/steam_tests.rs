use dotenvy::dotenv;
use std::env;
use game_recommender::steam::fetch_steam_games;
#[tokio::test]
async fn test_fetch_steam_games() {

    dotenv().expect("Failed to load .env file");

    let api_key = env::var("STEAM_API_KEY").expect("STEAM_API_KEY not found");
    let steam_id = env::var("STEAM_ID").expect("STEAM_ID not found");

    match fetch_steam_games(&steam_id, &api_key).await {
        Ok(games) => {
            assert!(!games.is_empty(), "No games found");
            for game in &games {
                println!("{} - {} hours", game.name, game.playtime_forever / 60);
            }
        }
        Err(e) => panic!("Error fetching Steam data: {:?}", e),
    }
}
