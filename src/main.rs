// #![allow(dead_code, non_snake_case, unused_imports, unused)]

use std::path::PathBuf;

use clap::{arg, command, value_parser};
use client::NavidromeClient;
use config::Config;

use navidrome_db::{structs::NavidromeFile, NavidromeDatabase};

use rusqlite::Connection;

use thiserror::Error;

mod backup;
mod client;
mod config;
mod input;
mod navidrome_db;
mod update;

#[derive(Error, Debug)]
pub enum ErrorSink {
    #[error("reqwest error")]
    Reqwest(#[from] reqwest::Error),
    #[error("json error")]
    SerdeJson(#[from] serde_json::Error),

    #[error("unexpected json")]
    UnexpectedJSON,
    #[error("sql error")]
    SQL(#[from] rusqlite::Error),

    #[error("no db connection")]
    NoDBConnection,

    #[error("missing json")]
    MissingJSON(String),
    #[error("missing song")]
    MissingJSONSong,
    #[error("missing playlist")]
    MissingJSONPlaylist,
    #[error("missing subsonic response")]
    MissingSubsonicResponse,
    #[error("missing search result")]
    MissingSearchResult,

    #[error("failed to handle backup path")]
    BackupPath,

    #[error("sqlite subprocess error")]
    IO(#[from] std::io::Error),

    #[error("todo!")]
    Todo,
}

#[tokio::main]
async fn main() {
    let matches = command!()
        .arg(arg!(-b --backup "Create a backup of the current navidrome database.").required(false)
        )
        .arg(arg!(-c --config <FILE> "A path to the config to use.").required(false).value_parser(value_parser!(PathBuf))
        )

        .arg(arg!(-r --restore "(Attempt to) restore metadata and playlists to the current navidrome instance using a backup."))
        .get_matches();

    let config = match matches.get_one::<PathBuf>("config") {
        Some(path) => Config::from_file(path),
        None => Config::from_file(&PathBuf::from("config.toml")),
    };

    if matches.get_flag("backup") {
        let source_string = &config.navidrome_db_in_use;
        let source_connection = match Connection::open(source_string) {
            Ok(c) => c,
            Err(e) => {
                println!(
                    "Failed to establish a connection to the backup destination.
{e:?}"
                );
                std::process::exit(1);
            }
        };

        match backup::create_backup(&source_connection, &config.backup_dir, &config) {
            Ok(_) => {}
            Err(e) => println!(
                "Failed to determine path for databse backup.
{e:?}"
            ),
        }
    }

    if matches.get_flag("restore") {
        println!("Establishing a connection with the current navidrome instance…");
        let nd_client = NavidromeClient::new(&config);
        match nd_client.validate_or_exit_via_ping().await {
            Ok(_) => log::debug!("Navidrome client authentication ok!"),
            Err(e) => {
                log::error!(
                    "Failed to authenticate.
{e:?}"
                );
                std::process::exit(1);
            }
        }

        let db_connection = match rusqlite::Connection::open(&config.navidrome_db_to_restore_from) {
            Ok(c) => c,
            Err(e) => {
                println!("Error reading backup database {e:?}");
                std::process::exit(2);
            }
        };

        println!(
            "Reading information from the backup navidrome database, this may take some time…"
        );
        let mut db = NavidromeDatabase::new(db_connection, &config.user).unwrap();
        db.import_media_files()
            .expect("Failed to import file from the backup database.");
        db.import_playlists()
            .expect("Failed to import playlists from the backup database.");

        match crate::update::ui::interface::update(&db, &nd_client, &config).await {
            Ok(_) => {}
            Err(e) => {
                println!(
                    "Ah heck, some issue when comparing the backup and current navidrome instances.
{e:?}."
                );
                std::process::exit(5);
            }
        };
    }

    std::process::exit(0);
}
