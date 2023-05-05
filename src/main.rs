use anyhow::Result;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

const USAGE_MESSAGE: &str =
    "Usage: tmux-time-tracker <action: attach/detach/gets/geth> [session_name]";

struct SessionTracker {
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

    pub async fn create_session_if_not_exist(&self, session_name: &str) -> Result<()> {
        // Check if session exists
        let session = sqlx::query!(
            "SELECT session_name FROM session_times WHERE session_name = ?",
            session_name
        )
        .fetch_optional(&self.pool)
        .await?;

        match session {
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
    async fn attach_to_session(&self, session_name: &str) -> Result<()> {
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

    async fn detach_from_all_sessions(&self) -> Result<()> {
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

    async fn get_total_session_time_in_seconds(&self, session_name: &str) -> Result<f64> {
        let record = sqlx::query!(
            "SELECT total_attached_time FROM session_times WHERE session_name = ?",
            session_name
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record.total_attached_time.unwrap_or(0.0))
    }

    async fn get_total_session_time_in_hours(&self, session_name: &str) -> Result<f64> {
        let total_attached_time_in_seconds =
            self.get_total_session_time_in_seconds(session_name).await?;
        Ok(total_attached_time_in_seconds / 60.0 / 60.0)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let db_url: &str = &env::var("DATABASE_URL")?;
    let tracker = SessionTracker::new(db_url).await?;

    // Parse CLI args
    let args: Vec<String> = env::args().collect();
    let action = args.get(1).expect(USAGE_MESSAGE).to_lowercase();

    match action.as_str() {
        "detach" => tracker.detach_from_all_sessions().await?,
        _ => with_session_name_branch(args, tracker).await?,
    }

    Ok(())
}

async fn with_session_name_branch(args: Vec<String>, tracker: SessionTracker) -> Result<()> {
    let action = args.get(1).expect(USAGE_MESSAGE).to_lowercase();
    let session_name = args.get(2).expect(USAGE_MESSAGE).to_lowercase();

    tracker
        .create_session_if_not_exist(session_name.as_str())
        .await?;

    match action.as_str() {
        "attach" => tracker.attach_to_session(session_name.as_str()).await?,
        "gets" => println!(
            "total_attached_time: {} s",
            tracker
                .get_total_session_time_in_seconds(session_name.as_str())
                .await?
        ),
        "geth" => println!(
            "total_attached_time: {} h",
            tracker
                .get_total_session_time_in_hours(session_name.as_str())
                .await?
        ),
        _ => panic!("{}", USAGE_MESSAGE),
    }

    Ok(())
}

fn get_seconds_since_unix() -> f64 {
    let now = SystemTime::now();
    let unix_time = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    unix_time.as_secs_f64()
}
