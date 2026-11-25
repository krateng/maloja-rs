#![forbid(unsafe_code)]

use log::info;
use std::io;
use std::io::Write;
use colored::Colorize;
use tokio::time::{sleep, Duration};
use maloja_rs::{setup, AppState, server, PasswordState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!();

    let state = AppState::new(None).await?;

    setup(false, &state).await?;

    match state.db.set_initial_password().await? {
        PasswordState::AllGood => {},
        PasswordState::AppGeneratedStillSet => {
            println!("⚠️ {}️", "You are still using your auto-generated password.".bright_red().on_bright_white());
            println!("⚠️ {}️", "Please change it as soon as possible.".bright_red().on_bright_white());
        }
        PasswordState::NewlyGenerated(new_pw) => {
            println!("Initial password: {}", new_pw.bright_yellow().on_purple());
            println!("⚠️ {}️", "Please change your password as soon as possible.".bright_red().on_bright_white());
        }

    }

    // SERVER
    info!("Starting up server...");
    server::run_server(&state).await?;

    info!("Shutting down...");
    sleep(Duration::from_secs(2)).await;
    io::stdout().flush()?;

    Ok(())
}


