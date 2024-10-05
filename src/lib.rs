use state::QuizState;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::Dialogue};

pub mod commands;
pub mod constructor;
pub mod database;
pub mod editor;
pub mod keyboard;
pub mod runner;
pub mod schema;
pub mod state;

type UserDialogue = Dialogue<QuizState, InMemStorage<QuizState>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;
