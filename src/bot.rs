use crate::database::db;
use game_recommender::llm::LLMClient;
use crate::steam::{fetch_steam_profile, SteamGame};
use serenity::async_trait;
use serenity::collector::MessageCollector;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::time::Duration;
use tracing::{error, info};

const API_URL: &str = "https://api.steampowered.com";

/// Struct representing the bot, including a database pool, API key, and cache.
pub struct Bot {
    pub database: sqlx::PgPool,
    pub steam_api_key: String,
    pub llm_client: LLMClient,
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        let args: Vec<&str> = msg.content.split_whitespace().collect();
        if args.is_empty() {
            return;
        }

        match args[0] {
            "!link_steam" => {
                if args.len() == 2 {
                    self.handle_link_steam(&ctx, &msg, args[1]).await;
                } else {
                    // Tell the user they need to provide a Steam ID
                    if let Err(e) = msg.channel_id.say(
                        &ctx.http,
                        "Please provide your Steam ID after the command. \
                         For help finding your Steam ID, visit: \
                         https://www.ubisoft.com/en-gb/help/account/article/finding-your-steam-id/000060565",
                    ).await {
                        eprintln!("Error sending message: {:?}", e);
                    }
                }
            }
            "!steam_games" => {
                self.handle_steam_games(&ctx, &msg).await;
            }
            "!top_games" => {
                self.display_top_games(&ctx, &msg).await;
            }
            "!recommend" => {
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
    /// Helper to retrieve a user's Steam games.
    /// Checks the cache first (if data is less than 600s old); otherwise, fetches from the database.
    async fn get_steam_games(&self, steam_id: &str) -> Option<Vec<SteamGame>> {
        match db::get_user_games(&self.database, steam_id).await {
            Ok(games) if !games.is_empty() => Some(games),
            Ok(_) => {
                error!("No games found in the database for steam_id: {}", steam_id);
                None
            },
            Err(e) => {
                error!("Error fetching games from database: {:?}", e);
                None
            }
        }
    }


    /// Handles the `!link_steam <steam_id>` command
    pub async fn handle_link_steam(&self, ctx: &Context, msg: &Message, steam_id: &str) {
        // Attempt to fetch the Steam profile to validate the provided Steam ID.
        let profile_data = match fetch_steam_profile(steam_id, &self.steam_api_key).await {
            Ok(profile) => profile,
            Err(err) => {
                error!("Error fetching Steam profile: {:?}", err);
                let _ = msg.channel_id.say(
                    &ctx.http,
                    "Error fetching Steam profile. Please make sure your Steam ID is correct!",
                ).await;
                return;
            }
        };

        // Send a confirmation for the user to link their account.
        let confirmation_message = format!(
            "Here is the Steam profile with the associated Steam ID: **{}**.\n\
             Reply with `yes` within 30 seconds to confirm linking, or `no` to cancel.",
            profile_data.persona_name
        );

        if let Err(err) = msg.channel_id.say(&ctx.http, confirmation_message).await {
            eprintln!("Error sending confirmation message: {:?}", err);
            return;
        }

        // Use a MessageCollector instead of await_reply.
        let collector = MessageCollector::new(ctx)
            .channel_id(msg.channel_id)
            .author_id(msg.author.id)
            .timeout(Duration::from_secs(30));

        // Grab the next matching message.
        let user_reply = match collector.next().await {
            Some(reply_msg) => reply_msg.content.to_lowercase(),
            None => {
                let _ = msg.channel_id.say(
                    &ctx.http,
                    "No confirmation was received. Please run the command again if you wish to link your Steam account.",
                ).await;
                return;
            }
        };

        // Check if the user replied "yes".
        if user_reply.starts_with("yes") {
            let discord_id = msg.author.id.get() as i64;
            let author_name = &msg.author.name;

            // Link the Steam ID in the DB.
            if let Err(e) = db::link_steam(&self.database, author_name, discord_id, steam_id).await {
                error!("Database error linking Steam ID: {:?}", e);
                let _ = msg.channel_id.say(
                    &ctx.http,
                    "Failed to link Steam ID. Please try again later.",
                ).await;
                return;
            }

            let _ = msg.channel_id.say(
                &ctx.http,
                format!("Successfully linked Steam ID `{}` to your Discord account!", steam_id),
            ).await;

            // Try to fetch and store the user's Steam games.
            match crate::steam::fetch_steam_games(API_URL, steam_id, &self.steam_api_key).await {
                Ok(games_vector) => {
                    let steam_owned_games = crate::steam::SteamOwnedGames { games: games_vector };
                    if let Err(e) = db::store_steam_games(&self.database, steam_id, steam_owned_games).await {
                        error!("Failed to store games in database: {:?}", e);
                        let _ = msg.channel_id.say(&ctx.http, "Error storing games in the database.").await;
                    } else {
                        let _ = msg.channel_id.say(&ctx.http, "Steam games successfully updated in the database!").await;
                    }
                }
                Err(e) => {
                    error!("Failed to fetch games from Steam API: {:?}", e);
                    let _ = msg.channel_id.say(&ctx.http, "Error retrieving Steam data.").await;
                }
            }
        } else {
            let _ = msg.channel_id.say(
                &ctx.http,
                "Canceled. Please run the command again if you wish to link your Steam account.",
            ).await;
        }
    }

    /// Handles the `!steam_games` command by fetching and displaying the user's most played game.
    pub async fn handle_steam_games(&self, ctx: &Context, msg: &Message) {
        let discord_id = msg.author.id.get() as i64;
        match db::get_steam_id(&self.database, discord_id).await {
            Ok(Some(steam_id)) => {
                self.fetch_and_display_steam_games(ctx, msg, &steam_id).await;
            }
            Ok(None) => {
                let _ = msg.channel_id.say(
                    &ctx.http,
                    "You haven't linked your Steam ID yet! Use `!link_steam <steam_id>`.",
                ).await;
            }
            Err(e) => {
                error!("Database error fetching Steam ID: {:?}", e);
                let _ = msg.channel_id.say(&ctx.http, "Database error. Please try again later.").await;
            }
        }
    }

    /// Fetches and displays the user's most played game.
    pub async fn fetch_and_display_steam_games(&self, ctx: &Context, msg: &Message, steam_id: &str) {
        if let Some(games) = self.get_steam_games(steam_id).await {
            match games.iter().max_by_key(|g| g.playtime_forever) {
                Some(top_game) => {
                    let response = format!(
                        "Your most played game: **{}** ({} hours)",
                        top_game.name,
                        top_game.playtime_forever / 60
                    );
                    let _ = msg.channel_id.say(&ctx.http, response).await;
                }
                None => {
                    let _ = msg.channel_id.say(&ctx.http, "No games found in your Steam account.").await;
                }
            }
        } else {
            let _ = msg.channel_id.say(&ctx.http, "Error retrieving Steam data.").await;
        }
    }

    /// Displays the user's top 5 games by playtime.
    pub async fn display_top_games(&self, ctx: &Context, msg: &Message) {
        let discord_id = msg.author.id.get() as i64;
        let steam_id = match db::get_steam_id(&self.database, discord_id).await {
            Ok(Some(id)) => id,
            Ok(None) => {
                let _ = msg.channel_id.say(
                    &ctx.http,
                    "You haven't linked your Steam ID yet! Use `!link_steam <steam_id>`.",
                ).await;
                return;
            }
            Err(e) => {
                error!("Error retrieving Steam ID: {:?}", e);
                let _ = msg.channel_id.say(&ctx.http, "Database error. Please try again later.").await;
                return;
            }
        };

        if let Some(games) = self.get_steam_games(&steam_id).await {
            let mut sorted_games = games.clone();
            sorted_games.sort_by_key(|g| std::cmp::Reverse(g.playtime_forever));
            let top_games: Vec<String> = sorted_games
                .iter()
                .take(5)
                .map(|game| format!("**{}** ({} hours)", game.name, game.playtime_forever / 60))
                .collect();
            let response_message = format!("Your top 5 played games:\n{}", top_games.join("\n"));
            let _ = msg.channel_id.say(&ctx.http, response_message).await;
        } else {
            let _ = msg.channel_id.say(&ctx.http, "Error retrieving Steam data.").await;
        }
    }

    /// Get recommendations based on game history
    pub async fn recommend_games(&self, ctx: &Context, msg: &Message) {
        let discord_id = msg.author.id.get() as i64;

        match db::get_steam_id(&self.database, discord_id).await {
            Ok(Some(steam_id)) => {
                let _ = msg.channel_id.say(
                    &ctx.http,format!("Getting recommendations for {}", msg.author)).await;

                // Fetch recommendations
                match self.llm_client.get_recommendation(&self.database, &steam_id).await {
                    Ok(recommendations) => {
                        let _ = msg.channel_id.say(
                            &ctx.http,
                            format!("Based on your game history, you might enjoy: \n{}",
                                    recommendations)).await;
                    }
                    Err(e) => {
                        error!("Error generating a response: {:?}", e);
                        let _ = msg.channel_id.say(
                            &ctx.http, "Error generating a response.").await;
                    }
                }
            }
            Ok(None) => {
                let _ = msg.channel_id.say(
                    &ctx.http,
                    "You haven't linked your Steam ID yet! Use `!link_steam <steam_id>`.",
                ).await;
                return;
            }
            Err(e) => {
                error!("Error retrieving Steam ID: {:?}", e);
                let _ = msg.channel_id.say(&ctx.http, "Database error. Please try again later.").await;
                return;
            }
        }
    }
}
