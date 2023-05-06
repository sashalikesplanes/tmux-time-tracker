use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tmux_time_tracker::run().await?;
    Ok(())
}
