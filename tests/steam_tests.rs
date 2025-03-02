// test dependencies
use shuttle_runtime::__internals::serde_json;

mod common;
use common::STEAM_ID;
use common::DATABASE_TEST_URL;
use common::DISCORD_ID;
use common::DISCORD_USERNAME;
use game_recommender::database::db;
use game_recommender::steam::*;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path, query_param};


#[tokio::test]
async fn test_mocked_fetch_steam_games() {
    let mock_server = MockServer::start().await;

    // Mock response data
    let mock_games = vec![
        SteamGame { name: "Test Game 1".to_string(), playtime_forever: 600 },
        SteamGame { name: "Test Game 2".to_string(), playtime_forever: 1200 },
    ];

    let response = serde_json::json!({
        "response": { "games": mock_games }
    });

    Mock::given(method("GET"))
        .and(path("/IPlayerService/GetOwnedGames/v1/"))
        .and(query_param("key", "test_api_key"))
        .and(query_param("steamid", "test_steam_id_12345"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&mock_server)
        .await;


    // Use the mock server instead of the real Steam API
    let result = fetch_steam_games(&mock_server.uri(), "test_steam_id_12345", "test_api_key").await;

    assert!(result.is_ok());
    let games = result.unwrap();
    assert_eq!(games.len(), mock_games.len());
    assert_eq!(games[0].name, "Test Game 1");
    assert_eq!(games[1].playtime_forever, 1200);
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

    sqlx::query!("DELETE FROM games;")
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
    let mock_server = MockServer::start().await;

    // Prepare mock response data for the Steam API
    let mock_games = vec![
        SteamGame { name: "Test Game 1".to_string(), playtime_forever: 600 },
        SteamGame { name: "Test Game 2".to_string(), playtime_forever: 1200 },
    ];
    let response = serde_json::json!({
        "response": { "games": mock_games }
    });

    // Mount the mock for the expected GET request.
    // Adjust the query parameters as needed. Here we assume STEAM_ID is a constant string.
    Mock::given(method("GET"))
        .and(path("/IPlayerService/GetOwnedGames/v1/"))
        .and(query_param("key", "test_api_key"))
        .and(query_param("steamid", &*STEAM_ID))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&mock_server)
        .await;

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
    let games_vec = fetch_steam_games(&mock_server.uri(), &STEAM_ID, "test_api_key")
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
    let mock_server = MockServer::start().await;

    // Prepare mock response data for the Steam API
    let mock_games = vec![
        SteamGame { name: "Test Game 1".to_string(), playtime_forever: 600 },
        SteamGame { name: "Test Game 2".to_string(), playtime_forever: 1200 },
    ];
    let response = serde_json::json!({
        "response": { "games": mock_games }
    });

    // Mount the mock for the expected GET request.
    // Adjust the query parameters as needed. Here we assume STEAM_ID is a constant string.
    Mock::given(method("GET"))
        .and(path("/IPlayerService/GetOwnedGames/v1/"))
        .and(query_param("key", "test_api_key"))
        .and(query_param("steamid", &*STEAM_ID))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&mock_server)
        .await;

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
    let games_vec = fetch_steam_games(&mock_server.uri(), &STEAM_ID, "test_api_key")
        .await.expect("Failed to fetch steam games");

    let owned_games = SteamOwnedGames { games: games_vec };

    db::store_steam_games(&connection, &STEAM_ID, owned_games)
        .await.expect("Failed to store steam games");

    let user_games = db::get_user_games(&connection, &STEAM_ID)
        .await.expect("Failed to fetch steam games from database");

    assert!(!user_games.is_empty(), "The user games is not empty.");
}

#[tokio::test]
async fn test_game_data_update_behavior() {
    let connection = sqlx::PgPool::connect(&DATABASE_TEST_URL)
        .await
        .expect("Failed to connect");

    // Clean up games for this STEAM_ID.
    sqlx::query!("DELETE FROM games WHERE steam_id = $1", STEAM_ID.to_string())
        .execute(&connection)
        .await
        .unwrap();

    // Insert initial game data.
    let initial_game = SteamGame {
        name: "New Game".to_string(),
        playtime_forever: 600,
    };
    let owned_games = SteamOwnedGames { games: vec![initial_game.clone()] };
    db::store_steam_games(&connection, &STEAM_ID, owned_games)
        .await
        .expect("Failed to store initial game");

    // Update the same game with new playtime.
    let updated_game = SteamGame {
        name: "New Game".to_string(),
        playtime_forever: 1200,
    };
    let updated_owned_games = SteamOwnedGames { games: vec![updated_game.clone()] };
    db::store_steam_games(&connection, &STEAM_ID, updated_owned_games)
        .await
        .expect("Failed to update game");

    let games = db::get_user_games(&connection, &STEAM_ID)
        .await
        .expect("Failed to fetch games");
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].playtime_forever, 1200, "Game playtime should be updated to 1200");
}
