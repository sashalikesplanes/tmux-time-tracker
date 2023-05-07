//! # tmux_time_tracker
//!
//! A CLI program meant to hook into Tmux
//! Allows for tracking the time spent attached to Tmux session
use anyhow::{anyhow, bail, Result};
use dirs;
use sqlx::types::chrono;
use std::{env, path::PathBuf};
use tokio::fs;
pub mod session_tracker;
pub use session_tracker::SessionTracker;

/// Represents all valid actions with which the program can get invoked
#[derive(Debug)]
enum Actions {
    Detach,
    Attach(String),
    Gets(String), // get seconds
    Geth(String), // get hours
}

/// Executes the program
pub async fn run() -> Result<()> {
    let home_path = get_home_path().await?;
    setup_logger(&home_path)?;

    // Parse CLI args
    let args: Vec<String> = env::args().collect();
    let action = Actions::new(&args)?;

    let tracker = SessionTracker::new(&home_path).await?;

    match &action {
        Actions::Detach => tracker.detach_from_all_sessions().await?,
        Actions::Attach(s) => tracker.attach_to_session(s.as_str()).await?,
        Actions::Gets(s) => println!(
            "Total Attached Time: {} s",
            tracker
                .get_today_session_time_in_seconds(s.as_str())
                .await?
        ),
        Actions::Geth(s) => println!(
            "Total Attached Time: {} h",
            tracker.get_today_session_time_in_hours(s.as_str()).await?
        ),
    }

    log::info!("tmux-time-tracker ran succesfully for {:?}", action);
    Ok(())
}

/// Ensures that the home directory is setup and returns it
async fn get_home_path() -> Result<String> {
    // The env var is used for the dev database
    let home_dir = dirs::home_dir().expect("Home directory should be available");
    let mut config_dir = PathBuf::from(home_dir);
    config_dir.push(".config/tmux-time-tracker");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).await?;
    }

    Ok(config_dir.to_str().ok_or(anyhow!("Failed to parse path into a &str"))?.to_owned())
}

fn setup_logger(home_path: &str) -> Result<()> {
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(fern::log_file(format!("{home_path}/output.log"))?)
        .apply()?;

    Ok(())
}

/// Determines the current Action based on CLI arguments
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
