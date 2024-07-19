-- Your SQL goes here


CREATE TABLE characters (
    id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    name TEXT,

    PRIMARY KEY(user_id,id)
);

