#![forbid(unsafe_code)]

mod api;
mod configuration;
mod database;
mod entity;
mod server;
mod timeranges;
mod uri;

use crate::configuration::logging::{display_path, display_url};
use colored::Colorize;
use log::info;
use std::io;
use std::io::Write;
use tokio::time::{sleep, Duration};
use maloja_rs::setup;

#[tokio::main]
async fn main() {
    println!();

    setup(false).await;

    // SERVER
    info!("Starting up server...");
    server::run_server().await;

    info!("Shutting down...");
    sleep(Duration::from_secs(2)).await;
    io::stdout().flush().unwrap();
}


