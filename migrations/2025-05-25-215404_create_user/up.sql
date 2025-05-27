CREATE TABLE USERS
(
    id            UUID NOT NULL PRIMARY KEY,
    parent_id     UUID DEFAULT NULL,
    username      TEXT NOT NULL,
    password_hash TEXT NOT NULL
);

