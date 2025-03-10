use crate::steam::SteamGame;
use crate::steam::SteamOwnedGames;
use sqlx::PgPool;

/// Links a user's steam account to their discord via database
pub async fn link_steam(
    pool: &PgPool,
    username: &str,
    discord_id: i64,
    steam_id: &str,
) -> Result<(), sqlx::Error> {
    // This will associate the user's steam account with their discord account
    sqlx::query!(
        "INSERT INTO users(username, discord_id, steam_id) VALUES ($1, $2, $3)
         ON CONFLICT(discord_id) DO UPDATE SET steam_id = $3;",
        username,
        discord_id,
        steam_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Gets the user's steam id from the database
pub async fn get_steam_id(pool: &PgPool, discord_id: i64) -> Result<Option<String>, sqlx::Error> {
    let steam_id = sqlx::query!(
        "SELECT steam_id FROM users WHERE discord_id = $1;",
        discord_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(steam_id.map(|record| record.steam_id))
}

/// Stores a user's steam games into the database
pub async fn store_steam_games(
    pool: &PgPool,
    steam_id: &str,
    owned_games: SteamOwnedGames,
) -> Result<(), sqlx::Error> {
    // Build the query because it is FAR faster
    let mut query_builder = sqlx::QueryBuilder::new(
        "INSERT INTO games (steam_id, name, playtime_forever, last_updated) ",
    );

    query_builder.push_values(owned_games.games.iter(), |mut row_builder, game| {
        row_builder
            .push_bind(steam_id)
            .push_bind(&game.name)
            .push_bind(game.playtime_forever as i32)
            .push("NOW()");
    });

    // Finish building the query after iterating through the games
    query_builder.push(
        " ON CONFLICT (steam_id, name)
          DO UPDATE SET playtime_forever = EXCLUDED.playtime_forever,
          last_updated = NOW();",
    );

    let query = query_builder.build();
    query.execute(pool).await?;
    Ok(())
}

/// Fetches the user's steam games from the database
pub async fn get_user_games(pool: &PgPool, steam_id: &str) -> Result<Vec<SteamGame>, sqlx::Error> {
    let records = sqlx::query!(
        "SELECT name, playtime_forever FROM games WHERE steam_id = $1;",
        steam_id
    )
    .fetch_all(pool)
    .await?;

    let games = records
        .into_iter()
        .map(|rec| SteamGame {
            name: rec.name,
            playtime_forever: rec.playtime_forever as u32,
        })
        .collect();

    Ok(games)
}

/// Fetches all user steam id's from the database
pub async fn get_all_steam_ids(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    // Query for the steam IDs
    let steam_ids = sqlx::query!("SELECT steam_id FROM users;")
        .fetch_all(pool)
        .await?;

    let steam_ids = steam_ids.into_iter().map(|row| row.steam_id).collect();
    Ok(steam_ids)
}
