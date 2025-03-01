use std::collections::HashMap;
// test dependencies
use chrono::Utc;
mod common;
use common::STEAM_API_KEY;
use common::STEAM_ID;
use common::DATABASE_TEST_URL;
use common::DISCORD_ID;
use common::DISCORD_USERNAME;
use game_recommender::database::db;
use game_recommender::steam::*;
use game_recommender::bot::Bot;
use tokio::sync::Mutex;


#[tokio::test]
async fn test_fetch_steam_games() {
    match fetch_steam_games(&STEAM_ID, &STEAM_API_KEY).await {
        Ok(games) => {
            assert!(!games.is_empty(), "No games found");
            // optional print for debugging
            // for game in &games {
            //     println!("{} - {} hours", game.name, game.playtime_forever / 60);
            // }
        }
        Err(e) => panic!("Error fetching Steam data: {:?}", e),
    }
}

#[tokio::test]
async fn test_link_and_get_steam_id() {
    // Set up a connection to a test database
    let connection = sqlx::PgPool::connect(&DATABASE_TEST_URL)
    .await.expect("Failed to connect to PostgreSQL database");

    // Clean up previous runs if necessary
    sqlx::query!("DELETE FROM users;")
        .execute(&connection)
        .await
        .unwrap();

    // Seed test data (use your steam id unless you want to set up a test steam account)
    db::link_steam(&connection, &DISCORD_USERNAME, *DISCORD_ID, &STEAM_ID)
        .await.expect("Failed to link steam");

    // Retrieve the steam ID and check
    let retrieved_id = db::get_steam_id(&connection, *DISCORD_ID)
        .await.unwrap();
    assert_eq!(retrieved_id, Some(STEAM_ID.to_string()));
}

#[tokio::test]
async fn test_populate_database_with_games() {
    let connection = sqlx::PgPool::connect(&DATABASE_TEST_URL)
    .await.expect("Failed to connect to PostgreSQL database");

    // Clean up previous runs if necessary
    sqlx::query!("DELETE FROM users;")
        .execute(&connection)
        .await
        .unwrap();

    sqlx::query!("DELETE FROM games;")
        .execute(&connection)
        .await
        .unwrap();

    // Seed test data (use your steam id unless you want to set up a test steam account)
    db::link_steam(&connection, &DISCORD_USERNAME, *DISCORD_ID, &STEAM_ID)
        .await.expect("Failed to link steam");

    // Fetch Steam games from the API.
    let games_vec = fetch_steam_games(&STEAM_ID, &STEAM_API_KEY)
        .await.expect("Failed to fetch steam games");

    // Wrap the vector into a SteamOwnedGames struct
    let owned_games = SteamOwnedGames { games: games_vec };

    // Store steam games (what is actually being tested here)
    db::store_steam_games(&connection, &STEAM_ID, owned_games)
        .await.expect("Failed to store steam games");

    let stored_games = db::get_user_games(&connection, &STEAM_ID)
        .await.expect("Failed to fetch steam games from database");

    assert!(
        !stored_games.is_empty(),
        "No games were stored in the database."
    );
}

#[tokio::test]
async fn test_fetch_user_games() {
    let connection = sqlx::PgPool::connect(&DATABASE_TEST_URL)
        .await.expect("Failed to connect to PostgreSQL database");

    // Ensure the user does not exist before running the test
    sqlx::query!("DELETE FROM users WHERE steam_id = $1", STEAM_ID.to_string())
        .execute(&connection)
        .await
        .expect("Failed to delete existing user");

    sqlx::query!("DELETE FROM games WHERE steam_id = $1", STEAM_ID.to_string())
        .execute(&connection)
        .await
        .expect("Failed to delete existing games");

    // Insert test user
    db::link_steam(&connection, &DISCORD_USERNAME, *DISCORD_ID, &STEAM_ID)
        .await.expect("Failed to link steam");

    // Fetch Steam games
    let games_vec = fetch_steam_games(&STEAM_ID, &STEAM_API_KEY)
        .await.expect("Failed to fetch steam games");

    let owned_games = SteamOwnedGames { games: games_vec };

    db::store_steam_games(&connection, &STEAM_ID, owned_games)
        .await.expect("Failed to store steam games");

    let user_games = db::get_user_games(&connection, &STEAM_ID)
        .await.expect("Failed to fetch steam games from database");

    assert!(!user_games.is_empty(), "The user games is not empty.");
}

#[tokio::test]
async fn test_cache_logic() {
    let connection = sqlx::PgPool::connect(&DATABASE_TEST_URL)
        .await.expect("Failed to connect to PostgreSQL database");

    // Create bot with empty cache
    let bot = Bot {
        database: connection,
        steam_api_key: STEAM_API_KEY.to_string(),
        cache: Mutex::new(HashMap::new()),
    };

    // Insert a fake entry into the cache
    {
        let mut cache_lock = bot.cache.lock().await;
        let steam_id = STEAM_ID.to_string();
        let fake_game = SteamGame {
            appid: 12345,
            name: "CacheTestGame".to_string(),
            playtime_forever: 500,
        };

        cache_lock.insert(steam_id.clone(), (vec![fake_game], Utc::now().timestamp()));
    }

    // Now check if the cache works correctly
    {
        let cache_lock = bot.cache.lock().await;
        let steam_id = STEAM_ID.to_string();
        let maybe_entry = cache_lock.get(&steam_id);
        assert!(maybe_entry.is_some(), "Expected cached entry for STEAM_ID");

        if let Some((games, timestamp)) = maybe_entry {
            assert_eq!(games.len(), 1, "Should have exactly 1 game cached.");
            assert_eq!(games[0].name, "CacheTestGame", "Cached game name mismatch");

            let age = Utc::now().timestamp() - *timestamp;
            assert!(age < 600, "Cache entry should be fresh, but it's {} seconds old", age);
        }
    }
}
