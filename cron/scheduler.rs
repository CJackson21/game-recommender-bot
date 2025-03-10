use sqlx::PgPool;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::error;

use crate::database::db::{get_all_steam_ids, store_steam_games};
use crate::steam::{fetch_steam_games, SteamOwnedGames};

const API_URL: &str = "https://api.steampowered.com";

/// Function to sync the database with updated games
pub async fn sync_all_users_games(
    pool: &PgPool,
    steam_api_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Syncing all users' games...");

    // Fetch all user steam IDs
    match get_all_steam_ids(pool).await {
        Ok(steam_ids) => {
            println!("Fetched {} Steam IDs", steam_ids.len());

            for steam_id in steam_ids {
                match fetch_steam_games(API_URL, &steam_id, steam_api_key).await {
                    Ok(games) => {
                        let owned_games = SteamOwnedGames { games };
                        if let Err(e) = store_steam_games(pool, &steam_id, owned_games).await {
                            error!("Failed to store games for Steam ID {}: {:?}", steam_id, e);
                        } else {
                            println!("Successfully updated games for Steam ID {}", steam_id);
                        }
                    }
                    Err(e) => error!("Failed to fetch games for Steam ID {}: {:?}", steam_id, e),
                }
            }
        }
        Err(e) => error!("Failed to fetch user Steam IDs: {:?}", e),
    }

    Ok(())
}

/// Daily scheduler to run `sync_all_users_games`
pub async fn start_scheduler(
    pool: PgPool,
    steam_api_key: String,
) -> Result<JobScheduler, Box<dyn std::error::Error>> {
    let scheduler = JobScheduler::new().await?;

    let pool = std::sync::Arc::new(pool);
    let api_key = std::sync::Arc::new(steam_api_key);

    // Schedule a job to run every day at 3 AM
    let job = {
        let pool = Arc::clone(&pool);
        let api_key = Arc::clone(&api_key);

        Job::new_async("0 3 * * *", move |_uuid, _l| {
            let pool = Arc::clone(&pool);
            let api_key = Arc::clone(&api_key);
            Box::pin(async move {
                if let Err(e) = sync_all_users_games(&pool, &api_key).await {
                    error!("Daily sync failed: {:?}", e);
                } else {
                    println!("Daily sync completed.");
                }
            })
        })?
    };

    scheduler.add(job).await?;
    scheduler.start().await?;

    Ok(scheduler)
}
