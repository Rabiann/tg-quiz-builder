use std::{error::Error, sync::Arc};

use teloxide::{
    dispatching::{
        dialogue::{self, InMemStorage},
        DpHandlerDescription, UpdateFilterExt, UpdateHandler,
    },
    dptree::{self, Handler},
    payloads::SendMessageSetters,
    prelude::{DependencyMap, Requester},
    types::{Message, ReplyMarkup, Update},
    Bot,
};
use tracing::instrument;

use crate::{
    commands::{cancel, help, start, Command},
    constructor,
    database::connection::{Connection, RetreiveQuiz},
    editor,
    keyboard::quizes_keyboard,
    runner,
    state::QuizState,
    HandlerResult, UserDialogue,
};

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
                bot.send_message(msg.chat.id, "No available quizes.")
                    .await?;
            } else {
                bot.send_message(msg.chat.id, "Please, choose available quiz:")
                    .reply_markup(quizes_keyboard(&quizes))
                    .await?;
                dialogue.update(QuizState::Selection).await?;
            }
        }
        Some("Edit an existing quiz✏️️") => {
            let quizes = connection.retreive_all_quiz_names().await?;
            if quizes.len() < 1 {
                bot.send_message(msg.chat.id, "No available quizes.")
                    .await?;
            } else {
                bot.send_message(msg.chat.id, "Select a quiz.")
                    .reply_markup(quizes_keyboard(&quizes))
                    .await?;
                dialogue.update(QuizState::StartSelect).await?;
            }
        }
        other => {
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
    Update::filter_message()
        .branch(
            case![QuizState::ReceiveQuizName]
                .endpoint(constructor::receive_quiz_description::<Connection>),
        )
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
fn editor_scheme() -> Handler<
    'static,
    DependencyMap,
    Result<(), Box<(dyn Error + Send + Sync + 'static)>>,
    DpHandlerDescription,
> {
    use dptree::case;
    Update::filter_message()
        .branch(case![QuizState::StartSelect].endpoint(editor::select_quiz::<Connection>))
        .branch(
            case![QuizState::HandleQuiz { quiz_name }].endpoint(editor::handle_quiz::<Connection>),
        )
        .branch(case![QuizState::EditName { quiz_name }].endpoint(editor::edit_name::<Connection>))
        .branch(
            case![QuizState::EditDescription { quiz_name }]
                .endpoint(editor::edit_description::<Connection>),
        )
        .branch(
            case![QuizState::SelectQuestion { quiz_name }]
                .endpoint(editor::select_question::<Connection>),
        )
        .branch(
            case![QuizState::HandleQuestion {
                quiz_name,
                question_name
            }]
            .endpoint(editor::handle_question::<Connection>),
        )
        .branch(
            case![QuizState::EditQuestionText {
                quiz_name,
                question_name
            }]
            .endpoint(editor::edit_question_text::<Connection>),
        )
        .branch(
            case![QuizState::SelectAnswer {
                quiz_name,
                question_name
            }]
            .endpoint(editor::select_answer::<Connection>),
        )
        .branch(
            case![QuizState::EditAnswerText {
                quiz_name,
                question_name,
                answer_name
            }]
            .endpoint(editor::edit_answer_text::<Connection>),
        )
        .branch(
            case![QuizState::EditCorectness {
                quiz_name,
                question_name,
                answer_name
            }]
            .endpoint(editor::edit_corectness::<Connection>),
        )
        .branch(
            case![QuizState::HandleAnswer {
                quiz_name,
                question_name,
                answer_name
            }]
            .endpoint(editor::handle_answer::<Connection>),
        )
        .branch(
            case![QuizState::AddAnswer {
                quiz_name,
                question_name
            }]
            .endpoint(editor::editor_add_answer),
        )
        .branch(
            case![QuizState::AddAnswerCorrectness {
                quiz_name,
                question_name,
                text
            }]
            .endpoint(editor::editor_add_corectness::<Connection>),
        )
        .branch(
            case![QuizState::AddQuestion { quiz_name }]
                .endpoint(editor::editor_add_question::<Connection>),
        )
}

#[instrument(level = "info")]
async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Unable to handle the message. Enter /help to see usages.",
    )
    .await?;
    Ok(())
}
