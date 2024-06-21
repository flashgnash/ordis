-- Your SQL goes here


CREATE TABLE users (
    id TEXT NOT NULL,
    username TEXT,
    count INTEGER DEFAULT 0,

    PRIMARY KEY(id)
);

