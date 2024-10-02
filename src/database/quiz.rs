use std::{fmt, path::Display, vec};

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Quiz {
    uuid: Uuid,
    title: String,
    description: String,
    author: String,
    questions: Vec<Question>,
}

#[derive(Debug, Clone)]
pub struct Question {
    uuid: Uuid,
    text: String,
    answers: Vec<Answer>,
}

#[derive(Debug, Clone)]
pub struct Answer {
    uuid: Uuid,
    text: String,
    is_correct: bool,
}

impl fmt::Display for Quiz {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut questions = String::new();
        for question in self.questions() {
            questions.push_str(&format!("# {}\n", question.to_string()));
        }
        write!(f, "<b>{}</b>\n<i>{}</i>\n\nBy @{}\n\nQuestions:{}", self.title(), self.description(), self.author(), questions)
    }
}

impl fmt::Display for Question {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut answers = String::new();
        for (i, answer) in self.answers().into_iter().enumerate() {
            answers.push_str(&format!("{}){}\n", i+1, answer.to_string()));
        }
        answers.push('\n');

        write!(f, "__{}__\n{}", self.text(), answers)
    }
}

impl fmt::Display for Answer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.text(), if self.is_correct() { 'V' } else { 'X' })
    }
}

impl Quiz {
    pub fn new(
        title: String,
        description: String,
        author: String,
        questions: Option<Vec<Question>>,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            title,
            description,
            author,
            questions: questions.unwrap_or_default(),
        }
    }

    pub fn retreive(uuid: Uuid, title: String, description: String, author: String) -> Self {
        Self {
            uuid,
            title,
            description,
            author,
            questions: vec![],
        }
    }

    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn title(&self) -> &String {
        &self.title
    }

    pub fn description(&self) -> &String {
        &self.description
    }

    pub fn author(&self) -> &String {
        &self.author
    }

    pub fn questions(&self) -> &[Question] {
        &self.questions
    }

    pub fn add_question(&mut self, question: Question) {
        self.questions.push(question);
    }

    // pub fn create(title: String, description: String, author: String, questions: Vec<Question>) -> Self
}

impl Question {
    pub fn new(text: String, answers: Option<Vec<Answer>>) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            text,
            answers: answers.unwrap_or_default(),
        }
    }

    pub fn retreive(uuid: Uuid, text: String) -> Self {
        Self {
            uuid,
            text,
            answers: vec![],
        }
    }

    pub fn text(&self) -> String {
        self.text.clone()
    }

    pub fn answers(&self) -> &[Answer] {
        &self.answers
    }

    pub fn add_answer(&mut self, answer: Answer) {
        self.answers.push(answer);
    }

    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }
}

impl Answer {
    pub fn new(text: String, is_correct: bool) -> Answer {
        Self {
            uuid: Uuid::new_v4(),
            text,
            is_correct,
        }
    }

    pub fn retreive(uuid: Uuid, text: String, is_correct: bool) -> Answer {
        Self {
            uuid,
            text,
            is_correct,
        }
    }

    pub fn text(&self) -> String {
        self.text.clone()
    }

    pub fn is_correct(&self) -> bool {
        self.is_correct
    }

    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }
}
