// test dependencies
mod common;
use common::STEAM_API_KEY;
use common::STEAM_ID;
use common::DATABASE_TEST_URL;
use common::DISCORD_ID;
use common::DISCORD_USERNAME;
use game_recommender::database::db;
use game_recommender::steam::fetch_steam_games;

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
