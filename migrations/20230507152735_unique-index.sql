-- Add migration script here
  CREATE UNIQUE INDEX session_day_idx ON previous_session_times(session_name, day);
