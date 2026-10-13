#![forbid(unsafe_code)]

mod api;
mod configuration;
mod database;
mod entity;
mod server;
mod timeranges;
mod uri;

use log::info;
use std::io;
use std::io::Write;
use tokio::time::{sleep, Duration};
use maloja_rs::setup;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!();

    setup(false).await?;

    // SERVER
    info!("Starting up server...");
    server::run_server().await?;

    info!("Shutting down...");
    sleep(Duration::from_secs(2)).await;
    io::stdout().flush()?;

    Ok(())
}


