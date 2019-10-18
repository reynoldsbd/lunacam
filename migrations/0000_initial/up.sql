CREATE TABLE settings (

    name
        TEXT
        PRIMARY KEY ASC
        NOT NULL,

    value
        TEXT
        NOT NULL

);

CREATE TABLE cameras (

    id
        INTEGER
        PRIMARY KEY ASC
        NOT NULL,

    name
        TEXT
        NOT NULL,

    address
        TEXT
        NOT NULL,

    enabled
        BOOLEAN
        NOT NULL
        DEFAULT FALSE,

    orientation
        INTEGER
        NOT NULL
        DEFAULT 0

);

CREATE TABLE users (

    id
        INTEGER
        PRIMARY KEY ASC
        NOT NULL,

    username
        TEXT
        NOT NULL
        UNIQUE,

    pwhash
        TEXT
        NOT NULL

);
