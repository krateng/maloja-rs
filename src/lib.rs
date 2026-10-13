use colored::Colorize;
use log::info;
use crate::configuration::logging::{display_path, display_url};

mod api;
mod configuration;
mod database;
mod entity;
mod server;
#[cfg(test)]
mod tests;
mod timeranges;
mod uri;


pub async fn setup(headless: bool) -> Result<(), Box<dyn std::error::Error>> {
    let env_file = if cfg!(test) { ".env.test" } else { ".env" };
    dotenvy::from_filename(env_file).ok();

    // just get the config object to start to make sure we get those errors at the start
    let _ = &configuration::CONFIG;

    configuration::logging::setup_logger()?;
    if !headless {
        debug_info();
    }

    // create files
    info!("Creating local files...");
    configuration::create_config_template()?;

    // DATABASE
    info!("Initializing database...");
    database::init_db().await?;

    Ok(())
}

fn debug_info() {
    println!("{} v{}", "Maloja".yellow(), env!("CARGO_PKG_VERSION"));
    let folders = &configuration::FOLDERS;
    let conf = &configuration::CONFIG;
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