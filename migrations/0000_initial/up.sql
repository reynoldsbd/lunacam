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

    friendly_name
        TEXT
        NOT NULL,

    hostname
        TEXT
        NOT NULL,

    device_key
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
