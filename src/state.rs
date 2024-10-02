use crate::quiz::{Answer, Question, Quiz};

#[derive(Debug, Clone)]
pub struct QuizData {
    pub(crate) quiz_name: String,
    pub(crate) description: String,
    pub(crate) author: String,
    pub(crate) questions: Vec<Question>,
}

#[derive(Debug, Clone, Default)]
pub enum QuizState {
    #[default]
    Start,
    // PART FOR --- CREATING QUIZ ---
    ReceiveQuizName,
    ReceiveQuizDescription {
        quiz_name: String,
    },
    ReceiveQuizAuthor {
        quiz_info: QuizData,
    },
    ReceiveNewQuestion {
        quiz_info: QuizData,
    },
    ReceiveNewAnswer {
        quiz_info: QuizData,
        new_question: String,
        answers: Vec<Answer>,
    },
    ReceiveAnswerIsCorrect {
        quiz_info: QuizData,
        new_question: String,
        answers: Vec<Answer>,
        new_answer: String,
    },
    ReceiveAddAnotherAnswer {
        quiz_info: QuizData,
        new_question: String,
        answers: Vec<Answer>,
    },
    ReceiveAddAnotherQuestion {
        quiz_info: QuizData,
    },

    // PART FOR --- RUNNING QUIZ ---
    Selection,
    ReadyToRun {
        quiz: Quiz,
        curr_idx: usize,
    },
    Running {
        quiz: Quiz,
        curr_idx: usize,
        score: u32,
    },
    Done {
        score: u32,
    },

    // PART FOR --- EDITING ---
    StartSelect,
    HandleQuiz {
        quiz_name: String
    }, 
    HandleQuestion {
        quiz_name: String,
        question_name: String,
    },
    // DeleteQuiz {
    //     quiz_name: String,
    // },
    EditName {
        quiz_name: String,
        // new_name: String,
    },
    EditDescription {
        quiz_name: String,
        // new_description: String,
    },
    AddQuestion {
        quiz_name: String,
    },
    AddQuestionText {
        quiz_name: String,
        text: String,
        answers: Vec<Answer>,
    },
    AddQuestionAnswer {
        quiz_name: String,
        text: String,
        answers: Vec<Answer>,
        new_answer: String,
    },
    AddQuestionCorectness {
        quiz_name: String,
        text: String,
        answers: Vec<Answer>,
        new_answer: String,
        is_correct: bool,
    },
    AddQuestionReceiveAnotherAnswer {
        quiz_name: String,
        text: String,
        answers: Vec<Answer>,
    },
    SelectQuestion {
        quiz_name: String,
        // question_name: String,
    },
    EditQuestionText {
        quiz_name: String,
        question_name: String,
        // new_text: String,
    },
    DeleteQuestion {
        quiz_name: String,
        question_name: String,
    },
    AddAnswer {
        quiz_name: String,
        question_name: String,
    },
    AddAnswerText {
        quiz_name: String,
        question_name: String,
        text: String,
    },
    AddAnswerCorrectness {
        quiz_name: String,
        question_name: String,
        text: String,
    },
    SelectAnswer {
        quiz_name: String,
        question_name: String,
    },
    HandleAnswer {
        quiz_name: String,
        question_name: String,
        answer_name: String
    },
    EditAnswerText {
        quiz_name: String,
        question_name: String,
        answer_name: String,
    },
    EditCorectness {
        quiz_name: String,
        question_name: String,
        answer_name: String,
    },
    DeleteAnswer {
        quiz_name: String,
        question_name: String,
        answer_name: String,
    },
}
