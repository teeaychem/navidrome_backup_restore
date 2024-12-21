//! Structs
//!
//! These correspond to OpenSubsonic results, for easy JSON deserialisation.

use serde::{de, Deserialize, Serialize};

/// https://opensubsonic.netlify.app/docs/endpoints/getsong/
#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct NavidromeFile {
    pub title: String,
    pub id: String,
    pub path: String,

    pub album: Option<String>,
    pub albumId: Option<String>,
    pub artist: Option<String>,
    pub artistId: Option<String>,

    pub comment: Option<String>,

    pub track: Option<usize>,
    pub year: Option<usize>,

    #[serde(default)]
    pub playCount: usize,
    pub played: Option<String>,
    #[serde(default)]
    pub rating: usize,
    #[serde(deserialize_with = "some_string_to_bool", default)]
    pub starred: bool,
}

fn some_string_to_bool<'d, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: de::Deserializer<'d>,
{
    let maybe: Option<String> = de::Deserialize::deserialize(deserializer)?;

    match maybe {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

impl NavidromeFile {
    pub fn one_line(&self) -> String {
        format!(
            "{}  --  {}  --  {}",
            &self.title,
            &self.artist.clone().unwrap_or("[No Artist]".to_string()),
            &self.album.clone().unwrap_or("[No Album]".to_string()),
        )
    }
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct NavidromePlaylist {
    pub comment: Option<String>,
    pub created: String,
    pub id: String,
    pub name: String,
    pub owner: String,
    pub songCount: usize,
    #[serde(default)]
    pub track_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Annotation {
    pub item_id: String,
    pub item_type: String,
    pub play_count: usize,
    pub play_date: Option<String>,
    pub rating: Option<usize>,
    pub starred: bool,
    pub starred_at: Option<String>,
}
