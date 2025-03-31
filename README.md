# 🎮 Game Recommender Discord Bot
A Rust-powered Discord bot that links to a user's Steam account, fetches their game library, and (coming soon) provides LLM-based game recommendations. Built with Serenity, SQLx, and Tokio.

---

## 🚀 Features
- 🔗 Link Steam Account – Connect your Discord identity to your Steam ID
- 📥 Fetch Owned Games – Pulls and stores your Steam game library
- 🔄 Auto-Sync – Periodically updates game data in the background
- 📊 Game Stats – View most-played games
- 🧠 LLM Game Recs (Coming Soon) – Suggests new games using AI

---

## 📦 Installation
### 1️⃣ Prerequisites

- Rust & Cargo
- Docker
- PostgreSQL client
- Discord bot token + Steam API key

---

### 2️⃣ Start PostgreSQL with Docker
#### Run the included Docker config:

```docker-compose up -d```
- db (port 5432) — main database
- test-db (port 5433) — used for integration tests

--- 

### 3️⃣ Create .env File
#### env:

- DISCORD_TOKEN=your_discord_token
- STEAM_API_KEY=your_steam_api_key
- LLM_API_KEY=your_llm_api_key
- DISCORD_CHANNEL_ID=your_channel_id
- DATABASE_URL=postgres://user:password@localhost:5432/games_db
- DATABASE_TEST_URL=postgres://user:password@localhost:5433/test_games_db

--- 

### 4️⃣ Run Database Migrations
```cargo sqlx migrate run```

#### For the test database:

```cargo sqlx migrate run --database-url $DATABASE_TEST_URL```

---

### 5️⃣ Run the Bot

```cargo run```
# 🤖 Usage
## ✅ Bot Commands

| Command                  | Description                                 |
|--------------------------|---------------------------------------------|
| `!link_steam <steam_id>` | Link your Steam account                     |
| `!steam_games`           | Show your most-played games                 |
| `!recommend` (WIP)       | Get AI-generated game recommendations       |

---

## 🧪 Running Tests

Ensure `test-db` is running on port `5433`, then run:

```cargo test -- --nocapture```

## 🗂️ Project Structure

```text
game-recommender/
├── src/
│   ├── main.rs            # App entry point
│   ├── bot/               # Discord bot logic
│   ├── database/          # DB operations
│   ├── steam.rs           # Steam API logic
│   ├── llm.rs             # LLM logic (recommendations)
├── cron/scheduler.rs      # Background scheduler
├── tests/steam_tests.rs   # Integration tests
├── migrations/            # SQLx migrations
├── docker-compose.yml     # PostgreSQL setup
├── .env                   # Local secrets
├── Cargo.toml             # Project config
├── README.md              # You're here!
```

## 🛣️ Roadmap

### ✅ Current
- Steam account linking
- Game sync and storage
- Discord bot integration

### 🔜 Coming Soon
- AI-generated recommendations (via HuggingFace or local LLM)
- Genre-based filtering
- Personalized analytics & stats
- Ability to ask for recommendations based on input (i.e. !recommend chill roguelike)

---

## 🛠 Contributing

PRs are welcome! Fork this repo, make your changes, and submit a pull request 🚀

---

## 📜 License

This project is licensed under the **MIT License**.
