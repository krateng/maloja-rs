#![forbid(unsafe_code)]


mod entity;
mod database;
mod configuration;
mod api;
mod server;

use std::io;
use std::io::Write;
use tokio::time::{sleep, Duration};
use colored::Colorize;
use log::{info, debug, warn};
use crate::configuration::logging::{display_path, display_url};

#[tokio::main]
async fn main() {
    println!();

    // SETUP
    // just get the config object to start to make sure we get those errors at the start
    let _ = &configuration::CONFIG;

    configuration::logging::setup_logger().unwrap();
    debug_info();

    // create files
    info!("Creating local files...");
    configuration::create_config_template().unwrap();

    // DATABASE
    info!("Initializing database...");
    database::init_db().await;

    // SERVER
    server::run_server().await;

    sleep(Duration::from_secs(10)).await;
    io::stdout().flush().unwrap();
}

fn debug_info() {
    println!("{} v{}", "Maloja".yellow(), env!("CARGO_PKG_VERSION"));
    let folders = &configuration::FOLDERS;
    let conf = &configuration::CONFIG;
    println!("Data directory:   {}", display_path(&folders.data));
    println!("Config directory: {}", display_path(&folders.config));
    println!("Log directory:    {}", display_path(&folders.logs));
    println!("Bind:             {}", display_url(format!("{}:{}", conf.bind_address, conf.port).as_str()));
    println!();
    println!();
}
