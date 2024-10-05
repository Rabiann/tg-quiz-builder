use commands::{cancel, help, start};
use database::connection::{Connection, RetreiveQuiz};
use dotenvy::dotenv;
use keyboard::quizes_keyboard;
use state::QuizState;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use teloxide::dispatching::dialogue::{self, InMemStorage};
use teloxide::dispatching::{DpHandlerDescription, UpdateHandler};
use teloxide::error_handlers::IgnoringErrorHandlerSafe;
use teloxide::prelude::*;
use teloxide::types::ReplyMarkup;
use teloxide::update_listeners::webhooks::{self, Options};
use teloxide::utils::command::BotCommands;
use tracing::{instrument, level_filters};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;
use url::Url;

use database::quiz;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(EnvFilter::from_env("LOG_LEVEL"))
        .json()
        .with_span_events(FmtSpan::ENTER)
        .log_internal_errors(true)
        .with_ansi(true)
        .with_line_number(true)
        .with_target(false)
        .init();

    let connection_string = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set.");
    let connection =
        Arc::new(Connection::connect(std::borrow::Cow::Owned(connection_string)).await);

    connection.perform_connection_if_needed().await;

    let teloxide_token = std::env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN should be set.");
    let bot = Bot::new(teloxide_token);

    let ngrok_url = std::env::var("NGROK_URL")
        .map(|d| d.parse::<Url>().unwrap())
        .ok();
    let ngrok_addr = std::env::var("NGROK_ADDR")
        .map(|d| {
            d.parse::<SocketAddr>()
                .expect("NGROK_ADDR can't be parsed.")
        })
        .ok();

    let mut dispatcher = Dispatcher::builder(bot.clone(), schema())
        .dependencies(dptree::deps![InMemStorage::<QuizState>::new(), connection])
        .enable_ctrlc_handler()
        .build();

    if let (Some(ngrok_url), Some(ngrok_addr)) = (ngrok_url, ngrok_addr) {
        let listener = webhooks::axum(bot, Options::new(ngrok_addr, ngrok_url))
            .await
            .expect("Failed to build a listener.");
        dispatcher
            .dispatch_with_listener(listener, Arc::new(IgnoringErrorHandlerSafe))
            .await
    } else {
        dispatcher.dispatch().await
    }
}
