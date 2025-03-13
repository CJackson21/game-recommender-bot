mod bot;
mod database;
mod llm;
#[path = "../cron/scheduler.rs"]
mod scheduler;
mod steam;

use crate::llm::LLMClient;
use bot::Bot;
use scheduler::start_scheduler;
use serenity::prelude::*;
use serenity::model::id::ChannelId;
use shuttle_runtime::SecretStore;
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
    let token = secrets.get("DISCORD_TOKEN").expect("DISCORD_TOKEN missing");

    let steam_api_key = secrets.get("STEAM_API_KEY").expect("STEAM_API_KEY missing");

    let llm_api_key = secrets.get("LLM_API_KEY").expect("LLM_API_KEY missing");

    let channel_id = ChannelId::new(
        secrets
            .get("DISCORD_CHANNEL_ID")
            .expect("DISCORD_CHANNEL_ID missing")
            .parse::<u64>()
            .expect("Invalid DISCORD_CHANNEL_ID format"),
    );

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
    let intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let bot = Bot {
        database: connection,
        steam_api_key,
        llm_client: LLMClient::new(&llm_api_key),
        channel_id
    };

    let client = Client::builder(&token, intents)
        .event_handler(bot)
        .await
        .expect("Error creating client");

    // Start the bot
    Ok(client.into())
}
