use anyhow::{anyhow, bail, Result};
use dirs;
use session_tracker::SessionTracker;
use std::{env, path::PathBuf};
use tokio::fs;
mod session_tracker;

pub async fn run() -> Result<()> {
    // Parse CLI args
    let args: Vec<String> = env::args().collect();
    let action = Actions::new(&args)?;

    let db_url = get_db_url().await?;
    let tracker = SessionTracker::new(db_url.as_str()).await?;

    match action {
        Actions::Reset => tracker.clear_all_sessions().await?,
        Actions::Detach => tracker.detach_from_all_sessions().await?,
        Actions::GetAll => tracker.print_all_sessions_total_attached_time().await?,
        Actions::Attach(s) => tracker.attach_to_session(s.as_str()).await?,
        Actions::Gets(s) => println!(
            "Total Attached Time: {} s",
            tracker
                .get_total_session_time_in_seconds(s.as_str())
                .await?
        ),
        Actions::Geth(s) => println!(
            "Total Attached Time: {} s",
            tracker.get_total_session_time_in_hours(s.as_str()).await?
        ),
    }

    Ok(())
}

/** Prepares the dir for storing the db file and provides the URL to it */
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

/** Enum representing all the ways in which the program can get called */
enum Actions {
    Detach,
    Reset,
    GetAll,
    Attach(String),
    Gets(String), // get seconds
    Geth(String), // get hours
}

impl Actions {
    /** Determine the action based on CLI args */
    pub fn new(args: &[String]) -> Result<Self> {
        const USAGE_MESSAGE: &str =
            "Usage: tmux-time-tracker <action: attach/detach/gets/geth> [session_name]";

        let action = args.get(1).ok_or(anyhow!(USAGE_MESSAGE))?;
        match action.as_str() {
            "detach" => Ok(Actions::Detach),
            "reset" => Ok(Actions::Reset),
            "getall" => Ok(Actions::GetAll),
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
