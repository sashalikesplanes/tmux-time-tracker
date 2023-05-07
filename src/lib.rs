//! # tmux_time_tracker
//!
//! A CLI program meant to hook into Tmux
//! Allows for tracking the time spent attached to Tmux session
use anyhow::{anyhow, bail, Result};
use dirs;
use std::{env, path::PathBuf};
use tokio::fs;
pub mod session_tracker;
pub use session_tracker::SessionTracker;

/// Represents all valid actions with which the program can get invoked
enum Actions {
    Detach,
    Attach(String),
    Gets(String), // get seconds
    Geth(String), // get hours
}

/// Executes the program
pub async fn run() -> Result<()> {
    // Parse CLI args
    let args: Vec<String> = env::args().collect();
    let action = Actions::new(&args)?;

    let db_url = get_db_url().await?;
    let tracker = SessionTracker::new(db_url.as_str()).await?;

    match action {
        Actions::Detach => tracker.detach_from_all_sessions().await?,
        Actions::Attach(s) => tracker.attach_to_session(s.as_str()).await?,
        Actions::Gets(s) => println!(
            "Total Attached Time: {} s",
            tracker
                .get_total_session_time_in_seconds(s.as_str())
                .await?
        ),
        Actions::Geth(s) => println!(
            "Total Attached Time: {} h",
            tracker.get_total_session_time_in_hours(s.as_str()).await?
        ),
    }

    Ok(())
}

/// Ensures that the db storage location is available
/// Returns the db connection URL
async fn get_db_url() -> Result<String> {
    // The env var is used for the dev database
    let home_dir = dirs::home_dir().expect("Home directory should be available");
    let mut config_dir = PathBuf::from(home_dir);
    config_dir.push(".config/tmux-time-tracker");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).await?;
    }

    Ok("sqlite://".to_owned() + config_dir.to_str().expect("Config dir should exist") + "/tmux.db")
}


impl Actions {
    /// Determines the current Action based on CLI arguments
    ///
    /// # Errors
    /// - Anyhow Error if wrong CLI arguments provided
    pub fn new(args: &[String]) -> Result<Self> {
        const USAGE_MESSAGE: &str =
            "Usage: tmux-time-tracker <action: attach/detach/gets/geth> [session_name]";

        let action = args.get(1).ok_or(anyhow!(USAGE_MESSAGE))?;
        match action.as_str() {
            "detach" => Ok(Actions::Detach),
            _ => {
                let session = args.get(2).ok_or(anyhow!(USAGE_MESSAGE))?;
                match action.as_str() {
                    "attach" => Ok(Actions::Attach(session.to_owned())),
                    "gets" => Ok(Actions::Gets(session.to_owned())),
                    "geth" => Ok(Actions::Geth(session.to_owned())),
                    _ => bail!(USAGE_MESSAGE),
                }
            }
        }
    }
}
