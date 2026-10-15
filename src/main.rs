#![forbid(unsafe_code)]

use log::info;
use std::io;
use std::io::Write;
use tokio::time::{sleep, Duration};
use maloja_rs::{setup, AppState, server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!();

    let state = AppState::new(None).await?;

    setup(false, &state).await?;

    // SERVER
    info!("Starting up server...");
    server::run_server(&state).await?;

    info!("Shutting down...");
    sleep(Duration::from_secs(2)).await;
    io::stdout().flush()?;

    Ok(())
}


