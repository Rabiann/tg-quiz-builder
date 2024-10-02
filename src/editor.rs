use std::sync::Arc;

use teloxide::{payloads::SendMessageSetters, prelude::Requester, types::Message, Bot};
use tracing::instrument;

use crate::{database::{connection::{CreateAnswer, CreateQuestion, DeleteAnswer, DeleteQuestion, DeleteQuiz, EditAnswer, EditQuestion, EditQuiz, RetreiveAnswer, RetreiveQuestion, RetreiveQuiz}, quiz::Quiz}, keyboard::{self, edit_question_keyboard, yes_no_keyboard}, state::QuizState, Command, HandlerResult, UserDialogue};
#[instrument(level = "info", skip(connection))]
pub(crate) async fn edit_corectness<Connect: EditAnswer>(bot: Bot, msg: Message, dialogue: UserDialogue, (quiz_name, question_name, answer_name): (String, String, String), connection: Arc<Connect>) -> HandlerResult {
    match msg.text() {
        Some("Yes") | Some("Yes✔") => {
            log::info!("User '@{}' makes question {} correct.", msg.chat.username().unwrap(), question_name);
            connection.edit_corectness(&quiz_name, &question_name, &answer_name, true).await?;
            bot.send_message(msg.chat.id, format!("Answer {} is now correct.", &answer_name)).await?;
            dialogue.update(QuizState::HandleAnswer { quiz_name, question_name, answer_name }).await?;
        }
        Some("No") | Some("No❌") => {
            log::info!("User '@{}' makes question {} incorrect.", msg.chat.username().unwrap(), question_name);
            connection.edit_corectness(&quiz_name, &question_name, &answer_name, false).await?;
            bot.send_message(msg.chat.id, format!("Answer {} is now incorrect.", &answer_name)).await?;
            dialogue.update(QuizState::HandleAnswer { quiz_name, question_name, answer_name }).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Invalid input. Try again.").await?;
        }
    }

    Ok(())
}

#[instrument(level = "info", skip(connection))]
pub(crate) async fn edit_answer_text<Connect: EditAnswer>(bot: Bot, msg: Message, dialogue: UserDialogue, (quiz_name, question_name, answer_name): (String, String, String), connection: Arc<Connect>) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            log::info!("User '@{}' edits answer text from {} -> {}", msg.chat.username().unwrap_or("anonymous"), answer_name, text);
            let new_answer_text = connection.edit_answer_text(&quiz_name, &question_name, &answer_name, text).await?;
            bot.send_message(msg.chat.id, format!("Answer text updated: {}.", new_answer_text)).reply_markup(keyboard::edit_answer_keyboard()).await?;
            dialogue.update(QuizState::HandleAnswer { quiz_name, question_name, answer_name }).await?;
        }
        _ => {
            log::info!("Invalid input from @{}", msg.chat.username().unwrap_or("anonymous"));
            bot.send_message(msg.chat.id, "Invalid input. Try again.").await?;
        }
    }

    Ok(())
}

#[instrument(level = "info", skip(connection))]
pub(crate) async fn edit_question_text<Connect: EditQuestion>(bot: Bot, msg: Message, dialogue: UserDialogue, (quiz_name, question_name): (String, String), connection: Arc<Connect>) -> HandlerResult {
    match msg.text() {
        Some(new_text) => {
            log::info!("User '@{}' edits question text from {} -> {}", msg.chat.username().unwrap_or("anonymous"), question_name, new_text);
            let new_text = connection.edit_text(&quiz_name, &question_name, new_text).await?;
            bot.send_message(msg.chat.id, "Question name updated.").reply_markup(keyboard::edit_question_keyboard()).await?;
            dialogue.update(QuizState::HandleQuestion { quiz_name, question_name }).await?;
        }
        _ => {
            log::info!("Invalid input from @{}", msg.chat.username().unwrap_or("anonymous"));
            bot.send_message(msg.chat.id, "Invalid input. Please try again.").await?;
        }
    }

    Ok(())
}

#[instrument(level = "info", skip(connection))]
pub(crate) async fn handle_answer<Connect: DeleteAnswer + RetreiveQuestion>(bot: Bot, msg: Message, dialogue: UserDialogue, (quiz_name, question_name, answer_name): (String, String, String), connection: Arc<Connect>) -> HandlerResult {
    match msg.text() {
        Some("/back") => {
            let question = connection.retreive_question(&quiz_name, &question_name).await?.unwrap();
            bot.send_message(msg.chat.id, "Returning back.").await?;
            bot.send_message(msg.chat.id, question.to_string()).reply_markup(keyboard::edit_question_keyboard()).await?;
            dialogue.update(QuizState::HandleQuestion { quiz_name, question_name }).await?;
        }
        Some("Delete answer") => {
            let deleted = connection.delete_answer(&quiz_name, &question_name, &answer_name).await?;
            bot.send_message(msg.chat.id, "Answer deleted.").reply_markup(keyboard::edit_question_keyboard()).await?;
            dialogue.update(QuizState::HandleQuestion {  quiz_name, question_name }).await?;
        }
        Some("Edit text") => {
            bot.send_message(msg.chat.id, "What's new answer text?").await?;
            dialogue.update(QuizState::EditAnswerText { quiz_name, question_name, answer_name }).await?;
        }
        Some("Edit corectness") => {
            bot.send_message(msg.chat.id, "Is that answer correct?").reply_markup(yes_no_keyboard()).await?;
            dialogue.update(QuizState::EditCorectness { quiz_name, question_name, answer_name }).await?;
        }
        Some("Back") => {
            bot.send_message(msg.chat.id, "Handle question.").reply_markup(keyboard::edit_question_keyboard()).await?;
            dialogue.update(QuizState::HandleQuestion { quiz_name, question_name }).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Invalid input. Please try again.").await?;
        }
    }

    Ok(())
}

#[instrument(level = "info", skip(connection))]
pub(crate) async fn select_answer<Connect: RetreiveAnswer>(bot: Bot, msg: Message, dialogue: UserDialogue, (quiz_name, question_name): (String, String), connection: Arc<Connect>) -> HandlerResult {
    match msg.text() {
        Some(answer) => {
            match connection.retreive_answer(&quiz_name, &question_name, answer).await {
                Ok(Some(ans)) => {
                    bot.send_message(msg.chat.id, format!("Answer '{}' selected. What do you want to do next?", ans.text())).reply_markup(keyboard::edit_answer_keyboard()).await?;
                    bot.send_message(msg.chat.id, ans.to_string()).await?;
                    dialogue.update(QuizState::HandleAnswer { quiz_name, question_name, answer_name: answer.to_owned() }).await?;
                }
                Ok(None) => {
                    bot.send_message(msg.chat.id, "Answer not found.").await?;
                }

                Err(_) => {
                    bot.send_message(msg.chat.id, "Some error occured. Please try again.").await?;
                }
            }
        }
        None => {
            bot.send_message(msg.chat.id, "Invalid input. Please try again.").await?;
        }
    }

    Ok(())
}

#[instrument(level = "info", skip(connection))]
pub(crate) async fn handle_question<Connect: DeleteQuestion + RetreiveAnswer + RetreiveQuiz>(bot: Bot, msg: Message, dialogue: UserDialogue, (quiz_name, question_name): (String, String), connection: Arc<Connect>) -> HandlerResult {
    match msg.text() {
        Some("/back") => {
            let quiz = connection.retreive_quiz(&quiz_name).await?.unwrap();
            bot.send_message(msg.chat.id, "Returning back.").await?;
            bot.send_message(msg.chat.id, quiz.to_string()).reply_markup(keyboard::edit_quiz_keyboard()).await?;
            dialogue.update(QuizState::HandleQuiz { quiz_name }).await?;
        }
        Some("Delete question") => {
            let deleted = connection.delete_question(&quiz_name, &question_name).await?;
            bot.send_message(msg.chat.id, "Question deleted.").reply_markup(keyboard::edit_quiz_keyboard()).await?;
            dialogue.update(QuizState::HandleQuiz { quiz_name }).await?;
        }
        Some("Edit text") => {
            bot.send_message(msg.chat.id, "What's new question text?").await?;
            dialogue.update(QuizState::EditQuestionText { quiz_name, question_name }).await?;
        }
        Some("Add answer") => {
            bot.send_message(msg.chat.id, "What a new answer looks like?").await?;
            dialogue.update(QuizState::AddAnswer { quiz_name, question_name }).await?;
        }
        Some("Edit answer") => {
            let answers = connection.retreive_all_answers_names(&quiz_name, &question_name).await?;
            bot.send_message(msg.chat.id, "Choose answer to edit:").reply_markup(keyboard::answers_block_keyboard(&answers)).await?;
            dialogue.update(QuizState::SelectAnswer { quiz_name, question_name }).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Invalid input. Please try again.").await?;
        }
    }

    Ok(())
} 

#[instrument(level = "info")] 
pub(crate) async fn editor_add_answer(bot: Bot, msg: Message, dialogue: UserDialogue, (quiz_name, question_name): (String, String)) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            bot.send_message(msg.chat.id, "Is that answer correct?").reply_markup(keyboard::yes_no_keyboard()).await?;
            dialogue.update(QuizState::AddAnswerCorrectness { quiz_name, question_name, text: text.to_owned() }).await?;
        }
        _ => { bot.send_message(msg.chat.id, "Invalid input. Please, try again.").await?; }
    }

    Ok(())
}

#[instrument(level = "info", skip(connection))]
pub(crate) async fn editor_add_corectness<Connect: CreateAnswer>(bot: Bot, msg: Message, dialogue: UserDialogue, (quiz_name, question_name, answer_name): (String, String, String), connection: Arc<Connect>) -> HandlerResult {
    match msg.text() {
        Some("Yes") | Some("Yes✔️") => {
            match connection.create_answer(&quiz_name, &question_name, &answer_name, true).await {
                Ok(added) => {
                    bot.send_message(msg.chat.id, format!("Answer {} saved. It is correct.", &answer_name)).reply_markup(edit_question_keyboard()).await?;
                    dialogue.update(QuizState::HandleQuestion { quiz_name, question_name }).await?;
                } 
                Err(e) => {
                    bot.send_message(msg.chat.id, "Error occured. Please try again later.").await?;
                }
            }
        }
        Some("No") | Some("No❌") => {
            match connection.create_answer(&quiz_name, &question_name, &answer_name, true).await {
                Ok(added) => {
                    bot.send_message(msg.chat.id, format!("Answer {} saved. It is incorrect.", &answer_name)).reply_markup(edit_question_keyboard()).await?;
                    dialogue.update(QuizState::HandleQuestion { quiz_name, question_name }).await?;
                }
                Err(e) => {
                    bot.send_message(msg.chat.id, "Error occured. Please try again later.").await?;
                }
            }
        }
        _=> {
            bot.send_message(msg.chat.id, "Invalid input. Try again.").await?;
        }
    }

    Ok(())
}

#[instrument(level = "info", skip(connection))]
pub(crate) async fn editor_add_question<Connect: CreateQuestion>(bot: Bot, msg: Message, dialogue: UserDialogue, quiz_name: String, connection: Arc<Connect>) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            match connection.create_question(&quiz_name, text).await {
                Ok(added) => {
                    bot.send_message(msg.chat.id, format!("Question '{}' created.", text)).await?;
                    dialogue.update(QuizState::HandleQuiz { quiz_name }).await?;
                }
                Err(e) => {
                    bot.send_message(msg.chat.id, "Error occured. Please try again later.").await?;
                }
            }
        }
        None => {
            bot.send_message(msg.chat.id, "Invalid input. Please try again.").await?;
        }
    }

    Ok(())
}


#[instrument(level = "info", skip(connection))]
pub(crate) async fn select_question<Connect: RetreiveQuestion>(bot: Bot, msg: Message, dialogue: UserDialogue, quiz_name: String, connection: Arc<Connect>) -> HandlerResult {
    match msg.text() {
        Some(question_name) => {
            match connection.retreive_question(&quiz_name, question_name).await {
                Ok(Some(question)) => {
                    bot.send_message(msg.chat.id, format!("Question '{}' selected. Please select an action:", question.text())).reply_markup(keyboard::edit_question_keyboard()).await?;
                    bot.send_message(msg.chat.id, question.to_string()).await?;
                    dialogue.update(QuizState::HandleQuestion { quiz_name, question_name: question_name.to_owned() }).await?;
                },
                Ok(None) => { bot.send_message(msg.chat.id, format!("Question '{}' not found. Try again.", question_name)).await?; }
                Err(e) => log::info!("Error occured: {:?}", e),
            }
        }
        None => {
            bot.send_message(msg.chat.id, "Please, select a question.").await?;
        }
    }
    
    Ok(())
}

#[instrument(level = "info", skip(connection))]
pub(crate) async fn select_quiz<QuizRetriever: RetreiveQuiz>(bot: Bot, msg: Message, dialogue: UserDialogue, connection: Arc<QuizRetriever>) -> HandlerResult {
    match msg.text() {
        Some(quiz_name) => {
            match connection.retreive_quiz(quiz_name).await {
                Ok(Some(quiz)) => { 
                    bot.send_message(msg.chat.id, format!("Quiz '{}' chosen. Please, select an action:", quiz.title())).reply_markup(keyboard::edit_quiz_keyboard()).await?;
                    bot.send_message(msg.chat.id, quiz.to_string()).await?;
                    dialogue.update(QuizState::HandleQuiz { quiz_name: quiz_name.into() }).await?;
                },
                Ok(None) => { bot.send_message(msg.chat.id, format!("Quiz '{}' not found. Try again.", quiz_name)).await?; },
                Err(e) => log::info!("Error occured: {:?}", e),
            }   
        }
        None => {
            bot.send_message(msg.chat.id, "Please, select a quiz.").await?;
        }
    }
    log::info!("{} selects quiz to edit", msg.chat.username().unwrap());

    Ok(())
}

#[instrument(level = "info", skip(connection))]
pub(crate) async fn handle_quiz<Connect: DeleteQuiz + RetreiveQuestion>(bot: Bot, msg: Message, dialogue: UserDialogue, quiz_name: String, connection: Arc<Connect>) -> HandlerResult {
    match msg.text() {
        Some("/back") => {
            bot.send_message(msg.chat.id, "Returning back.").reply_markup(keyboard::action_keyboard(msg.chat.username().unwrap_or("anonymous"))).await?;
            dialogue.update(QuizState::Start).await?;
        }
        Some("Delete quiz🗑️") => {
            let deleted = connection.delete_quiz(&quiz_name).await?;
            bot.send_message(msg.chat.id, format!("Quiz '{}' deleted.", deleted)).reply_markup(keyboard::action_keyboard(msg.chat.username().unwrap())).await?;
            dialogue.update(QuizState::Start).await?;
        },
        Some("Edit name") => {
            bot.send_message(msg.chat.id, "What's new quiz name?").await?;
            dialogue.update(QuizState::EditName { quiz_name }).await?;
        }  
        Some("Edit description") => {
            bot.send_message(msg.chat.id, "What's new quiz description?").await?;
            dialogue.update(QuizState::EditDescription { quiz_name }).await?;
        }
        Some("Add question") => {
            bot.send_message(msg.chat.id, "Choose question text?").await?;
            dialogue.update(QuizState::AddQuestion { quiz_name }).await?;
        }
        Some("Edit question") => {
            let questions = connection.retreive_all_question_names(&quiz_name).await?;
            if questions.len() < 1 {
                bot.send_message(msg.chat.id, "No available questions.").await?;
            } else {
                bot.send_message(msg.chat.id, "Choose question to edit").reply_markup(keyboard::questions_keyboard(&questions)).await?;
            }
            dialogue.update(QuizState::SelectQuestion { quiz_name }).await?;
        } 
        _ => {
            bot.send_message(msg.chat.id, "Invalid input. Try again.").await?;
        }
    }
    Ok(())
}

#[instrument(level = "info", skip(connection))]
pub(crate) async fn edit_name<Connect: EditQuiz>(bot: Bot, msg: Message, dialogue: UserDialogue, quiz_name: String, connection: Arc<Connect>) -> HandlerResult {
    match msg.text() {
        Some(new_name) => {
            if let Ok(new_name) = connection.edit_name(&quiz_name, new_name).await {
                bot.send_message(msg.chat.id, "Quiz name succesfully updated.").await?;
                dialogue.update(QuizState::HandleQuiz { quiz_name: new_name }).await?;
            } else {
                bot.send_message(msg.chat.id, "Sorry, error occured").await?; // add better error handling
                dialogue.update(QuizState::HandleQuiz { quiz_name }).await?;
            }
       }

       None => {
            bot.send_message(msg.chat.id, "Nothing entered. Try again.").await?;
       }
    }

    Ok(())
}

#[instrument(level = "info", skip(connection))]
pub(crate) async fn edit_description<Connect: EditQuiz>(bot: Bot, msg: Message, dialogue: UserDialogue, quiz_name: String, connection: Arc<Connect>) -> HandlerResult {
    match msg.text(){
        Some(new_name) => {
            let new_descriptrion = connection.edit_description(&quiz_name, new_name).await?;
            bot.send_message(msg.chat.id, "Quiz description successfully updated.").await?;
            dialogue.update(QuizState::HandleQuiz { quiz_name }).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Nothing entered. Try again.").await?;
        }
    }

    Ok(())
}

