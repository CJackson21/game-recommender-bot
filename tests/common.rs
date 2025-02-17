// test dependencies
use once_cell::sync::Lazy;
use std::env;
use dotenvy::dotenv;

pub static STEAM_API_KEY: Lazy<String> = Lazy::new(|| {
    dotenv().ok();
    env::var("STEAM_API_KEY").expect("STEAM_API_KEY not set")
});

pub static STEAM_ID: Lazy<String> = Lazy::new(|| {
    dotenv().ok();
    env::var("STEAM_ID").expect("STEAM_ID not set")
});

pub static DATABASE_TEST_URL: Lazy<String> = Lazy::new(|| {
    dotenv().ok();
    env::var("DATABASE_TEST_URL").expect("DATABASE_TEST_URL not set")
});


pub static DISCORD_ID: Lazy<i64> = Lazy::new(|| {
    dotenv().ok();
    env::var("DISCORD_ID")
        .expect("DISCORD_ID not set")
        .parse::<i64>()
        .expect("DISCORD_ID must be a valid i64")
});
pub static DISCORD_USERNAME: Lazy<String> = Lazy::new(|| {
    dotenv().ok();
    env::var("DISCORD_USERNAME").expect("DISCORD_USERNAME not set")
});
