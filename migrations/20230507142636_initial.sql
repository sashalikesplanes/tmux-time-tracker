-- Add migration script here
CREATE TABLE IF NOT EXISTS session_times (
    session_name TEXT PRIMARY KEY,
    -- stored as unix epoch seconds
    last_attached_time INTEGER DEFAUL NULL
);

CREATE TABLE IF NOT EXISTS previous_session_times (
    id INTEGER PRIMARY KEY,
    session_name TEXT,
    -- text representation of the date
    day TEXT REQUIRED,
    -- number of whole seconds
    time_attached INTEGER DEFAULT 0,

    FOREIGN KEY (session_name) REFERENCES session_times(session_name)
);
