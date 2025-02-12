use sqlx::{ PgPool };

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

