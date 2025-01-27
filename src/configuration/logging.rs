use chrono::Utc;
use colored::{ColoredString, Colorize};
use fern;
use std::path::PathBuf;

use crate::configuration::FOLDERS;

fn get_logfile_path() -> PathBuf {
    FOLDERS.logs.join("maloja.log")
}

pub fn setup_logger() -> Result<(), fern::InitError> {
    let logfile = match fern::log_file(get_logfile_path()) {
        Ok(logfile) => logfile,
        Err(e) => {
            println!(
                "Cannot set up log file {}: {}",
                display_path(&get_logfile_path()),
                e
            );
            panic!("Failed to setup logging");
        }
    };
    let dis = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                Utc::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(logfile)
        .apply()?;
    Ok(())
}

// Define common colors when various types of info are logged
pub fn display_path(path: &PathBuf) -> ColoredString {
    ColoredString::from(path.to_string_lossy().to_string()).bright_yellow()
}
pub fn display_envvar(var: &str) -> ColoredString {
    ColoredString::from(var).bright_purple()
}

pub fn display_url(var: &str) -> ColoredString {
    ColoredString::from(var).bright_blue()
}
