use dotenvy::dotenv;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup};

use crate::database::quiz::Answer;

pub(crate) fn yes_no_keyboard() -> KeyboardMarkup {
    let keyboard: Vec<Vec<KeyboardButton>> = vec![vec![
        KeyboardButton::new("Yes✔️"),
        KeyboardButton::new("No❌"),
    ]];

    KeyboardMarkup::new(keyboard)
}

pub(crate) fn answers_keyboard(answers: &[Answer]) -> InlineKeyboardMarkup {
    let keyboard: Vec<Vec<InlineKeyboardButton>> = answers
        .into_iter()
        .map(|ans| vec![InlineKeyboardButton::callback(ans.text(), ans.text())])
        .collect();

    InlineKeyboardMarkup::new(keyboard)
}

pub(crate) fn quizes_keyboard(quizes: &[String]) -> KeyboardMarkup {
    let keyboard = quizes
        .into_iter()
        .map(|quiz| vec![KeyboardButton::new(quiz)]);

    KeyboardMarkup::new(keyboard)
}

pub(crate) fn questions_keyboard(questions: &[String]) -> KeyboardMarkup {
    let keyboard = questions
        .into_iter()
        .map(|question| vec![KeyboardButton::new(question)]);

    KeyboardMarkup::new(keyboard)
}

pub(crate) fn answers_block_keyboard(answers: &[String]) -> KeyboardMarkup {
    let keyboard = answers
        .into_iter()
        .map(|answer| vec![KeyboardButton::new(answer)]);

    KeyboardMarkup::new(keyboard)
}

pub(crate) fn action_keyboard(username: impl Into<String>) -> KeyboardMarkup {
    dotenv().ok();

    let admin = std::env::var("ADMIN_NAME").unwrap_or_default();

    let mut keyboard = vec![vec![KeyboardButton::new("Take a quiz📝")]];

    if username.into() == admin {
        keyboard.push(vec![KeyboardButton::new("Create a new quiz🏗️")]);
        keyboard.push(vec![KeyboardButton::new("Edit an existing quiz✏️️")]);
    }

    KeyboardMarkup::new(keyboard)
}

pub(crate) fn edit_quiz_keyboard() -> KeyboardMarkup {
    let keyboard = vec![
        vec![
            KeyboardButton::new("Edit name"),
            KeyboardButton::new("Edit description"),
        ],
        vec![
            KeyboardButton::new("Edit question"),
            KeyboardButton::new("Add question"),
        ],
        vec![KeyboardButton::new("Delete quiz🗑️")],
    ];

    KeyboardMarkup::new(keyboard)
}

pub(crate) fn edit_question_keyboard() -> KeyboardMarkup {
    let keyboard = vec![
        vec![
            KeyboardButton::new("Edit text"),
            KeyboardButton::new("Edit answer"),
        ],
        vec![KeyboardButton::new("Add answer")],
        vec![KeyboardButton::new("Delete question🗑️")],
    ];

    KeyboardMarkup::new(keyboard)
}

pub(crate) fn edit_answer_keyboard() -> KeyboardMarkup {
    let keyboard = vec![
        vec![
            KeyboardButton::new("Edit text"),
            KeyboardButton::new("Edit corectness"),
        ],
        vec![KeyboardButton::new("Delete answer")],
    ];

    KeyboardMarkup::new(keyboard)
}
