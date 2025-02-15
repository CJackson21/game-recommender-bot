use sqlx::{ PgPool };
use game_recommender::steam::SteamOwnedGames;

pub struct User {
    pub discord_id: i64,
    pub username: String,
    pub steam_id: String,
}

pub async fn link_steam(pool: &PgPool, username: &str, discord_id: i64, steam_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO users(username, discord_id, steam_id) VALUES ($1, $2, $3)
         ON CONFLICT(discord_id) DO UPDATE SET steam_id = $3;",
        username,
        discord_id,
        steam_id,
    ).execute(pool).await?;

    Ok(())
}

pub async fn get_steam_id(pool: &PgPool, discord_id: i64) -> Result<Option<String>, sqlx::Error> {
    let record = sqlx::query!(
        "SELECT steam_id FROM users WHERE discord_id = $1;",
        discord_id
    )
        .fetch_optional(pool)
        .await?;

    Ok(record.map(|r| r.steam_id))
}

pub async fn store_steam_games(pool: &PgPool, steam_id: &str, owned_games: SteamOwnedGames) -> Result<(), sqlx::Error> {
    for game in owned_games.into_iter() {
        sqlx::query!(
            "INSERT INTO games (steam_id, name, playtime, last_updated)
             VALUES ($1, $2, $3, NOW())
             ON CONFLICT(steam_id, name) DO UPDATE
             SET playtime = EXCLUDED.playtime, last_updated = Now();",
            steam_id,
            game.name,
            game.playtime,
        ).execute(pool).await?;
    }

    Ok(())
}

pub async fn get_user_games(pool: &PgPool, steam_id: &str) -> Result<Vec<String>, sqlx::Error> {
    let games = sqlx::query!(
        "SELECT name FROM games WHERE steam_id = $1;",
        steam_id,
    ).fetch_all(pool).await?;

    // Extract the game names from the resulting query
    let game_names: Vec<String> = games.into_iter().map(|g| g.name).collect();

    Ok(game_names)
}

