use std::collections::HashMap;

use reqwest::Response;
use serde_json::Value;

use crate::{
    config::Config,
    navidrome_db::structs::{NavidromeFile, NavidromePlaylist},
    ErrorSink,
};

pub struct NavidromeClient {
    pub client: reqwest::Client,
    pub address: String,
    pub user: String,
    pub password: String,
    pub protocol_version: String,
    pub client_application: String,
}

#[allow(dead_code)]
impl NavidromeClient {
    pub fn new(config: &Config) -> Self {
        NavidromeClient {
            client: reqwest::Client::new(),
            address: config.navidrome_url.to_owned(),
            user: config.user.to_owned(),
            password: config.password.to_owned(),
            protocol_version: "1.16.1".to_owned(),
            client_application: "bckup".to_owned(),
        }
    }

    pub fn base_parameters_hash_map(&self) -> HashMap<&str, &str> {
        HashMap::from([
            ("u", self.user.as_str()),
            ("p", self.password.as_str()),
            ("v", self.protocol_version.as_str()),
            ("c", self.client_application.as_str()),
            ("f", "json"),
        ])
    }

    pub fn base_parameters_vec(&self) -> Vec<(&str, &str)> {
        Vec::from([
            ("u", self.user.as_str()),
            ("p", self.password.as_str()),
            ("v", self.protocol_version.as_str()),
            ("c", self.client_application.as_str()),
            ("f", "json"),
        ])
    }

    pub async fn ping(&self) -> Result<String, reqwest::Error> {
        self.client
            .post(format!("{}/rest/ping.view?", self.address))
            .form(&self.base_parameters_hash_map())
            .send()
            .await?
            .text()
            .await
    }

    pub async fn validate_or_exit_via_ping(&self) -> Result<(), ErrorSink> {
        let ping = self.ping().await?;
        let ping_value: serde_json::Value = serde_json::from_str(&ping)?;

        let status: String = ping_value
            .get("subsonic-response")
            .ok_or(ErrorSink::MissingSubsonicResponse)?
            .get("status")
            .ok_or(ErrorSink::MissingSearchResult)?
            .to_string();

        if status == "\"failed\"" {
            let reason: String = ping_value
                .get("subsonic-response")
                .ok_or(ErrorSink::MissingSubsonicResponse)?
                .get("error")
                .ok_or(ErrorSink::MissingSearchResult)?
                .get("message")
                .ok_or(ErrorSink::MissingSearchResult)?
                .to_string();
            println!("Authentication failed with message:\n{reason}");
            std::process::exit(1);
        }

        Ok(())
    }

    pub async fn get_now_playing(&self) -> Result<String, reqwest::Error> {
        self.client
            .post(format!("{}/rest/getNowPlaying.view?", self.address))
            .form(&self.base_parameters_hash_map())
            .send()
            .await?
            .text()
            .await
    }

    pub async fn get_playlists(&self) -> Result<Vec<NavidromePlaylist>, ErrorSink> {
        let mut params = self.base_parameters_hash_map();
        params.insert("username", &self.user);

        let playlists_text = self
            .client
            .post(format!("{}/rest/getPlaylists.view?", self.address))
            .form(&params)
            .send()
            .await?
            .text()
            .await?;

        let playlist_result_value = serde_json::from_str::<serde_json::Value>(&playlists_text)?;

        let playlist_values = playlist_result_value
            .get("subsonic-response")
            .ok_or(ErrorSink::MissingSubsonicResponse)?
            .get("playlists")
            .ok_or(ErrorSink::MissingJSON(String::from("playlists")))?
            .get("playlist")
            .ok_or(ErrorSink::MissingJSONPlaylist)?
            .as_array()
            .ok_or(ErrorSink::UnexpectedJSON)?;

        let client_playlists = playlist_values
            .iter()
            .map(|value| serde_json::from_value(value.clone()).unwrap())
            .collect();

        Ok(client_playlists)
    }

    pub async fn search_three(
        &self,
        query: &str,
        artist_count: usize,
        album_count: usize,
        song_count: usize,
    ) -> Result<Response, reqwest::Error> {
        let mut params = self.base_parameters_hash_map();

        let artist_count = artist_count.to_string();
        let album_count = album_count.to_string();
        let song_count = song_count.to_string();

        params.insert("query", query);
        params.insert("artistCount", &artist_count);
        params.insert("albumCount", &album_count);
        params.insert("songCount", &song_count);

        self.client
            .post(format!("{}/rest/search3.view?", self.address))
            .form(&params)
            .send()
            .await
    }

    pub async fn search_tracks(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<NavidromeFile>, ErrorSink> {
        let search_value: serde_json::Value =
            serde_json::from_str(&self.search_three(query, 0, 0, limit).await?.text().await?)?;

        let search_values: &Value = search_value
            .get("subsonic-response")
            .ok_or(ErrorSink::MissingSubsonicResponse)?
            .get("searchResult3")
            .ok_or(ErrorSink::MissingSearchResult)?;

        let search_values: &Vec<Value> = search_values
            .get("song")
            .ok_or(ErrorSink::MissingJSONSong)?
            .as_array()
            .ok_or(ErrorSink::UnexpectedJSON)?;

        let mut found_tracks: Vec<NavidromeFile> = Vec::default();

        for track_json in search_values {
            found_tracks.push(serde_json::from_value(track_json.to_owned())?);
        }

        Ok(found_tracks)
    }

    pub async fn get_playlist(&self, id: &str) -> Result<NavidromePlaylist, ErrorSink> {
        let mut params = self.base_parameters_hash_map();
        params.insert("username", &self.user);
        params.insert("id", id);

        let text = self
            .client
            .post(format!("{}/rest/getPlaylist.view?", self.address))
            .form(&params)
            .send()
            .await?
            .text()
            .await?;

        let playlist_result_value: Value = serde_json::from_str(&text)?;

        let playlist_value: &Value = playlist_result_value
            .get("subsonic-response")
            .ok_or(ErrorSink::MissingSubsonicResponse)?
            .get("playlist")
            .ok_or(ErrorSink::MissingJSONPlaylist)?;

        let mut playlist: NavidromePlaylist = serde_json::from_value(playlist_value.clone())?;

        let track_values = playlist_value
            .get("entry")
            .ok_or(ErrorSink::MissingJSON(String::from("entry")))?
            .as_array()
            .ok_or(ErrorSink::UnexpectedJSON)?;
        let mut track_ids = vec![];
        for t in track_values {
            let id: String = serde_json::from_value(
                t.get("id")
                    .ok_or(ErrorSink::MissingJSON(String::from("id")))?
                    .to_owned(),
            )?;
            track_ids.push(id);
        }

        playlist.track_ids = track_ids;

        Ok(playlist)
    }

    pub async fn get_song(&self, id: &str) -> Result<NavidromeFile, ErrorSink> {
        let mut params = self.base_parameters_hash_map();
        params.insert("username", &self.user);
        params.insert("id", id);

        let text = self
            .client
            .post(format!("{}/rest/getSong.view?", self.address))
            .form(&params)
            .send()
            .await?
            .text()
            .await?;

        let song_result_value: Value = serde_json::from_str(&text)?;

        let song_value: &Value = song_result_value
            .get("subsonic-response")
            .ok_or(ErrorSink::MissingSubsonicResponse)?
            .get("song")
            .ok_or(ErrorSink::MissingJSONSong)?;

        let song: NavidromeFile = serde_json::from_value(song_value.clone())?;

        Ok(song)
    }

    pub async fn scrobble(
        &self,
        id: &str,
        since_epoch: Option<i64>,
    ) -> Result<Response, reqwest::Error> {
        let mut params = self.base_parameters_hash_map();
        params.insert("id", id);
        let time_str;
        if let Some(time) = since_epoch {
            time_str = time.to_string();
            params.insert("time", &time_str);
        }
        self.client
            .post(format!("{}/rest/scrobble.view?", self.address))
            .form(&params)
            .send()
            .await
    }

    pub async fn set_rating(&self, id: &str, rating: usize) -> Result<Response, reqwest::Error> {
        let mut params = self.base_parameters_hash_map();
        params.insert("id", id);
        let rating_string = rating.to_string();
        params.insert("rating", &rating_string);
        self.client
            .post(format!("{}/rest/setRating.view?", self.address))
            .form(&params)
            .send()
            .await
    }

    pub async fn star(&self, id: &str) -> Result<Response, reqwest::Error> {
        let mut params = self.base_parameters_hash_map();
        params.insert("id", id);
        self.client
            .post(format!("{}/rest/star.view?", self.address))
            .form(&params)
            .send()
            .await
    }

    pub async fn unstar(&self, id: &str) -> Result<Response, reqwest::Error> {
        let mut params = self.base_parameters_hash_map();
        params.insert("id", id);
        self.client
            .post(format!("{}/rest/unstar.view?", self.address))
            .form(&params)
            .send()
            .await
    }

    pub async fn update_playlist(
        &self,
        id: &str,
        track_ids: &[String],
    ) -> Result<Response, reqwest::Error> {
        let mut params = self.base_parameters_vec();
        let playlist_id = "playlistId";
        let song_id = "songId";
        params.push((playlist_id, id));
        for id in track_ids {
            params.push((song_id, id));
        }
        self.client
            .post(format!("{}/rest/createPlaylist.view?", self.address))
            .form(&params)
            .send()
            .await
    }
}
