pub mod logging;

use crate::configuration::logging::{display_envvar, display_path};
use colored::Colorize;
use confique::serde::Deserialize;
use confique::{toml, Config, Error};
use log::{error, info, warn};
use std::fs::{remove_file, File, OpenOptions};
use std::io::Write;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::{env, fs, io};

pub static FOLDERS: LazyLock<ApplicationFolders> = LazyLock::new(|| {
    let af = ApplicationFolders {
        // we set the defaults to posix standards, even though maloja is container-oriented first
        // the Containerfile overwrites these
        data: PathBuf::from(
            env::var("MALOJA_DATA_PATH").unwrap_or(String::from("/var/lib/maloja")),
        ),
        config: PathBuf::from(
            env::var("MALOJA_CONFIG_PATH").unwrap_or(String::from("/etc/maloja")),
        ),
        logs: PathBuf::from(env::var("MALOJA_LOG_PATH").unwrap_or(String::from("/var/log/maloja"))),
    };
    let mut success: bool = true;

    let folder_logic: [(&PathBuf, &str, &str, bool); 3] = [
        (&af.data, "MALOJA_DATA_PATH", "/var/lib/maloja", true),
        (&af.config, "MALOJA_CONFIG_PATH", "/etc/maloja", false),
        (&af.logs, "MALOJA_LOG_PATH", "/var/log/maloja", true),
    ];
    for (folder, envvar, _default, writable) in folder_logic.iter() {
        match fs::create_dir_all(folder) {
            Ok(_) => {
                if *writable {
                    let test_file = folder.join(".write_test");
                    let result = OpenOptions::new().write(true).create(true).open(&test_file);
                    match result {
                        Ok(_) => {
                            let _ = remove_file(test_file);
                        }
                        Err(e) => {
                            println!("No write access to {}: {}. Make sure to set the environment variable {} to a writable directory.", display_path(folder), e, display_envvar(envvar));
                            success = false;
                        }
                    }
                }
            }
            Err(e) => {
                // logging isn't setup yet
                println!("Failed to create {}: {}. Make sure to set the environment variable {} to a writable directory.", display_path(folder), e, display_envvar(envvar));
                success = false;
            }
        }
    }
    if !success {
        panic!("Failed to initialize application folders");
    }
    af
});

pub static CONFIG: LazyLock<MalojaConfig> = LazyLock::new(|| {
    MalojaConfig::from_file(get_config_file_path()).unwrap_or_else(|_| {
        // we just load the default values
        warn!("Configuration file loading failed");
        MalojaConfig::builder().load().unwrap()
    })
});

pub struct ApplicationFolders {
    pub data: PathBuf,
    pub config: PathBuf,
    pub logs: PathBuf,
}

#[derive(Config, Debug)]
pub struct MalojaConfig {
    /// Enable logging
    #[config(default = true)]
    pub logging: bool,
    /// Host address to listen on. :: for IPv6, 0.0.0.0 for IPv4
    #[config(default = "::")]
    pub bind_address: IpAddr,
    /// Port to listen on
    #[config(default = 42010)]
    pub port: u16,
    /// How many hours until third party providers are asked again for new images of artists, tracks or albums
    #[config(default = 1000)]
    pub image_cache_expire_positive: u16,
    /// How many hours until image fetch for entities without an image is attempted again
    #[config(default = 500)]
    pub image_cache_expire_negative: u16,
    /// How many scrobbles a track needs in order to be considered Diamond status
    #[config(default = 1000)]
    pub scrobbles_track_diamond: u16,
    /// How many scrobbles a track needs in order to be considered Platinum status
    #[config(default = 750)]
    pub scrobbles_track_platinum: u16,
    /// How many scrobbles a track needs in order to be considered Gold status
    #[config(default = 500)]
    pub scrobbles_track_gold: u16,
    /// How many scrobbles an album needs in order to be considered Diamond status
    #[config(default = 1500)]
    pub scrobbles_album_diamond: u16,
    /// How many scrobbles an album needs in order to be considered Platinum status
    #[config(default = 1000)]
    pub scrobbles_album_platinum: u16,
    /// How many scrobbles an album needs in order to be considered Gold status
    #[config(default = 500)]
    pub scrobbles_album_gold: u16,
    /// API Key for Last.fm
    #[config()]
    pub last_fm_api_key: Option<String>,
    /// API Secret for Last.fm
    #[config()]
    pub lastfm_api_secret: Option<String>,
    /// API SK for Last.fm
    #[config()]
    pub lastfm_api_sk: Option<String>,
    /// API ID for Spotify
    #[config()]
    pub spotify_api_id: Option<String>,
    /// API Secret for Spotify
    #[config()]
    pub spotify_api_secret: Option<String>,
    /// API Key for AudioDB
    #[config()]
    pub audiodb_api_key: Option<String>,
    /// Offset for Week Start. 0 is Sunday
    #[config(default = 0)]
    pub week_offset: u8,
    /// Timezone offset, 0 is UTC
    #[config(default = 0)]
    pub utc_offset: u8,
    /// What artist string should be shown for tracks or albums with no artists
    #[config(default = "Various Artists")]
    pub default_albumartist: String,
    /// How to format dates
    #[config(default = "%d. %b %Y %I:%M %p")]
    pub time_format: String,
}

fn get_config_file_path() -> PathBuf {
    let path = FOLDERS.config.join("maloja.toml");
    path
}

pub fn create_config_template() -> io::Result<()> {
    let example_config = toml::template::<MalojaConfig>(Default::default());
    let file_path = get_config_file_path();
    if file_path.exists() {
        return Ok(());
    }
    // TODO: should we maybe create a separate example file in case config gets updated later (and the existing user file doesnt have those keys)?
    match File::create(&file_path) {
        Ok(mut file) => {
            file.write_all(example_config.as_bytes())?;
            Ok(())
        }
        Err(_) => {
            // Read only config directory isn't an error
            Ok(())
        }
    }
}
