use std::sync::Arc;

use teloxide::{
    dispatching::dialogue::GetChatId,
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{CallbackQuery, Message, ReplyMarkup},
    Bot,
};
use tracing::instrument;

use crate::{
    database::connection::RetreiveQuiz,
    keyboard::{action_keyboard, answers_keyboard, yes_no_keyboard},
    quiz::Quiz,
    state::QuizState,
    HandlerResult, UserDialogue,
};

#[instrument(level = "info", skip(connection))]
pub(crate) async fn selection<Retreiver: RetreiveQuiz>(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    connection: Arc<Retreiver>,
) -> HandlerResult {
    match msg.text() {
        Some(quiz_name) => match connection.retreive_quiz(quiz_name).await {
            Ok(res) => match res {
                Some(quiz) => {
                    log::info!(
                        "{} selected '{}'",
                        msg.chat.username().unwrap(),
                        quiz.title()
                    );

                    dialogue
                        .update(QuizState::ReadyToRun {
                            quiz: quiz.clone(),
                            curr_idx: 0,
                        })
                        .await?;
                    bot.send_message(msg.chat.id, format!("Title{}\nDescription{}\nBy {}.\nQuestions: {}\n Are you ready to begin? (Yes/No)", quiz.title(), quiz.description(), quiz.author(), quiz.questions().len())).reply_markup(yes_no_keyboard()).await?;
                }
                None => {
                    log::info!(
                        "{} failed to retreive quiz '{}': not found",
                        msg.chat.username().unwrap(),
                        quiz_name
                    );
                    bot.send_message(
                        msg.chat.id,
                        format!("Quiz with name '{}' not found.", quiz_name),
                    ).await?;
                }
            },
            Err(e) => {
                log::error!("Database error: {:?}", e);
                return Err(e);
            }
        },
        None => {
            bot.send_message(msg.chat.id, "Failed to retreive quiz: no input provided")
                .await?;
        }
    };
    Ok(())
}

#[instrument(level = "info")]
pub(crate) async fn running_ready(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    (quiz, mut curr_idx): (Quiz, usize),
) -> HandlerResult {
    match msg.text() {
        Some("Yes") | Some("Yes✔️") => {
            if quiz.questions().len() < 1 {
                bot.send_message(msg.chat.id, "Sorry, no questions for that quiz available.").reply_markup(action_keyboard(msg.chat.username().unwrap_or_default())).await?;
                dialogue.update(QuizState::Start).await?;
                return Ok(());
            }
            let mut curr_question = &quiz.questions()[curr_idx];
            log::info!(
                "{}: asking question #{}: '{}'",
                msg.chat.username().unwrap(),
                curr_idx + 1,
                curr_question.text()
            );
            bot.send_message(msg.chat.id, "Let's begin!")
                .reply_markup(ReplyMarkup::kb_remove())
                .await?;

            let mut answers_keyboard_markup = answers_keyboard(curr_question.answers());

            while answers_keyboard_markup.inline_keyboard.len() < 1 {
                bot.send_message(msg.chat.id, "Sorry, it seems that current answer doesn't have answers. Skipping...").await?;
                curr_idx += 1;
                
                if curr_idx >= quiz.questions().len() {
                    bot.send_message(msg.chat.id, "Oh, it was the only question. Quitting quiz...").reply_markup(action_keyboard(msg.chat.username().unwrap_or_default())).await?;
                    dialogue.update(QuizState::Start).await?;
                    return Ok(());
                }
                curr_question = &quiz.questions()[curr_idx];
                answers_keyboard_markup = answers_keyboard(curr_question.answers());
            }

            bot.send_message(
                msg.chat.id,
                format!("Question #{}\n{}", curr_idx + 1, curr_question.text()),
            )
            .reply_markup(answers_keyboard_markup)
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;
            dialogue
                .update(QuizState::Running {
                    quiz,
                    curr_idx,
                    score: 0,
                })
                .await?;
        }
        Some("No") | Some("No❌") => {
            log::info!(
                "{} quits quiz '{}'",
                msg.chat.username().unwrap(),
                &quiz.title()
            );
            bot.send_message(msg.chat.id, "OK. Quitting quiz...")
                .await?;
            dialogue.update(QuizState::Start).await?;
            bot.send_message(msg.chat.id, "What do you want to do now?")
                .reply_markup(action_keyboard(msg.chat.username().unwrap()))
                .await?;
        }
        _ => {
            bot.send_message(
                msg.chat.id,
                "Please, enter a valid answer <b>Yes</b> or <b>No</b>.",
            )
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;
        }
    }

    Ok(())
}

#[instrument(level = "info")] 
pub(crate) async fn take_answer(
    bot: Bot,
    dialogue: UserDialogue,
    q: CallbackQuery,
    (quiz, mut curr_idx, mut score): (Quiz, usize, u32),
) -> HandlerResult {
    if let Some(answer_str) = &q.data {
        let answer_data = quiz.questions()[curr_idx]
            .answers()
            .iter()
            .find(|answer| {
                log::info!("{}", answer.text());
                answer.text() == answer_str.clone()
            })
            .unwrap();
        log::info!(
            "{} answers {} to question '{}' of quiz '{}'. Correctness: {}",
            q.clone().from.username.unwrap(),
            answer_str,
            quiz.questions()[curr_idx].text(),
            quiz.title(),
            answer_data.is_correct()
        );
        let text = if answer_data.is_correct() {
            score += 1;
            format!("Given answer {}. Answer is correct.✅", answer_str)
        } else {
            format!("Given answer {}. Answer is incorrect.❌", answer_str)
        };

        bot.answer_callback_query(&q.id).await?;

        let chat_id = q.chat_id().unwrap();

        if let Some(message) = &q.message {
            bot.edit_message_text(
                chat_id,
                message.id(),
                format!(
                    "{}\n{}",
                    message.regular_message().unwrap().text().unwrap(),
                    text
                ),
            )
            .await?;
        }

        if curr_idx + 1 >= quiz.questions().len() {
            log::info!(
                "{} completed a quiz '{}' with result {}%",
                q.clone().from.username.unwrap(),
                quiz.title(),
                score as f32 / quiz.questions().len() as f32
            );
            bot.send_message(
                q.chat_id().unwrap(),
                "Congratulations! You completed the quiz!",
            )
            .await?;
            bot.send_message(
                q.chat_id().unwrap(),
                format!("Your result is {}/{}", score, quiz.questions().len()),
            )
            .await?;
            dialogue.update(QuizState::Start).await?;
            bot.send_message(q.chat_id().unwrap(), "What do you want to do now?")
                .reply_markup(action_keyboard(q.from.username.unwrap()))
                .await?;
        } else {
            let mut curr_question = &quiz.questions()[curr_idx + 1];
            log::info!(
                "{}: asking question #{}: '{}'",
                q.from.username.clone().unwrap_or_default(),
                curr_idx + 1,
                curr_question.text()
            );
            let mut answers_keyboard_markup = answers_keyboard(curr_question.answers());

            while answers_keyboard_markup.inline_keyboard.len() < 1 {
                bot.send_message(q.chat_id().unwrap(), "Sorry, it seems that current question doesn't have answers. Skipping...").await?;
                curr_idx += 1;

                if curr_idx + 1 >= quiz.questions().len() {
                    bot.send_message(q.chat_id().unwrap(), format!("Oh, no more questions left. Your score is {}/{}", score, quiz.questions().len())).reply_markup(action_keyboard(q.from.username.unwrap_or_default())).await?;
                    dialogue.update(QuizState::Start).await?;
                    return Ok(());
                }
                curr_question = &quiz.questions()[curr_idx + 1];
                answers_keyboard_markup = answers_keyboard(curr_question.answers());
            }

            bot.send_message(
                q.chat_id().unwrap(),
                format!("Question #{}\n{}", curr_idx + 1, curr_question.text()),
            )
            .reply_markup(answers_keyboard_markup)
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;
            dialogue
                .update(QuizState::Running {
                    quiz,
                    curr_idx: curr_idx + 1,
                    score,
                })
                .await?;
        }
    }

    Ok(())
}
