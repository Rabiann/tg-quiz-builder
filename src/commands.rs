use teloxide::{
    payloads::SendMessageSetters, prelude::Requester, types::Message, utils::command::BotCommands,
    Bot,
};

use crate::{keyboard::action_keyboard, state::QuizState, HandlerResult, UserDialogue};

#[derive(Debug, Clone, BotCommands)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "display help.")]
    Help,
    #[command(description = "start bot")]
    Cancel,
    #[command(description = "start the bot")]
    Start,
    #[command(description = "retutning back(only works in editor)")]
    Back,
}

pub(crate) async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

pub(crate) async fn cancel(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Cancelling dialogue").await?;
    dialogue.update(QuizState::Start).await?;
    Ok(())
}

pub(crate) async fn start(bot: Bot, msg: Message, dialogue: UserDialogue) -> HandlerResult {
    bot.send_message(msg.chat.id, "Please choose what to do:")
        .reply_markup(action_keyboard(msg.chat.username().unwrap()))
        .await?;
    dialogue.update(QuizState::Start).await?;
    Ok(())
}
