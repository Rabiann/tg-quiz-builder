-- Add migration script here
CREATE TABLE IF NOT EXISTS quizes (
    uuid UUID PRIMARY KEY NOT NULL,
    name VARCHAR UNIQUE NOT NULL,
    description VARCHAR NOT NULL,
    author VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS questions (
    uuid UUID PRIMARY KEY NOT NULL,
    text VARCHAR NOT NULL,
    quiz_id UUID NOT NULL,
    FOREIGN KEY(quiz_id) REFERENCES quizes(uuid)
    ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS answers (
    uuid UUID PRIMARY KEY NOT NULL,
    text VARCHAR NOT NULL,
    is_correct BOOLEAN NOT NULL,
    question_id UUID NOT NULL,
    FOREIGN KEY(question_id) REFERENCES questions(uuid) 
    ON DELETE CASCADE
);
