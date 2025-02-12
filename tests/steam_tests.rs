#[tokio::test]
async fn test_fetch_steam_games() {
    use game_recommender::steam::fetch_steam_games;

    let steam_id = "76561199351744930";
    let api_key = std::env::var("STEAM_API_KEY").expect("STEAM_API_KEY not found");

    match fetch_steam_games(steam_id, &api_key).await {
        Ok(games) => {
            assert!(!games.is_empty(), "No games found");
            for game in &games[..5] {
                println!("{} - {} hours", game.name, game.playtime_forever / 60);
            }
        }
        Err(e) => panic!("Error fetching Steam data: {:?}", e),
    }
}
