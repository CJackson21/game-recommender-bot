
# Project Title

A brief description of what this project does and who it's for

# 🎮 Game Recommender Discord Bot

A Rust-powered Discord bot that links to a user's Steam account, fetches their game library, and provides **game recommendations** (coming soon) using an LLM. Built with **Serenity**, **SQLx**, and **Shuttle**.

## 🚀 Features

- 🔗 **Link Steam Account**: Users can link their Steam ID to their Discord account.
- 📥 **Fetch Owned Games**: Fetches and stores the user's Steam library in a database.
- 🔄 **Sync Games**: Periodically updates the user's game data.
- 🔍 **Retrieve Game Data**: Users can see their most-played games.
- 🤖 **Game Recommendations** (WIP): Uses an **LLM-based** system to suggest new games.

---

## 📦 Installation

### 1️⃣ Prerequisites

- [Rust & Cargo](https://www.rust-lang.org/)
- [PostgreSQL](https://www.postgresql.org/)
- [Shuttle](https://www.shuttle.rs/) (for deployment)
- A **Discord bot token** and **Steam API key**

### 2️⃣ Clone the Repository

```sh
git clone https://github.com/yourusername/game-recommender.git
cd game-recommender

3️⃣ Set Up Environment Variables

Create a .env file (for local testing):

DATABASE_URL=postgres://username:password@localhost/production
DATABASE_TEST_URL=postgres://username:password@localhost/development
DISCORD_TOKEN=your_discord_bot_token
STEAM_API_KEY=your_steam_api_key
STEAM_ID=your_test_steam_id  # (for testing purposes)

For Shuttle secrets (if deploying):

shuttle secrets add DATABASE_URL "postgres://bot:password@localhost/users"
shuttle secrets add DISCORD_TOKEN "your_discord_bot_token"
shuttle secrets add STEAM_API_KEY "your_steam_api_key"

4️⃣ Run Database Migrations

cargo sqlx migrate run

For test database:

cargo sqlx migrate run --database-url "postgres://username:password@localhost/development"

5️⃣ Build & Run the Bot

cargo run

For Shuttle deployment:

shuttle deploy

🤖 Usage
✅ Bot Commands
Command	Description
!link_steam <steam_id>	Links a user's Discord account to their Steam ID.
!steam_games	Retrieves the user's stored game data.
!recommend (WIP)	Provides an LLM-based game recommendation.
🔧 Development
🧪 Running Tests

Run tests using:

cargo test -- --nocapture

🏗 Project Structure

game-recommender/
├── src/
│   ├── main.rs          # Discord bot logic
│   ├── database/        # Database functions
│   │   ├── db.rs
│   ├── steam.rs         # Steam API interactions
│   ├── llm.rs           # (TODO) Game recommendation logic
│
├── tests/               # Integration tests
│   ├── steam_tests.rs
│
├── migrations/          # SQL migrations
├── Secrets.toml         # (Shuttle secrets)
├── .env                 # Local environment variables
├── Cargo.toml           # Rust dependencies
├── README.md            # You're here!

🚀 Roadmap
✅ Current Features

    Steam account linking
    Fetch & store game data
    Basic database integration

🔜 Upcoming Features

    LLM-powered recommendations (RAG-based)
    Game genre preferences
    Trending game suggestions
    More detailed user analytics

🛠️ Contributing

Want to contribute? Feel free to fork this repo and submit a PR! 🚀
📜 License

This project is licensed under the MIT License.


---
