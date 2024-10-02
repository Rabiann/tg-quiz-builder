# Telegram Test Bot

This is my capstone project for Summer Rustcamp 2024. Telegram bot allows users to take tests, create new tests, and edit existing ones. It provides an interactive way for both test administrators and participants to engage with quizzes through a simple and user-friendly interface.

## Features

- **Take Tests**: Users can easily take quizzes and receive feedback in real-time.
- **Create Tests**: Administrators can create new tests, set questions, and define answer choices.
- **Edit Tests**: Administrators can update or delete existing tests, modify questions, and change answers as needed.

## Technologies Used

- **Rust**: The core language used for building the bot logic.
- **Teloxide**: A Rust framework for building Telegram bots.
- **PostgreSQL**: Used for storing tests, questions, and user data.
- **Docker**: For easy setup and deployment of the bot environment.

## Setup locally

1. Clone the repository:
    ```bash
    git clone https://github.com/your-username/telegram-test-bot.git
    cd telegram-test-bot
    ```

2. Install dependencies:
    ```bash
    cargo build
    ```

3. Set up the environment variables in the `.env` file:
    ```bash
    TELEGRAM_BOT_TOKEN=your-telegram-bot-token
    DATABASE_URL=postgres://USERNAME:PASSWORD@HOST:5432/DB
    ADMIN_NICKNAME=your-admin-nickname
    LOG_LEVEL=info
    SQLX_OFFLINE=true
    ```

4. Start the bot:
    ```bash
    cargo run --release
    ```

## Setup using docker compose

1. Build `compose.yml`
   ```bash
   docker compose -f compose.yml build
   ```

2. Run `compose.yml`
   ```bash
   docker compose -f compose.yml up
   ```
## Usage

- **Test Takers**: Start a chat with the bot and follow the prompts to take a test.
- **Test Creators/Editors**: Create or edit tests. Only administrators with the correct `ADMIN_NAME` can access these features.

## Commands

- `/start` - Start interacting with the bot.
- `/back` - Move to previous section (works in editor only)
- `/help` - Get help on using the bot.

## Future Improvements
- Adding non-question sections(plain text, images, media)
- Support for HTML/Markdown formats
- Adding open questions
- Quizes' results analytics  
