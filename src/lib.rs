//! # tmux_time_tracker
//!
//! A CLI program meant to hook into Tmux
//! Allows for tracking the time spent attached to Tmux session
use anyhow::{anyhow, bail, Result};
use dirs;
use sqlx::types::chrono;
use std::{env, path::PathBuf, process};
use tokio::fs;
pub mod session_tracker;
pub use session_tracker::SessionTracker;

const MESSAGE_TIMEOUT: u32 = 1_000;

/// Executes the program
pub async fn run() -> Result<()> {
    let home_path = get_home_path().await?;
    setup_logger(&home_path)?;

    // Parse CLI args
    let args: Vec<String> = env::args().collect();
    let tracker = SessionTracker::new(&home_path).await?;

    let action = args.get(1).ok_or(anyhow!("Missing first argument"))?;
    let session = args.get(2).ok_or(anyhow!("Missing second argument"));

    match (action.as_str(), session) {
        ("detached", _) => {
            log::info!("START - detached");
            // Nothing to display
            tracker.detach_from_all_sessions().await?;
        }
        ("attached", Ok(session)) => {
            log::info!("START - attached");
            attach_to_session_and_display_session_time(&tracker, session).await?;
        }
        ("changed", Ok(session)) => {
            log::info!("START - changed");
            tracker.detach_from_all_sessions().await?;
            attach_to_session_and_display_session_time(&tracker, session).await?;
        }
        _ => bail!("Could not pattern match on first and second arg"),
    }

    log::info!("tmux-time-tracker ran succesfully for {:?}", action);

    Ok(())
}

async fn attach_to_session_and_display_session_time(
    tracker: &SessionTracker,
    session: &str,
) -> Result<()> {
    tracker.attach_to_session(session).await?;

    let daily_session_time = tracker.get_daily_session_time_in_hours(session).await?;
    let weekly_session_time = tracker.get_weekly_session_time_in_hours(session).await?;
    display_tmux_msg(
        &format!(
            "Attached to: {} today for {}h, this week for {}h",
            session,
            daily_session_time,
            weekly_session_time
        ),
        MESSAGE_TIMEOUT,
    )?;

    Ok(())
}

fn display_tmux_msg(msg: &str, timeout: u32) -> Result<()> {
    process::Command::new("zsh")
        .arg("-c")
        .arg(format!(r#"tmux display-message -d {} "{}" "#, timeout, msg))
        .output()
        .or_else(|_| {
            bail!("Failed to run tmux");
        })?;

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
