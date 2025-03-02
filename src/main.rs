mod bot;
mod database;
mod steam;
#[path = "../cron/scheduler.rs"]
mod scheduler;

use bot::Bot;
use scheduler::start_scheduler;
use anyhow::Context as _;
use shuttle_runtime::SecretStore;
use serenity::prelude::*;
use tracing::error;

/// **Main function that initializes the bot**
#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
    #[shuttle_shared_db::Postgres] db_conn: String,
) -> shuttle_serenity::ShuttleSerenity {
    // Connect to the database
    let connection = sqlx::PgPool::connect(&db_conn)
        .await
        .expect("Failed to connect to the database");

    // Retrieve API keys from Shuttle Secrets
    let token = secrets
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let steam_api_key = secrets
        .get("STEAM_API_KEY")
        .expect("STEAM_API_KEY missing");

    // Clone for scheduler
    let scheduler_connection = connection.clone();
    let scheduler_api_key = steam_api_key.clone();

    // Start the background job scheduler
    let _scheduler = tokio::spawn(async move {
        if let Err(e) = start_scheduler(scheduler_connection, scheduler_api_key).await {
            error!("Failed to start scheduler: {:?}", e);
        }
    });

    // Configure Discord bot with event handlers
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let bot = Bot {
        database: connection,
        steam_api_key,
    };

    let client = Client::builder(&token, intents)
        .event_handler(bot)
        .await
        .expect("Error creating client");

    // Start the bot
    Ok(client.into())
}
