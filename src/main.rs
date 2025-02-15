mod database;
use database::db;
mod steam;

use anyhow::Context as _;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::{error, info};

struct Bot {
    database: sqlx::PgPool,
    steam_api_key: String,
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        let args: Vec<&str> = msg.content.split_whitespace().collect();

        // !link_steam <steam_id> command
        if args.len() == 2 && args[0] == "!link_steam" {
            let steam_id = args[1];
            let discord_id = msg.author.id.get() as i64;
            let author_name = &msg.author.name;

            match db::link_steam(&self.database, author_name, discord_id, steam_id).await {
                Ok(_) => {
                    if let Err(e) = msg.channel_id
                        .say(&ctx.http, format!("Successfully linked Steam ID `{}` to your Discord account!", steam_id))
                        .await
                    {
                        error!("Error sending message: {:?}", e);
                    }
                }
                Err(e) => {
                    error!("Database error: {:?}", e);
                    msg.channel_id.say(&ctx.http, "Failed to link Steam ID. Please try again later.").await.ok();
                }
            }
        }

        // !steam_games command
        if args[0] == "!steam_games" {
            let discord_id = msg.author.id.get() as i64;

            match db::get_steam_id(&self.database, discord_id).await {
                Ok(Some(steam_id)) => {
                    match steam::fetch_steam_games(&steam_id, &self.steam_api_key).await {
                        Ok(games) => {
                            if let Some(top_game) = games.iter().max_by_key(|g| g.playtime_forever) {
                                let response = format!(
                                    "Your most played game: **{}** ({} hours)",
                                    top_game.name,
                                    top_game.playtime_forever / 60
                                );
                                msg.channel_id.say(&ctx.http, response).await.ok();
                            } else {
                                msg.channel_id.say(&ctx.http, "No games found in your Steam account.").await.ok();
                            }
                        }
                        Err(e) => {
                            error!("Error fetching Steam games: {:?}", e);
                            msg.channel_id.say(&ctx.http, "Error retrieving Steam data.").await.ok();
                        }
                    }
                }
                Ok(None) => {
                    msg.channel_id
                        .say(&ctx.http, "You haven't linked your Steam ID yet! Use `!link_steam <steam_id>`.")
                        .await.ok();
                }
                Err(e) => {
                    error!("Database error fetching Steam ID: {:?}", e);
                    msg.channel_id.say(&ctx.http, "Database error. Please try again later.").await.ok();
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
    #[shuttle_shared_db::Postgres] db_conn: String,
) -> shuttle_serenity::ShuttleSerenity {
    let pool = sqlx::PgPool::connect(&db_conn)
        .await
        .expect("Failed to connect to the database");

    let token = secrets
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let steam_api_key = match secrets.get("STEAM_API_KEY") {
        Some(key) => key,
        None => {
            error!("STEAM_API_KEY is missing. Make sure it's set in Secrets.toml.");
            return Err(anyhow::anyhow!("STEAM_API_KEY missing").into());
        }
    };

    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let bot = Bot { database: pool.clone(), steam_api_key };

    let client = Client::builder(&token, intents)
        .event_handler(bot)
        .await
        .expect("Err creating client");

    Ok(client.into())
}
