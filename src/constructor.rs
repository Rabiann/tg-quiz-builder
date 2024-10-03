use std::sync::Arc;
use teloxide::types::ReplyMarkup;
use teloxide::{payloads::SendMessageSetters, prelude::Requester, types::Message, Bot};
use tracing::instrument;
use crate::database::connection::{CreateQuiz, RetreiveQuiz};
use crate::database::quiz::{Answer, Question, Quiz};
use crate::keyboard::{action_keyboard, yes_no_keyboard};
use crate::state::{QuizData, QuizState};
use crate::{HandlerResult, UserDialogue};

#[instrument(level = "info", skip(connection, bot, dialogue))]
pub(crate) async fn receive_quiz_description<DbConnection: RetreiveQuiz>(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    connection: Arc<DbConnection>,
) -> HandlerResult {
    match msg.text() {
        Some(title) => {
            log::info!(
                "{} receives quiz name {}",
                msg.chat.username().unwrap(),
                title
            );

            if let Ok(Some(_)) = connection.retreive_quiz(title).await {
                bot.send_message(msg.chat.id, "Quiz with already exists. Try again.").await?;
            } else {
                bot.send_message(msg.chat.id, "OK. What is new quiz about?")
                    .await?;
                dialogue
                    .update(QuizState::ReceiveQuizDescription {
                        quiz_name: title.to_owned(),
                    })
                    .await?;
                }
       }
        None => {
            bot.send_message(msg.chat.id, "Please, send a title of the new quiz.")
                .await?;
        }
    }

    Ok(())
}
#[instrument(level = "info", skip(dialogue, bot))]
pub(crate) async fn receive_quiz_author(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    title: String,
) -> HandlerResult {
    match msg.text() {
        Some(description) => {
            log::info!(
                "{} receives quiz description {} for {} quiz",
                msg.chat.username().unwrap(),
                description,
                &title
            );
            bot.send_message(
                msg.chat.id,
                "Do you want to add the first question?(Yes/No)",
            )
            .reply_markup(yes_no_keyboard())
            .await?;
            dialogue
                .update(QuizState::ReceiveQuizAuthor {
                    quiz_info: QuizData {
                        quiz_name: title,
                        description: description.to_owned(),
                        author: msg.chat.username().unwrap().to_owned(),
                        questions: Vec::default(),
                    },
                })
                .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send a description of the new quiz.")
                .await?;
        }
    }

    Ok(())
}

#[instrument(level = "info", skip(connection, bot, dialogue))]
pub(crate) async fn receive_new_question<DbConnection: CreateQuiz>(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    quiz_info: QuizData,
    connection: Arc<DbConnection>,
) -> HandlerResult {
    match msg.text() {
        Some("Yes") | Some("Yes✔️") => {
            bot.send_message(msg.chat.id, "Great. Please enter a question.")
                .reply_markup(ReplyMarkup::kb_remove())
                .await?;
            dialogue
                .update(QuizState::ReceiveNewQuestion { quiz_info })
                .await?;
        }
        Some("No") | Some("No❌") => {
            log::info!(
                "{} refuses to add a new question and saves {} quiz.",
                msg.chat.username().unwrap(),
                &quiz_info.quiz_name
            );
            let new_quiz = Quiz::new(
                quiz_info.quiz_name,
                quiz_info.description,
                quiz_info.author,
                Some(quiz_info.questions),
            );

            let quiz_name = connection.create_quiz(new_quiz).await?;
            bot.send_message(
                msg.chat.id,
                format!(
                    "OK. Saving quiz {}. What do you want to do next?",
                    quiz_name
                ),
            )
            .reply_markup(action_keyboard(msg.chat.username().unwrap()))
            .await?;

            dialogue.exit().await?;
        }
        _ => {
            bot.send_message(
                msg.chat.id,
                "Please enter a valid answer <b>Yes</b> or <b>No</b>",
            )
            .reply_markup(yes_no_keyboard())
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        }
    }

    Ok(())
}

#[instrument(level = "info", skip(bot, dialogue))]
pub(crate) async fn receive_new_answer(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    quiz_info: QuizData,
) -> HandlerResult {
    match msg.text() {
        Some(new_question) => {
            log::info!(
                "{} adds a new question {}",
                msg.chat.username().unwrap(),
                new_question
            );
            bot.send_message(msg.chat.id, "OK. What's the answer to your question?")
                .await?;
            dialogue
                .update(QuizState::ReceiveNewAnswer {
                    quiz_info,
                    new_question: new_question.to_owned(),
                    answers: Vec::default(),
                })
                .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please send valid answer.")
                .await?;
        }
    }

    Ok(())
}

#[instrument(level = "info", skip(bot, dialogue))]
pub(crate) async fn receive_answer_is_correct(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    (quiz_info, new_question, answers): (QuizData, String, Vec<Answer>),
) -> HandlerResult {
    match msg.text() {
        Some(answer) => {
            log::info!(
                "{} adds answer '{}' to question '{}' in quiz '{}'",
                msg.chat.username().unwrap(),
                answer,
                &new_question,
                &quiz_info.quiz_name
            );
            bot.send_message(msg.chat.id, "Got it. Is that answer correct?(Yes/No)")
                .reply_markup(yes_no_keyboard())
                .await?;
            dialogue
                .update(QuizState::ReceiveAnswerIsCorrect {
                    quiz_info,
                    new_question,
                    answers,
                    new_answer: answer.to_owned(),
                })
                .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please, enter a valid answer.")
                .await?;
        }
    }

    Ok(())
}

#[instrument(level = "info", skip(bot, dialogue))]
pub(crate) async fn receive_add_another_answer_or_question(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    (quiz_info, new_question, mut answers, new_answer): (QuizData, String, Vec<Answer>, String),
) -> HandlerResult {
    println!("{}", new_answer);
    match msg.text() {
        Some("Yes") | Some("Yes✔️") => {
            log::info!(
                "Answer '{}' in question '{}' is correct",
                &new_answer,
                &new_question
            );
            bot.send_message(
                msg.chat.id,
                "Okay, that answer is correct. Do you want to add a new answer?(Yes/No)",
            )
            .await?;
            //
            //  Logic to save answer
            //
            let added_answer = Answer::new(new_answer, true);
            answers.push(added_answer);
            dialogue
                .update(QuizState::ReceiveAddAnotherAnswer {
                    quiz_info,
                    new_question,
                    answers,
                })
                .await?;
        }
        Some("No") | Some("No❌") => {
            log::info!(
                "Answer '{}' in question '{}' is incorrect",
                &new_answer,
                &new_question
            );
            bot.send_message(
                msg.chat.id,
                "Okay, that answer is incorrect. Do you want to create a new answer?(Yes/No)",
            )
            .await?;
            //
            //  Logic to save answer
            //
            let added_answer = Answer::new(new_answer, false);
            answers.push(added_answer);
            dialogue
                .update(QuizState::ReceiveAddAnotherAnswer {
                    quiz_info,
                    new_question,
                    answers,
                })
                .await?;
        }
        _ => {
            bot.send_message(
                msg.chat.id,
                "Please, enter a valid answer <b>Yes</b> or <b>No</b>.",
            )
            .await?;
        }
    }

    Ok(())
}

#[instrument(level = "info", skip(bot, dialogue))]
pub(crate) async fn receive_add_new_answer(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    (mut quiz_info, new_question, answers): (QuizData, String, Vec<Answer>),
) -> HandlerResult {
    match msg.text() {
        Some("Yes") | Some("Yes✔️") => {
            log::info!(
                "{} adds other answer to question '{}' in quiz '{}'",
                msg.chat.username().unwrap(),
                new_question,
                &quiz_info.quiz_name
            );
            bot.send_message(msg.chat.id, "Great. What's the another answer?")
                .reply_markup(ReplyMarkup::kb_remove())
                .await?;
            dialogue
                .update(QuizState::ReceiveNewAnswer {
                    quiz_info,
                    new_question,
                    answers,
                })
                .await?;
        }
        Some("No") | Some("No❌") => {
            log::info!(
                "{} saves question in quiz '{}'",
                msg.chat.username().unwrap(),
                &quiz_info.quiz_name
            );
            bot.send_message(
                msg.chat.id,
                "OK. Saving question. Do you want to add another question? (Yes/No)",
            )
            .reply_markup(yes_no_keyboard())
            .await?;
            //
            //  Logic to save question
            //
            let question_added = Question::new(new_question, Some(answers));
            quiz_info.questions.push(question_added);
            dialogue
                .update(QuizState::ReceiveQuizAuthor { quiz_info })
                .await?;
        }
        _ => {
            bot.send_message(
                msg.chat.id,
                "Pleasem enter a valid answer <b>Yes</b> or <b>No</b>",
            )
            .await?;
        }
    }

    Ok(())
}
