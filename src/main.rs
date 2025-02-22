mod database;
use database::db;
mod steam;
#[path = "../cron/scheduler.rs"]
mod scheduler;

use anyhow::Context as _;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use scheduler::start_scheduler;
use tracing::{error, info};

struct Bot {
    database: sqlx::PgPool,
    steam_api_key: String,
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
    async fn handle_link_steam(&self, ctx: &Context, msg: &Message, steam_id: &str) {
        let discord_id = msg.author.id.get() as i64;
        let author_name = &msg.author.name;

        // Link the user's Steam ID in the database.
        if let Err(e) = db::link_steam(&self.database, author_name, discord_id, steam_id).await {
            error!("Database error linking Steam ID: {:?}", e);
            let _ = msg.channel_id
                .say(&ctx.http, "Failed to link Steam ID. Please try again later.")
                .await;
            return;
        }

        // Notify the user that linking was successful.
        let _ = msg.channel_id
            .say(&ctx.http, format!("Successfully linked Steam ID `{}` to your Discord account!", steam_id))
            .await;

        // Fetch games from the Steam API.
        match steam::fetch_steam_games(steam_id, &self.steam_api_key).await {
            Ok(games_vector) => {
                let steam_owned_games = steam::SteamOwnedGames { games: games_vector };
                // Store the fetched games in the database.
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
    async fn handle_steam_games(&self, ctx: &Context, msg: &Message) {
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
    async fn fetch_and_display_steam_games(&self, ctx: &Context, msg: &Message, steam_id: &str) {
        match steam::fetch_steam_games(steam_id, &self.steam_api_key).await {
            Ok(games) => {
                if let Some(top_game) = games.iter()
                    .max_by_key(|g| g.playtime_forever) {
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


#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
    #[shuttle_shared_db::Postgres] db_conn: String,
) -> shuttle_serenity::ShuttleSerenity {
    let connection = sqlx::PgPool::connect(&db_conn)
        .await
        .expect("Failed to connect to the database");

    let token = secrets
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let steam_api_key = secrets.get("STEAM_API_KEY")
        .expect("STEAM_API_KEY missing");

    // Clone the connection and API key for the scheduler
    let scheduler_connection = connection.clone();
    let scheduler_api_key = steam_api_key.clone();

    let _scheduler = tokio::spawn(async move {
        if let Err(e) = start_scheduler(scheduler_connection, scheduler_api_key)
            .await { eprintln!("Failed to start scheduler: {:?}", e); }
    });

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let bot = Bot { database: connection, steam_api_key };

    let client = Client::builder(&token, intents)
        .event_handler(bot)
        .await
        .expect("Err creating client");

    Ok(client.into())
}
