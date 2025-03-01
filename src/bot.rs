use crate::database::db;
use crate::steam::SteamGame;
use chrono::Utc;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::{error, info};

/// Struct representing the bot, including a database pool, API key, and cache.
pub struct Bot {
    pub database: sqlx::PgPool,
    pub steam_api_key: String,
    pub cache: Mutex<HashMap<String, (Vec<SteamGame>, i64)>>,
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        let args: Vec<&str> = msg.content.split_whitespace().collect();
        if args.is_empty() {
            return;
        }

        match args[0] {
            "!link_steam" if args.len() == 2 => {
                self.handle_link_steam(&ctx, &msg, args[1]).await;
            }
            "!steam_games" => {
                self.handle_steam_games(&ctx, &msg).await;
            }
            _ => {}
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

impl Bot {
    /// Handles the `!link_steam <steam_id>` command
    pub async fn handle_link_steam(&self, ctx: &Context, msg: &Message, steam_id: &str) {
        let discord_id = msg.author.id.get() as i64;
        let author_name = &msg.author.name;

        if let Err(e) = db::link_steam(&self.database, author_name, discord_id, steam_id).await {
            error!("Database error linking Steam ID: {:?}", e);
            let _ = msg.channel_id
                .say(&ctx.http, "Failed to link Steam ID. Please try again later.")
                .await;
            return;
        }

        let _ = msg.channel_id
            .say(&ctx.http, format!("Successfully linked Steam ID `{}` to your Discord account!", steam_id))
            .await;

        match crate::steam::fetch_steam_games(steam_id, &self.steam_api_key).await {
            Ok(games_vector) => {
                let steam_owned_games = crate::steam::SteamOwnedGames { games: games_vector };
                if let Err(e) = db::store_steam_games(&self.database, steam_id, steam_owned_games)
                    .await {
                    error!("Failed to store games in database: {:?}", e);
                    let _ = msg.channel_id
                        .say(&ctx.http, "Error storing games in the database.")
                        .await;
                } else {
                    let _ = msg.channel_id
                        .say(&ctx.http, "Steam games successfully updated in the database!")
                        .await;
                }
            }
            Err(e) => {
                error!("Failed to fetch games from Steam API: {:?}", e);
                let _ = msg.channel_id
                    .say(&ctx.http, "Error retrieving Steam data.")
                    .await;
            }
        }
    }

    /// Handles the `!steam_games` command
    pub async fn handle_steam_games(&self, ctx: &Context, msg: &Message) {
        let discord_id = msg.author.id.get() as i64;

        match db::get_steam_id(&self.database, discord_id).await {
            Ok(Some(steam_id)) => {
                self.fetch_and_display_steam_games(ctx, msg, &steam_id).await;
            }
            Ok(None) => {
                msg.channel_id
                    .say(&ctx.http, "You haven't linked your Steam ID yet! Use `!link_steam <steam_id>`.")
                    .await.ok();
            }
            Err(e) => {
                error!("Database error fetching Steam ID: {:?}", e);
                msg.channel_id
                    .say(&ctx.http, "Database error. Please try again later.")
                    .await.ok();
            }
        }
    }

    /// Fetches and displays Steam games for a user
    pub async fn fetch_and_display_steam_games(&self, ctx: &Context, msg: &Message, steam_id: &str) {
        let mut cache = self.cache.lock().await;

        if let Some((games, timestamp)) = cache.get(steam_id) {
            if Utc::now().timestamp() - timestamp < 600 {
                msg.channel_id
                    .say(&ctx.http, format!("Your most played game: {}, {} hours",
                                            games[0].name, games[0].playtime_forever))
                    .await.ok();
                return;
            }
        }

        match crate::steam::fetch_steam_games(steam_id, &self.steam_api_key).await {
            Ok(games) => {
                cache.insert(steam_id.to_string(), (games.clone(), Utc::now().timestamp()));

                if let Some(top_game) = games.iter().max_by_key(|g| g.playtime_forever) {
                    let response = format!(
                        "Your most played game: **{}** ({} hours)",
                        top_game.name,
                        top_game.playtime_forever / 60
                    );
                    msg.channel_id.say(&ctx.http, response).await.ok();
                } else {
                    msg.channel_id
                        .say(&ctx.http, "No games found in your Steam account.")
                        .await.ok();
                }
            }
            Err(e) => {
                error!("Error fetching Steam games: {:?}", e);
                msg.channel_id.say(&ctx.http, "Error retrieving Steam data.")
                    .await.ok();
            }
        }
    }
}
