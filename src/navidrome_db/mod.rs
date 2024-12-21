use std::collections::HashMap;

use rusqlite::Connection;
use structs::{NavidromeFile, NavidromePlaylist};

use crate::ErrorSink;

pub mod structs;

pub struct NavidromeDatabase {
    pub connection: Option<Connection>,
    pub user: String,
    pub user_id: String,
    pub media_files: HashMap<String, NavidromeFile>,
    pub playlists: HashMap<String, NavidromePlaylist>,
}

impl Drop for NavidromeDatabase {
    fn drop(&mut self) {
        if self.connection.is_some() {
            let taken = std::mem::take(&mut self.connection);
            let _ = taken.unwrap().close();
        }
    }
}

pub fn user_id(connection: &Connection, user: &str) -> Result<String, rusqlite::Error> {
    let mut stmt = connection.prepare(
        "
SELECT id, user_name
FROM user
WHERE user_name = :user",
    )?;
    let mut rows = stmt.query(&[(":user", user)])?;
    if let Some(row) = rows.next()? {
        return row.get("id");
    }
    Err(rusqlite::Error::QueryReturnedNoRows)
}

#[allow(dead_code)]
impl NavidromeDatabase {
    pub fn new(connection: rusqlite::Connection, user: &str) -> Result<Self, rusqlite::Error> {
        let user_id = user_id(&connection, user)?;
        let db = NavidromeDatabase {
            connection: Some(connection),
            user: user.to_owned(),
            user_id,
            media_files: HashMap::<String, NavidromeFile>::default(),
            playlists: HashMap::<String, NavidromePlaylist>::default(),
        };
        Ok(db)
    }

    pub fn import_media_files(&mut self) -> Result<(), ErrorSink> {
        if let Some(connection) = &self.connection {
            let query_string = "
SELECT id, artist_id, album_id, path, title, album, artist, album_artist, track_number, year
FROM media_file";

            let mut stmt = connection.prepare(query_string)?;
            let mut rows = stmt.raw_query();
            while let Some(row) = rows.next()? {
                let mut media_file = NavidromeFile {
                    id: row.get("id")?,
                    path: row.get("path")?,
                    title: row.get("title")?,
                    album: row.get("album")?,
                    artist: row.get("artist")?,
                    albumId: row.get("album_id").unwrap_or(None),
                    year: row.get("year")?,
                    artistId: row.get("artist_id").unwrap_or(None),
                    track: row.get("track_number").unwrap_or_default(),

                    playCount: 0,
                    comment: None,
                    played: None,
                    rating: 0,
                    starred: false,
                };

                let query_annotation = "
SELECT user_id, item_id, item_type, play_count, play_date, rating, starred, starred_at
FROM annotation
WHERE user_id = :user_id AND item_id = :item_id";
                let mut stmt = connection.prepare(query_annotation)?;
                let mut rows =
                    stmt.query(&[(":user_id", &self.user_id), (":item_id", &media_file.id)])?;
                if let Some(annotation) = rows.next()? {
                    media_file.comment = annotation.get("comment").unwrap_or(None);
                    media_file.playCount = annotation.get("play_count")?;
                    media_file.played = annotation.get("play_date").unwrap_or(None);
                    media_file.rating = annotation.get("rating").unwrap_or(0);
                    media_file.starred = annotation.get("starred_at").unwrap_or(false);
                }

                self.media_files
                    .insert(media_file.id.to_owned(), media_file);
            }
        } else {
            return Err(ErrorSink::NoDBConnection);
        }
        Ok(())
    }

    pub fn import_playlists(&mut self) -> Result<(), ErrorSink> {
        if let Some(connection) = &self.connection {
            let playlist_query = "
SELECT id, name, owner_id, comment, created_at, owner_id, song_count, rules
FROM playlist
WHERE owner_id = :owner_id";
            let mut playlist_statement = connection.prepare(playlist_query)?;
            let mut playlist_rows = playlist_statement.query(&[(":owner_id", &self.user_id)])?;
            while let Some(playlist) = playlist_rows.next()? {
                let smart: Result<Option<bool>, rusqlite::Error> = playlist.get("rules");
                if let Ok(None) = smart {
                    let playlist_id = playlist.get("id")?;

                    let tracks_query = "
SELECT id, playlist_id, media_file_id
FROM playlist_tracks
WHERE playlist_id = :playlist_id";

                    let mut track_statement = connection.prepare(tracks_query)?;
                    let track_parameters = [(":playlist_id", &playlist_id)];
                    let mut tracks = track_statement.query(&track_parameters)?;

                    let mut raw_tracks = Vec::<(usize, String)>::default();
                    while let Some(row) = tracks.next()? {
                        raw_tracks.push((row.get("id")?, row.get("media_file_id")?));
                    }
                    raw_tracks.sort_by(|a, b| std::cmp::Ord::cmp(&a.0, &b.0));

                    let playlist = NavidromePlaylist {
                        id: playlist_id,
                        name: playlist.get("name")?,
                        comment: playlist.get("comment").unwrap_or_default(),
                        created: playlist.get("created_at")?,
                        owner: self.user.to_owned(),
                        songCount: playlist.get("song_count")?,
                        track_ids: raw_tracks.into_iter().map(|(_, id)| id).collect(),
                    };
                    self.playlists.insert(playlist.id.to_owned(), playlist);
                } else {
                    let smart_name: String = playlist.get("name")?;
                    log::debug!("Skipped smart playlist: {}", smart_name);
                }
            }
            Ok(())
        } else {
            Err(ErrorSink::NoDBConnection)
        }
    }

    pub fn display_playlists(&self) {
        for playlist in self.playlists.values() {
            println!("{}", playlist.name);
            if playlist.name != "全て" {
                for (index, media_file_id) in playlist.track_ids.iter().enumerate() {
                    let mf = self.media_files.get(media_file_id).unwrap();
                    println!("\t{index} - {}", mf.one_line());
                }
            }
        }
    }
}
