use colored::Colorize;
use log::info;
pub use crate::configuration::app_state::AppState;
use crate::configuration::config::create_config_template;
use crate::configuration::logging::{display_path, display_url};

mod api;
mod configuration;
mod database;
mod entity;
pub mod server;
#[cfg(test)]
mod tests;
mod timeranges;
mod uri;


pub async fn setup(headless: bool, state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let env_file = if cfg!(test) { ".env.test" } else { ".env" };
    dotenvy::from_filename(env_file).ok();
    // we allow an env file to set custom folders for tests,
    // but per default we will use a ram directory


    configuration::logging::setup_logger(state)?;
    if !headless {
        debug_info(state);
    }

    // create files
    info!("Creating local files...");
    create_config_template(state)?;

    // DATABASE
    info!("Initializing database...");
    database::init_db(state).await?;

    Ok(())
}

fn debug_info(state: &AppState) {
    println!("{} v{}", "Maloja".yellow(), env!("CARGO_PKG_VERSION"));
    let conf = &state.config;
    let folders = &state.folders;
    println!("Data directory:   {}", display_path(&folders.data));
    println!("Config directory: {}", display_path(&folders.config));
    println!("Log directory:    {}", display_path(&folders.logs));
    println!(
        "Bind:             {}",
        display_url(format!("{}:{}", conf.bind_address, conf.port).as_str())
    );
    println!();
    println!();
}
