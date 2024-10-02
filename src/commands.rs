use teloxide::{dispatching::dialogue, payloads::SendMessageSetters, prelude::Requester, types::Message, utils::command::BotCommands, Bot};

use crate::{keyboard::action_keyboard, state::{QuizData, QuizState}, Command, HandlerResult, UserDialogue};


pub(crate) async fn help(bot: Bot, msg: Message) -> HandlerResult {
    log::info!("{} called /help", msg.chat.username().unwrap());
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

pub(crate) async fn cancel(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    log::info!("Cancelling progress {}", msg.chat.username().unwrap());
    bot.send_message(msg.chat.id, "Cancelling dialogue").await?;
    // dialogue.exit().await?;
    dialogue.update(QuizState::Start).await?;
    Ok(())
}

// pub(crate) async fn back(bot: Bot, dialogue: UserDialogue) -> HandlerResult {
//     let current_state = dialogue.get().await?.unwrap();

//     let (to_update, redirect) = match current_state {
//         QuizState::Start => (QuizState::Start, start),
//         QuizState::ReceiveQuizName => QuizState::Start,
//         // QuizState::ReceiveQuizDescription { quiz_name: _ } => QuizState::ReceiveQuizName,
//         // QuizState::ReceiveQuizAuthor { quiz_info } => QuizState::ReceiveQuizDescription { quiz_name: quiz_info.quiz_name },
//         // QuizState::ReceiveNewQuestion { quiz_info } => QuizState::ReceiveQuizAuthor { quiz_info },
//         // QuizState::ReceiveNewAnswer { quiz_info, new_question, answers } => QuizState::ReceiveNewQuestion { quiz_info },
//         // QuizState::ReceiveAnswerIsCorrect { quiz_info, new_question, answers, new_answer } => QuizState::ReceiveNewAnswer { quiz_info, new_question, answers },
//         // QuizState::ReceiveAddAnotherAnswer { quiz_info, new_question, answers } => QuizState::ReceiveAnswerIsCorrect { quiz_info, new_question, answers: answers[0..answers.len()-1].to_vec(), new_answer: answers.last().unwrap().text() },
//         // QuizState::ReceiveAddAnotherQuestion { quiz_info } => QuizState::ReceiveAddAnotherAnswer { quiz_info: QuizData { questions: quiz_info.questions[0..quiz_info.questions.len()-1].to_vec(), ..quiz_info }, new_question: quiz_info.questions.last().unwrap().text(), answers: quiz_info.questions.last().unwrap().answers().to_vec() },
//         // QuizState::Selection => QuizState::Start,
//         // QuizState::ReadyToRun { quiz, curr_idx } => QuizState::Selection,
//         // QuizState::Running { quiz, curr_idx, score } => QuizState::ReadyToRun { quiz, curr_idx },
//         // QuizState::Done { score } => QuizState::Selection,
//         // QuizState::StartSelect => QuizState::Start,
//         // QuizState::HandleQuiz { quiz_name } => QuizState::StartSelect,
//         // QuizState::HandleQuestion { quiz_name, question_name } => QuizState::HandleQuiz { quiz_name },
//         // QuizState::EditName { quiz_name } => QuizState::HandleQuiz { quiz_name },
//         // QuizState::EditDescription { quiz_name } => QuizState::HandleQuiz { quiz_name },
//         // QuizState::EditQuestionText { quiz_name, question_name } => QuizState::HandleQuestion { quiz_name, question_name },
//         // QuizState::HandleAnswer { quiz_name, question_name, answer_name } => QuizState::HandleQuestion { quiz_name, question_name },
//         // QuizState::EditAnswerText { quiz_name, question_name, answer_name } => QuizState::HandleAnswer { quiz_name, question_name, answer_name },
//         // QuizState::EditCorectness { quiz_name, question_name, answer_name } => QuizState::HandleAnswer { quiz_name, question_name, answer_name },
//         _ => unreachable!()
//     };

//     dialogue.update(to_update).await?;

//     Ok(())
// }

pub(crate) async fn start(bot: Bot, msg: Message, dialogue: UserDialogue) -> HandlerResult {
    // log::info!("Starting action. User: {}", msg.chat.username().unwrap());
    bot.send_message(msg.chat.id, "Please choose what to do:")
        .reply_markup(action_keyboard(msg.chat.username().unwrap()))
        .await?;
    dialogue.update(QuizState::Start).await?;
    Ok(())
}