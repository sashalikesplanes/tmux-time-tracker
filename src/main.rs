use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    match tmux_time_tracker::run().await {
        Ok(_) => Ok(()),
        Err(err) => { 
            log::error!("Error executing tmux-time-tracker {err}");
            Err(err)
        }
    }
}
