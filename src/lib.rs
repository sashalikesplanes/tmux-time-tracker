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

/// Executes the program
pub async fn run() -> Result<()> {
    let home_path = get_home_path().await?;
    setup_logger(&home_path)?;

    // Parse CLI args
    let args: Vec<String> = env::args().collect();
    let tracker = SessionTracker::new(&home_path).await?;

    const USAGE_MESSAGE: &str =
        "Usage: tmux-time-tracker <action: attach/detach/gets/geth> [session_name]";

    let action = args.get(1).ok_or(anyhow!(USAGE_MESSAGE))?;
    let session = args.get(2).ok_or(anyhow!(USAGE_MESSAGE));
    match (action.as_str(), session) {
        ("detach", _) => tracker.detach_from_all_sessions().await?,
        ("attach", Ok(s)) => tracker.attach_to_session(s.as_str()).await?,
        ("gets", Ok(s)) => println!(
            "Total Attached Time: {} s",
            tracker
                .get_today_session_time_in_seconds(s.as_str())
                .await?
        ),
        ("geth", Ok(s)) => println!(
            "Total Attached Time: {} h",
            tracker.get_today_session_time_in_hours(s.as_str()).await?
        ),
        _ => bail!(USAGE_MESSAGE),
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

    Ok(config_dir
        .to_str()
        .ok_or(anyhow!("Failed to parse path into a &str"))?
        .to_owned())
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
