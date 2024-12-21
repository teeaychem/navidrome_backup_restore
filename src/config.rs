use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub navidrome_url: String,
    pub navidrome_db_in_use: String,
    pub navidrome_db_to_restore_from: String,
    pub backup_dir: String,
    pub user: String,
    pub password: String,
    pub similar_track_search_limit: usize,

    pub backup_stdout: bool,
    pub backup_pages_per_step: i32,
    pub backup_pause_between_pages: u64,
}

impl Config {
    pub fn from_file(path: &PathBuf) -> Config {
        let file_string = match std::fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(_) => {
                println!("Unable to open config");
                std::process::exit(1);
            }
        };
        let config: Config = match toml::from_str(&file_string) {
            Ok(toml) => toml,
            Err(e) => {
                println!(
                    "Unable to read config.
{e:?}"
                );
                std::process::exit(1);
            }
        };
        config
    }
}
