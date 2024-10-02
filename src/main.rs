use commands::{cancel, help, start};
use database::connection::{Connection, RetreiveQuiz};
use keyboard::quizes_keyboard;
use log::Level;
use state::QuizState;
use teloxide::error_handlers::IgnoringErrorHandlerSafe;
use teloxide::update_listeners::webhooks::{self, Options};
use tracing::{instrument, level_filters};
use tracing_subscriber::fmt::format::FmtSpan;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use teloxide::types::ReplyMarkup;
use dotenvy::dotenv;
use teloxide::dispatching::dialogue::{self, InMemStorage};
use teloxide::dispatching::{DpHandlerDescription, UpdateHandler};
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use url::Url;

pub mod constructor;
pub mod database;
pub mod keyboard;
pub mod runner;
pub mod state;
pub mod editor;
pub mod commands;

use database::quiz;

#[derive(Debug, Clone, BotCommands)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "display help.")]
    Help,
    #[command(description = "start bot")]
    Cancel,
    #[command(description = "start the bot")]
    Start,
    #[command(description = "retutning back(only works in editor)")]
    Back
}

type UserDialogue = Dialogue<QuizState, InMemStorage<QuizState>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;

#[tokio::main]
async fn main() {
    dotenv().ok();
    // pretty_env_logger::init();
    let rust_log = std::env::var("LOG_LEVEL").unwrap_or("error".into());
    println!("{rust_log}");
    tracing_subscriber::fmt().with_max_level(level_filters::LevelFilter::from_level(rust_log.parse().unwrap())).json().with_span_events(FmtSpan::ENTER).log_internal_errors(true).with_ansi(true).with_line_number(true).with_target(false).init();

    let connection_string = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set.");
    let connection =
        Arc::new(Connection::connect(std::borrow::Cow::Owned(connection_string)).await);

    connection.perform_connection_if_needed().await;

    let teloxide_token = std::env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN should be set.");
    let bot = Bot::new(teloxide_token);
    log::info!("Starting bot...");

    let ngrok_url = std::env::var("NGROK_URL").map(|d| d.parse::<Url>().unwrap()).ok();
    let ngrok_addr = std::env::var("NGROK_ADDR").map(|d|d.parse::<SocketAddr>().expect("NGROK_ADDR can't be parsed.")).ok();
 
    let mut dispatcher = Dispatcher::builder(bot.clone(), schema())
        .dependencies(dptree::deps![InMemStorage::<QuizState>::new(), connection])
        .enable_ctrlc_handler()
        .build();

    if let (Some(ngrok_url), Some(ngrok_addr)) = (ngrok_url, ngrok_addr) {
        let listener = webhooks::axum(bot, Options::new(ngrok_addr, ngrok_url)).await.expect("Failed to build a listener.");
        dispatcher.dispatch_with_listener(listener, Arc::new(IgnoringErrorHandlerSafe)).await
    } else {
        dispatcher.dispatch()
        .await
    }
   

}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Help].endpoint(help))
        .branch(case![Command::Start].endpoint(start))
        .branch(case![Command::Cancel].endpoint(cancel));
        // .branch(case![Command::Back].endpoint(back));

    let handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![QuizState::Start].endpoint(choose_what_to_do::<Connection>))
        .branch(constructor_scheme())
        .branch(running_scheme())
        .branch(editor_scheme())
        .endpoint(invalid_state);

    dialogue::enter::<Update, InMemStorage<QuizState>, QuizState, _>()
        .branch(handler)
        .branch(callback_query_scheme())
}

async fn choose_what_to_do<QuizRetriever: RetreiveQuiz>(
    bot: Bot,
    msg: Message,
    dialogue: UserDialogue,
    connection: Arc<QuizRetriever>,
) -> HandlerResult {
    match msg.text() {
        Some("Create a new quiz🏗️") => {
            log::info!(
                "{} chooses to create a new quiz.",
                msg.chat.username().unwrap()
            );
                bot.send_message(
                msg.chat.id,
                "Let's start creating a new quiz! What's its title?",
                )
                .reply_markup(ReplyMarkup::kb_remove())
                .await?;
                dialogue.update(QuizState::ReceiveQuizName).await?;     
        }
        Some("Take a quiz📝") => {
            let quizes = connection.retreive_all_quiz_names().await?;
            if quizes.len() < 1 {
                bot.send_message(msg.chat.id, "No available quizes.").await?;
            } else {
                log::info!("{} chooses to take a quiz.", msg.chat.username().unwrap());
                bot.send_message(msg.chat.id, "Please, choose available quiz:")
                .reply_markup(quizes_keyboard(&quizes))
                .await?;
                dialogue.update(QuizState::Selection).await?;
            }
        }
        Some("Edit an existing quiz✏️️") => {
            let quizes = connection.retreive_all_quiz_names().await?;
            if quizes.len() < 1 {
                bot.send_message(msg.chat.id, "No available quizes.").await?;
            } else {
                log::info!("{} chooses to edit an existing quiz", msg.chat.username().unwrap());
                bot.send_message(msg.chat.id, "Select a quiz.").reply_markup(quizes_keyboard(&quizes)).await?;
                dialogue.update(QuizState::StartSelect).await?;
            }
        }
        other => {
            log::error!(
                "Invalid message {:?} from {}",
                other,
                msg.chat.username().unwrap()
            );
            bot.send_message(msg.chat.id, "Invalid input. Please try again.")
                .await?;
        }
    }

    Ok(())
}

#[instrument(level = "debug")]
fn constructor_scheme() -> Handler<
    'static,
    DependencyMap,
    Result<(), Box<(dyn Error + Send + Sync + 'static)>>,
    DpHandlerDescription,
> {
    use dptree::case;
    log::debug!("Building a dispatch tree for consturctor");
    Update::filter_message()
        .branch(case![QuizState::ReceiveQuizName].endpoint(constructor::receive_quiz_description::<Connection>))
        .branch(
            case![QuizState::ReceiveQuizDescription { quiz_name }]
                .endpoint(constructor::receive_quiz_author),
        )
        .branch(
            case![QuizState::ReceiveQuizAuthor { quiz_info }]
                .endpoint(constructor::receive_new_question::<Connection>),
        )
        .branch(
            case![QuizState::ReceiveNewQuestion { quiz_info }]
                .endpoint(constructor::receive_new_answer),
        )
        .branch(
            case![QuizState::ReceiveNewAnswer {
                quiz_info,
                new_question,
                answers
            }]
            .endpoint(constructor::receive_answer_is_correct),
        )
        .branch(
            case![QuizState::ReceiveAnswerIsCorrect {
                quiz_info,
                new_question,
                answers,
                new_answer
            }]
            .endpoint(constructor::receive_add_another_answer_or_question),
        )
        .branch(
            case![QuizState::ReceiveAddAnotherAnswer {
                quiz_info,
                new_question,
                answers
            }]
            .endpoint(constructor::receive_add_new_answer),
        )
        .branch(
            case![QuizState::ReceiveAddAnotherQuestion { quiz_info }]
                .endpoint(constructor::receive_new_question::<Connection>),
        )
}

#[instrument(level = "debug")]
fn running_scheme() -> teloxide::prelude::Handler<
    'static,
    teloxide::prelude::DependencyMap,
    Result<(), Box<(dyn Error + Send + Sync + 'static)>>,
    DpHandlerDescription,
> {
    use dptree::case;
    log::debug!("Building despatching tree for runner");
    Update::filter_message()
        .branch(case![QuizState::Selection].endpoint(runner::selection::<Connection>))
        .branch(case![QuizState::ReadyToRun { quiz, curr_idx }].endpoint(runner::running_ready))
}

#[instrument(level = "debug")]
fn callback_query_scheme() -> teloxide::prelude::Handler<
    'static,
    teloxide::prelude::DependencyMap,
    Result<(), Box<(dyn Error + Send + Sync + 'static)>>,
    DpHandlerDescription,
> {
    use dptree::case;

    log::debug!("Buildig a dispatching tree for callback query");
    Update::filter_callback_query().branch(
        case![QuizState::Running {
            quiz,
            curr_idx,
            score
        }]
        .endpoint(runner::take_answer),
    )
}

#[instrument(level = "debug")]
fn editor_scheme() -> Handler<'static, DependencyMap, Result<(), Box<(dyn Error + Send + Sync + 'static)>>, DpHandlerDescription>  {
    use dptree::case;
    log::debug!("Building dispatching tree for editor");
    Update::filter_message()
        .branch(case![QuizState::StartSelect].endpoint(editor::select_quiz::<Connection>))
        .branch(case![QuizState::HandleQuiz { quiz_name }].endpoint(editor::handle_quiz::<Connection>))
        .branch(case![QuizState::EditName { quiz_name }].endpoint(editor::edit_name::<Connection>))
        .branch(case![QuizState::EditDescription { quiz_name }].endpoint(editor::edit_description::<Connection>))
        .branch(case![QuizState::SelectQuestion { quiz_name }].endpoint(editor::select_question::<Connection>))
        .branch(case![QuizState::HandleQuestion { quiz_name, question_name }].endpoint(editor::handle_question::<Connection>))
        .branch(case![QuizState::EditQuestionText { quiz_name, question_name }].endpoint(editor::edit_question_text::<Connection>))
        .branch(case![QuizState::SelectAnswer { quiz_name, question_name }].endpoint(editor::select_answer::<Connection>))
        .branch(case![QuizState::EditAnswerText { quiz_name, question_name, answer_name }].endpoint(editor::edit_answer_text::<Connection>))
        .branch(case![QuizState::EditCorectness { quiz_name, question_name, answer_name }].endpoint(editor::edit_corectness::<Connection>))
        .branch(case![QuizState::HandleAnswer { quiz_name, question_name, answer_name }].endpoint(editor::handle_answer::<Connection>))
        .branch(case![QuizState::AddAnswer { quiz_name, question_name }].endpoint(editor::editor_add_answer))
        .branch(case![QuizState::AddAnswerCorrectness { quiz_name, question_name, text }].endpoint(editor::editor_add_corectness::<Connection>))
        .branch(case![QuizState::AddQuestion { quiz_name }].endpoint(editor::editor_add_question::<Connection>))
        // .branch(case![])
}

#[instrument(level = "info")]
async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    log::info!(
        "{}: invalid input '{:?}'",
        msg.chat.username().unwrap(),
        msg.text()
    );
    bot.send_message(
        msg.chat.id,
        "Unable to handle the message. Enter /help to see usages.",
    )
    .await?;
    Ok(())
}