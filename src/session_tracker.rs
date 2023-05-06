use std::time;

use anyhow::Result;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

pub struct SessionTracker {
    pool: sqlx::SqlitePool,
}

impl SessionTracker {
    pub async fn new(db_url: &str) -> Result<Self> {
        // Setup DB connection
        if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
            Sqlite::create_database(db_url).await?
        }
        let pool = SqlitePool::connect(db_url).await?;
        let tracker = SessionTracker { pool };
        tracker.create_table_if_not_exists().await?;
        Ok(tracker)
    }

    async fn create_table_if_not_exists(&self) -> Result<()> {
        // Create the database if it doesn't exist
        sqlx::query!(
            "
CREATE TABLE IF NOT EXISTS session_times (
    session_name TEXT PRIMARY KEY,
    last_attached_time DOUBLE DEFAUL NULL,
    total_attached_time DOUBLE DEFAULT 0
)
        "
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn create_session_if_not_exists(&self, session_name: &str) -> Result<()> {
        let record = sqlx::query!(
            "SELECT session_name FROM session_times WHERE session_name = ?",
            session_name
        )
        .fetch_optional(&self.pool)
        .await?;

        match record {
            Some(_) => Ok(()),
            None => {
                sqlx::query!(
                    "INSERT INTO session_times (session_name) VALUES (?)",
                    session_name
                )
                .execute(&self.pool)
                .await?;
                Ok(())
            }
        }
    }

    pub async fn attach_to_session(&self, session_name: &str) -> Result<()> {
        self.create_session_if_not_exists(session_name).await?;

        let time_since_unix = get_seconds_since_unix();
        sqlx::query!(
            "UPDATE session_times SET last_attached_time = ? WHERE session_name = ?",
            time_since_unix,
            session_name
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn detach_from_all_sessions(&self) -> Result<()> {
        let records = sqlx::query!(
            "SELECT session_name, total_attached_time, last_attached_time
                FROM session_times
                WHERE last_attached_time IS NOT NULL",
        )
        .fetch_all(&self.pool)
        .await?;

        for record in records {
            let session_name = record
                .session_name
                .expect("Primary key should always exist");
            let total_attached_time = record.total_attached_time.unwrap_or(0.0);
            let last_attached_time = record
                .last_attached_time
                .unwrap_or(get_seconds_since_unix());
            let new_total_attached_time =
                total_attached_time + get_seconds_since_unix() - last_attached_time;

            sqlx::query!(
                "UPDATE session_times SET last_attached_time = NULL, total_attached_time = ? WHERE session_name = ?",
                new_total_attached_time,
                session_name
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_total_session_time_in_seconds(&self, session_name: &str) -> Result<f64> {
        self.create_session_if_not_exists(session_name).await?;

        let record = sqlx::query!(
            "SELECT total_attached_time FROM session_times WHERE session_name = ?",
            session_name
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record.total_attached_time.unwrap_or(0.0))
    }

    pub async fn get_total_session_time_in_hours(&self, session_name: &str) -> Result<f64> {
        let total_attached_time_in_seconds =
            self.get_total_session_time_in_seconds(session_name).await?;
        Ok(total_attached_time_in_seconds / 60.0 / 60.0)
    }

    pub async fn print_all_sessions_total_attached_time(&self) -> Result<()> {
        let records = sqlx::query!("SELECT session_name, total_attached_time FROM session_times")
            .fetch_all(&self.pool)
            .await?;

        for record in records {
            let session_name = record
                .session_name
                .expect("Primary key should always exist");
            let total_attached_time = record.total_attached_time.unwrap_or(0.0) / 60.0 / 60.0;
            println!("Session: {} - {} h", session_name, total_attached_time);
        }
        Ok(())
    }

    pub async fn clear_all_sessions(&self) -> Result<()> {
        sqlx::query!("UPDATE session_times SET total_attached_time = 0")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

fn get_seconds_since_unix() -> f64 {
    let now = time::SystemTime::now();
    let unix_time = now
        .duration_since(time::UNIX_EPOCH)
        .expect("Time went backwards");
    unix_time.as_secs_f64()
}
