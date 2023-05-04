use std::env;

use sqlx::{Sqlite, SqlitePool, migrate::MigrateDatabase};

const DB_URL: &str = "sqlite://tmux.db";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup DB connection
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        Sqlite::create_database(DB_URL).await?
    }
    let pool = SqlitePool::connect(DB_URL).await?;

    // Parse CLI args
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        panic!("Usage tmux-time-tracker <session_name> <action: attach/detach/report>");
    }

    let session_id = args[1].to_lowercase();
    let action = args[2].to_lowercase();

    match action.as_str() {
        "attach" => println!("{} attach", session_id),
        "detach" => println!("{} detach", session_id),
        "get" => println!("get"),
        _ => panic!("Invalid action selected, only <attach/detach/get> supported")
    }

    println!("Connected to the db");
    return Ok(());
}
