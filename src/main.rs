mod bot;
mod database;
mod llm;
#[path = "../cron/scheduler.rs"]
mod scheduler;
mod steam;

use crate::llm::LLMClient;
use bot::Bot;
use scheduler::start_scheduler;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use std::env;
use dotenvy::dotenv;
use tracing::error;

/// **Main function that initializes the bot**
#[tokio::main]
async fn main() {
    // Load .env file
    dotenv().ok();

    // Load secrets from .env
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN missing");
    let steam_api_key = env::var("STEAM_API_KEY").expect("STEAM_API_KEY missing");
    let llm_api_key = env::var("LLM_API_KEY").expect("LLM_API_KEY missing");
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL missing");
    let channel_id = ChannelId::new(
        env::var("DISCORD_CHANNEL_ID")
            .expect("DISCORD_CHANNEL_ID missing")
            .parse::<u64>()
            .expect("Invalid DISCORD_CHANNEL_ID format"),
    );

    // Connect to the database
    let connection = sqlx::PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to the database");

    // Start the scheduler
    let scheduler_connection = connection.clone();
    let scheduler_api_key = steam_api_key.clone();
    let _scheduler = tokio::spawn(async move {
        if let Err(e) = start_scheduler(scheduler_connection, scheduler_api_key).await {
            error!("Failed to start scheduler: {:?}", e);
        }
    });

    // Start the bot
    let intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let bot = Bot {
        database: connection,
        steam_api_key,
        llm_client: LLMClient::new(&llm_api_key),
        channel_id
    };

    let mut client = Client::builder(&token, intents)
        .event_handler(bot)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        eprintln!("Client ended unexpectedly: {:?}", why);
    }
}
