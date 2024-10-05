use std::{borrow::Cow, error::Error};

use sqlx::postgres::PgPool;

use super::quiz::{Answer, Question, Quiz};

pub(crate) struct Connection {
    pool: PgPool,
}

impl Connection {
    pub(crate) async fn connect<'a>(connection_string: Cow<'a, str>) -> Self {
        let pool = PgPool::connect(&connection_string)
            .await
            .expect("Failed to connect to database");
        Self { pool }
    }

    pub(crate) async fn perform_connection_if_needed(&self) {
        let tables = sqlx::query!("SELECT * FROM information_schema.tables")
            .fetch_all(&self.pool)
            .await
            .expect("Table retreiving failed.");

        if tables.len() < 1 {
            sqlx::migrate!().run(&self.pool).await.unwrap();
        }
    }
}

type GenericError = Result<String, Box<dyn Error + Send + Sync>>;

pub(crate) trait CreateQuiz {
    async fn create_quiz(&self, quiz: Quiz) -> GenericError;
}

pub(crate) trait DeleteQuiz {
    async fn delete_quiz(&self, id: impl Into<String>) -> GenericError;
}

pub(crate) trait DeleteQuestion {
    async fn delete_question(
        &self,
        quiz_id: impl Into<String>,
        question_id: impl Into<String>,
    ) -> GenericError;
}

pub(crate) trait DeleteAnswer {
    async fn delete_answer(
        &self,
        quiz_id: impl Into<String>,
        question_id: impl Into<String>,
        answer_id: impl Into<String>,
    ) -> GenericError;
}

pub(crate) trait RetreiveQuiz {
    async fn retreive_quiz(
        &self,
        id: impl Into<String>,
    ) -> Result<Option<Quiz>, Box<dyn Error + Send + Sync>>;

    async fn retreive_all_quiz_names(&self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>>;
}

pub(crate) trait EditQuiz {
    async fn edit_name(&self, id: impl Into<String>, new: impl Into<String>) -> GenericError;

    async fn edit_description(&self, id: impl Into<String>, new: impl Into<String>)
        -> GenericError;

    // async fn add_question(&self, new: Question) -> GenericError;

    // async fn delete_question(&self, id: impl Into<String>) -> GenericError;
}

pub(crate) trait RetreiveQuestion {
    async fn retreive_question(
        &self,
        id_quiz: impl Into<String>,
        id_question: impl Into<String>,
    ) -> Result<Option<Question>, Box<dyn Error + Send + Sync>>;

    async fn retreive_all_question_names(
        &self,
        id_quiz: impl Into<String>,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>>;
}

pub(crate) trait EditQuestion {
    async fn edit_text(
        &self,
        quiz_id: impl Into<String>,
        question_id: impl Into<String>,
        new: impl Into<String>,
    ) -> GenericError;

    // async fn add_answer(&self, new: Answer) -> GenericError;

    // async fn delete_answer(&self, id: impl Into<String>) -> GenericError;
}

pub(crate) trait CreateAnswer {
    async fn create_answer(
        &self,
        quiz_id: impl Into<String>,
        question_id: impl Into<String>,
        new: impl Into<String>,
        is_correct: bool,
    ) -> GenericError;
}

pub(crate) trait CreateQuestion {
    async fn create_question(
        &self,
        quiz_id: impl Into<String>,
        new: impl Into<String>,
    ) -> GenericError;
}

pub(crate) trait RetreiveAnswer {
    async fn retreive_answer(
        &self,
        id_quiz: impl Into<String>,
        id_question: impl Into<String>,
        id_answer: impl Into<String>,
    ) -> Result<Option<Answer>, Box<dyn Error + Send + Sync>>;

    async fn retreive_all_answers_names(
        &self,
        id_quiz: impl Into<String>,
        id_question: impl Into<String>,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>>;
}

pub(crate) trait EditAnswer {
    async fn edit_answer_text(
        &self,
        quiz_id: impl Into<String>,
        question_id: impl Into<String>,
        answer_id: impl Into<String>,
        new: impl Into<String>,
    ) -> GenericError;

    async fn edit_corectness(
        &self,
        quiz_id: impl Into<String>,
        question_id: impl Into<String>,
        answer_id: impl Into<String>,
        is_correct: bool,
    ) -> GenericError;
}

impl CreateQuiz for Connection {
    async fn create_quiz(&self, quiz: Quiz) -> GenericError {
        log::debug!("Creating transaction");
        let mut tx = self.pool.begin().await?;

        log::debug!("Adding quiz");
        let name = sqlx::query!(
            "INSERT INTO quizes VALUES ($1, $2, $3, $4) RETURNING name",
            quiz.uuid(),
            quiz.title(),
            quiz.description(),
            quiz.author()
        )
        .fetch_one(&mut *tx)
        .await?
        .name;

        log::debug!("Adding questions");
        for question in quiz.questions() {
            log::debug!(
                "Adding question {} with uuid {}",
                question.text(),
                question.uuid()
            );
            sqlx::query!(
                "INSERT INTO questions VALUES ($1, $2, $3)",
                question.uuid(),
                question.text(),
                quiz.uuid()
            )
            .execute(&mut *tx)
            .await?;

            for answer in question.answers() {
                log::debug!(
                    "Adding answer {} with uuid {}",
                    question.text(),
                    question.uuid()
                );
                sqlx::query!(
                    "INSERT INTO answers VALUES ($1, $2, $3, $4)",
                    answer.uuid(),
                    answer.text(),
                    answer.is_correct(),
                    question.uuid()
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        log::debug!("Closing transaction");
        tx.commit().await?;

        Ok(name)
    }
}

impl RetreiveQuiz for Connection {
    async fn retreive_quiz(
        &self,
        id: impl Into<String>,
    ) -> Result<Option<Quiz>, Box<dyn Error + Send + Sync>> {
        let mut tx = self.pool.begin().await?;

        let quiz_record = sqlx::query!(
            "SELECT uuid, name, description, author FROM quizes WHERE name = $1",
            id.into()
        )
        .fetch_one(&mut *tx)
        .await;

        match quiz_record {
            Ok(quiz_record) => {
                let mut quiz = Quiz::retreive(
                    quiz_record.uuid,
                    quiz_record.name,
                    quiz_record.description,
                    quiz_record.author,
                );

                let quiz_questions = sqlx::query!(
                    "SELECT uuid, text, quiz_id FROM questions WHERE quiz_id = $1",
                    quiz_record.uuid
                )
                .fetch_all(&mut *tx)
                .await?;

                for question_record in quiz_questions {
                    let mut question =
                        Question::retreive(question_record.uuid, question_record.text);

                    let question_answers = sqlx::query!("SELECT uuid, text, is_correct, question_id FROM answers WHERE question_id = $1", question_record.uuid).fetch_all(&mut *tx).await?;

                    question_answers.into_iter().for_each(|answer| {
                        question.add_answer(Answer::retreive(
                            answer.uuid,
                            answer.text,
                            answer.is_correct,
                        ))
                    });

                    quiz.add_question(question);
                }

                tx.commit().await?;

                Ok(Some(quiz))
            }
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(None),
                e => Err(Box::new(e)),
            },
        }
    }

    async fn retreive_all_quiz_names(&self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let quizes_records = sqlx::query!("SELECT name FROM quizes")
            .fetch_all(&self.pool)
            .await?;

        let quiz_names: Vec<String> = quizes_records.into_iter().map(|q| q.name).collect();

        Ok(quiz_names)
    }
}

impl DeleteQuiz for Connection {
    async fn delete_quiz(&self, id: impl Into<String>) -> GenericError {
        let id = id.into();
        sqlx::query!("DELETE FROM quizes WHERE name = $1", &id)
            .execute(&self.pool)
            .await?;

        Ok(id)
    }
}

impl EditQuiz for Connection {
    async fn edit_name(&self, id: impl Into<String>, new: impl Into<String>) -> GenericError {
        let new = sqlx::query!(
            "UPDATE quizes SET name=$1 WHERE name=$2 RETURNING name",
            new.into(),
            id.into()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(new.name)
    }

    async fn edit_description(
        &self,
        id: impl Into<String>,
        new: impl Into<String>,
    ) -> GenericError {
        let new = sqlx::query!(
            "UPDATE quizes SET description=$1 WHERE name=$2 RETURNING description",
            new.into(),
            id.into()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(new.description)
    }

    // async fn add_question(&self, new: Question) -> GenericError {
    //     todo!()
    // }

    // async fn delete_question(&self, id: impl Into<String>) -> GenericError {
    //     todo!()
    // }
}

impl RetreiveQuestion for Connection {
    async fn retreive_question(
        &self,
        id_quiz: impl Into<String>,
        id_question: impl Into<String>,
    ) -> Result<Option<Question>, Box<dyn Error + Send + Sync>> {
        let mut tx = self.pool.begin().await?;

        let question = sqlx::query!("SELECT questions.uuid, questions.text FROM questions INNER JOIN quizes ON questions.quiz_id = quizes.uuid WHERE quizes.name = $1 AND questions.text = $2", id_quiz.into(), id_question.into()).fetch_one(&mut *tx).await;
        match question {
            Ok(question) => {
                let mut question = Question::retreive(question.uuid, question.text);
                let answers = sqlx::query!(
                    "SELECT uuid, text, is_correct FROM answers WHERE question_id = $1",
                    question.uuid()
                )
                .fetch_all(&mut *tx)
                .await?;
                answers
                    .into_iter()
                    .map(|answer| Answer::retreive(answer.uuid, answer.text, answer.is_correct))
                    .for_each(|answer| question.add_answer(answer));

                tx.commit().await?;

                Ok(Some(question))
            }
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(None),
                e => Err(Box::new(e)),
            },
        }
    }

    async fn retreive_all_question_names(
        &self,
        id_quiz: impl Into<String>,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let question_records = sqlx::query!("SELECT text FROM questions INNER JOIN quizes ON questions.quiz_id = quizes.uuid WHERE quizes.name = $1", id_quiz.into()).fetch_all(&self.pool).await?;

        let question_names = question_records.into_iter().map(|q| q.text).collect();

        Ok(question_names)
    }
}

impl DeleteQuestion for Connection {
    async fn delete_question(
        &self,
        quiz_id: impl Into<String>,
        question_id: impl Into<String>,
    ) -> GenericError {
        let deleted = sqlx::query!("DELETE FROM questions USING quizes WHERE quizes.uuid = questions.quiz_id AND quizes.name = $1 AND questions.text = $2 RETURNING questions.text", quiz_id.into(), question_id.into()).fetch_one(&self.pool).await?;

        Ok(deleted.text)
    }
}

impl EditQuestion for Connection {
    async fn edit_text(
        &self,
        quiz_id: impl Into<String>,
        question_id: impl Into<String>,
        new: impl Into<String>,
    ) -> GenericError {
        let updated = sqlx::query!("UPDATE questions SET text = $1 FROM quizes WHERE quizes.uuid = questions.quiz_id AND quizes.name = $2 AND questions.text = $3 RETURNING text", new.into(), quiz_id.into(), question_id.into()).fetch_one(&self.pool).await?;

        Ok(updated.text)
    }
}

impl RetreiveAnswer for Connection {
    async fn retreive_answer(
        &self,
        id_quiz: impl Into<String>,
        id_question: impl Into<String>,
        id_answer: impl Into<String>,
    ) -> Result<Option<Answer>, Box<dyn Error + Send + Sync>> {
        let answer = sqlx::query!("SELECT answers.uuid, answers.text, answers.is_correct FROM answers INNER JOIN questions ON answers.question_id = questions.uuid INNER JOIN quizes ON quizes.uuid = questions.quiz_id WHERE quizes.name = $1 AND questions.text = $2 AND answers.text = $3", id_quiz.into(), id_question.into(), id_answer.into()).fetch_one(&self.pool).await?;

        Ok(Some(Answer::retreive(
            answer.uuid,
            answer.text,
            answer.is_correct,
        )))
    }

    async fn retreive_all_answers_names(
        &self,
        id_quiz: impl Into<String>,
        id_question: impl Into<String>,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let answers = sqlx::query!("SELECT answers.text FROM answers INNER JOIN questions ON answers.question_id = questions.uuid INNER JOIN quizes ON quizes.uuid = questions.quiz_id WHERE questions.text = $1 AND quizes.name = $2", id_question.into(), id_quiz.into()).fetch_all(&self.pool).await?;

        Ok(answers.into_iter().map(|r| r.text).collect())
    }
}

impl DeleteAnswer for Connection {
    async fn delete_answer(
        &self,
        quiz_id: impl Into<String>,
        question_id: impl Into<String>,
        answer_id: impl Into<String>,
    ) -> GenericError {
        let answer = sqlx::query!("DELETE FROM answers USING questions, quizes WHERE questions.uuid = answers.question_id AND questions.text = $1 AND questions.quiz_id = quizes.uuid AND quizes.name = $2 AND answers.text = $3 RETURNING answers.text", question_id.into(), quiz_id.into(), answer_id.into()).fetch_one(&self.pool).await?;

        Ok(answer.text)
    }
}

impl EditAnswer for Connection {
    async fn edit_answer_text(
        &self,
        quiz_id: impl Into<String>,
        question_id: impl Into<String>,
        answer_id: impl Into<String>,
        new: impl Into<String>,
    ) -> GenericError {
        let record = sqlx::query!("UPDATE answers SET text = $1 FROM questions INNER JOIN quizes ON questions.quiz_id = quizes.uuid WHERE questions.uuid = answers.question_id AND quizes.name = $2 AND questions.text = $3 AND answers.text = $4 RETURNING answers.text", new.into(), quiz_id.into(), question_id.into(), answer_id.into()).fetch_one(&self.pool).await?;

        Ok(record.text)
    }

    async fn edit_corectness(
        &self,
        quiz_id: impl Into<String>,
        question_id: impl Into<String>,
        answer_id: impl Into<String>,
        is_correct: bool,
    ) -> GenericError {
        let record = sqlx::query!("UPDATE answers SET is_correct = $1 FROM questions INNER JOIN quizes ON questions.quiz_id = quizes.uuid WHERE questions.uuid = answers.question_id AND quizes.name = $2 AND questions.text = $3 AND answers.text = $4 RETURNING answers.text", is_correct, quiz_id.into(), question_id.into(), answer_id.into()).fetch_one(&self.pool).await?;

        Ok(record.text)
    }
}

impl CreateAnswer for Connection {
    async fn create_answer(
        &self,
        quiz_id: impl Into<String>,
        question_id: impl Into<String>,
        new: impl Into<String>,
        is_correct: bool,
    ) -> GenericError {
        let quiz_uuid = sqlx::query!(
            "SELECT uuid FROM quizes WHERE quizes.name = $1",
            quiz_id.into()
        )
        .fetch_one(&self.pool)
        .await?;

        let question_uuid = sqlx::query!(
            "SELECT uuid FROM questions WHERE questions.quiz_id = $1 AND text = $2",
            quiz_uuid.uuid,
            question_id.into()
        )
        .fetch_one(&self.pool)
        .await?;

        let new_answer = Answer::new(new.into(), is_correct);

        let added = sqlx::query!(
            "INSERT INTO answers VALUES ($1, $2, $3, $4) RETURNING text",
            new_answer.uuid(),
            new_answer.text(),
            new_answer.is_correct(),
            question_uuid.uuid
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(added.text)
    }
}

impl CreateQuestion for Connection {
    async fn create_question(
        &self,
        quiz_id: impl Into<String>,
        new: impl Into<String>,
    ) -> GenericError {
        let quiz_uuid = sqlx::query!(
            "SELECT uuid FROM quizes WHERE quizes.name = $1",
            quiz_id.into()
        )
        .fetch_one(&self.pool)
        .await?;

        let new_question = Question::new(new.into(), None);

        let added = sqlx::query!(
            "INSERT INTO questions VALUES ($1, $2, $3) RETURNING text",
            new_question.uuid(),
            new_question.text(),
            quiz_uuid.uuid
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(added.text)
    }
}
