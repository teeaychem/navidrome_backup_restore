use crate::{
    client::NavidromeClient, config::Config, navidrome_db::structs::NavidromeFile, ErrorSink,
};

pub async fn id_match(client: &NavidromeClient, file: &NavidromeFile) -> Result<bool, ErrorSink> {
    match client.get_song(&file.id).await {
        Ok(_) => Ok(true),
        Err(ErrorSink::MissingJSONSong) => Ok(false),
        Err(e) => Err(e),
    }
}

pub async fn get_suggestions(
    client: &NavidromeClient,
    file: &NavidromeFile,
    query_artist: bool,
    query_album: bool,
    config: &Config,
) -> Result<Vec<String>, ErrorSink> {
    let mut query_string = file.title.clone();

    if query_artist {
        if let Some(artist) = &file.artist {
            query_string.push(' ');
            query_string.push_str(artist);
        }
    }

    if query_album {
        if let Some(album) = &file.album {
            query_string.push(' ');
            query_string.push_str(album);
        }
    }

    let refind = client
        .search_tracks(query_string.as_str(), config.similar_track_search_limit)
        .await;
    match refind {
        Ok(ok) => Ok(ok.iter().map(|f| f.id.to_owned()).collect()),
        Err(ErrorSink::MissingJSONSong) => Ok(Vec::default()),
        Err(e) => Err(e),
    }
}
