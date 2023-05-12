//! # session_tracker
//!
//! Provides a SessionTracker struct which provides
//! access to a sqlite database along with methods
//! for manipulating the database

use anyhow::Result;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

/// A Struct holding a connection pool for a Sqlite db
pub struct SessionTracker {
    pool: sqlx::SqlitePool,
}

impl SessionTracker {
    /// Setups the database connection, and creates the database if it does not exist
    pub async fn new(home_path: &str) -> Result<Self> {
        let db_url = format!("sqlite://{home_path}/tmux.db");

        if !Sqlite::database_exists(db_url.as_str()).await.unwrap_or(false) {
            Sqlite::create_database(db_url.as_str()).await?
        }
        let pool = SqlitePool::connect(db_url.as_str()).await?;

        sqlx::migrate!().run(&pool).await?;

        let tracker = SessionTracker { pool };
        Ok(tracker)
    }

    /// Stores the attachement to a session in the db
    pub async fn attach_to_session(&self, session_name: &str) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO session_times (session_name, last_attached_time)
                VALUES (?, unixepoch())
            ON CONFLICT (session_name) DO UPDATE
                SET last_attached_time = unixepoch()
            "#,
            session_name
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Marks all sessions as detached, updates the current day with the duration of attachement
    /// We do this across all sessions in case multiple have been attached to
    pub async fn detach_from_all_sessions(&self) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Select the current attached times and save them
        sqlx::query!(
            "
            WITH records AS (
                SELECT session_name, (unixepoch() - last_attached_time) as attached_time
                FROM session_times
                WHERE last_attached_time IS NOT NULL
            )
            INSERT INTO previous_session_times (session_name, day, time_attached)
                SELECT session_name, date(), attached_time
                FROM records
                WHERE true
            ON CONFLICT (session_name, day) DO UPDATE SET
                time_attached = time_attached + excluded.time_attached
            "
        )
        .execute(&mut tx)
        .await?;

        // Set the current attached times to NULL
        sqlx::query!("UPDATE session_times SET last_attached_time = NULL WHERE last_attached_time IS NOT NULL") .execute(&mut tx) .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Gets the total time in seconds that the session has been attached to today
    pub async fn get_today_session_time_in_seconds(&self, session_name: &str) -> Result<i64> {
        let record = sqlx::query!(
            r#"SELECT time_attached FROM previous_session_times WHERE session_name = ? AND day = date()"#,
            session_name
        )
        .fetch_optional(&self.pool)
        .await?;

        match record {
            None => return Ok(0),
            Some(record) => Ok(record.time_attached.unwrap_or(0)),
        }
    }

    /// Gets the total time in hours that the session has been attached to today
    pub async fn get_daily_session_time_in_hours(&self, session_name: &str) -> Result<i64> {
        let total_attached_time_in_seconds =
            self.get_today_session_time_in_seconds(session_name).await?;
        Ok(total_attached_time_in_seconds / (60 * 60))
    }

    pub async fn get_weekly_session_time_in_hours(&self, session_name: &str) -> Result<i32> {
        let record = sqlx::query!(
            r#"
            SELECT sum(time_attached) / 60 / 60 as total_attached_time
                FROM previous_session_times
                WHERE session_name = ? AND day >= date('now', 'weekday 0', '-7 day');
            "#,
            session_name
        )
        .fetch_optional(&self.pool)
        .await?;

        match record {
            None => return Ok(0),
            Some(record) => Ok(record.total_attached_time.unwrap_or(0)),
        }
    }
}
