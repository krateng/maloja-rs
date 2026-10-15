use std::fs::File;
use std::io;
use std::io::Write;
use std::net::IpAddr;
use std::path::PathBuf;
use confique::{toml, Config};
use crate::configuration::app_state::{AppState, ApplicationFolders};

#[derive(Config, Debug, Clone)]
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
    /// Timezone
    #[config(default = "UTC")]
    pub timezone: String, //TODO enum? Tz?
    /// What artist string should be shown for tracks or albums with no artists
    #[config(default = "Various Artists")]
    pub default_artist: String,
    /// How to format dates
    #[config(default = "%d. %b %Y %I:%M %p")]
    pub time_format: String,
}

pub fn get_config_file_path(folders: &ApplicationFolders) -> PathBuf {
    let path = folders.config.join("maloja.toml");
    path
}

pub fn create_config_template(state: &AppState) -> io::Result<()> {
    let example_config = toml::template::<MalojaConfig>(Default::default());
    let file_path = get_config_file_path(&state.folders);
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
