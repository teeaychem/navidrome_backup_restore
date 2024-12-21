use crossterm::style::Attribute;

use crate::{update::ui::constants::*, NavidromeFile};

pub fn display_missing(original: &NavidromeFile) -> String {
    let d_title = &original.title;
    let d_artist = &original.artist.clone().unwrap_or("[No Artist]".to_string());
    let d_album = &original.album.clone().unwrap_or("[No Album]".to_string());
    let d_track = match &original.track {
        Some(n) => n.to_string(),
        None => "[No Track]".to_string(),
    };
    let d_year = match &original.year {
        Some(y) => y.to_string(),
        None => "[No Year]".to_string(),
    };

    format!(
        "{VERTICAL_BAR:>SONG_PADDING$} {TITLE_FIELD_STR:<FIELD_PADDING$}: {d_title}
{VERTICAL_BAR:>SONG_PADDING$} {ARTIST_FIELD_STR:<FIELD_PADDING$}: {d_artist}
{VERTICAL_BAR:>SONG_PADDING$} {ALBUM_FIELD_STR:<FIELD_PADDING$}: {d_album}
{VERTICAL_BAR:>SONG_PADDING$} {TRACK_FIELD_STR:<FIELD_PADDING$}: {d_track}
{VERTICAL_BAR:>SONG_PADDING$} {YEAR_FIELD_STR:<FIELD_PADDING$}: {d_year}",
    )
}

pub fn candidate_string(
    n: Option<usize>,
    original: &NavidromeFile,
    candidate: &NavidromeFile,
) -> String {
    let d_title = comparison_string(&Some(&original.title), &Some(&candidate.title));
    let d_artist = comparison_string(&original.artist, &candidate.artist);
    let d_album = comparison_string(&original.album, &candidate.album);
    let d_track = comparison_string(&original.track, &candidate.track);
    let d_year = comparison_string(&original.year, &candidate.year);
    let d_playcount = {
        let original = &Some(&original.playCount);
        let candidate = &Some(&candidate.playCount);
        comparison_string(original, candidate)
    };
    let d_rating = comparison_string(
        &Some(&original.rating.to_string()),
        &Some(&"???".to_string()),
    );
    let d_starred = comparison_string(&Some(&original.starred), &Some(&candidate.starred));
    let mut the_string = CANDIDATE_SEP.to_string();
    the_string.push('\n');
    if let Some(n) = n {
        the_string.push_str(
            format!(
                "{VERTICAL_BAR:>SONG_PADDING$} {}Candidate{}: {}",
                Attribute::Underlined,
                Attribute::Reset,
                n
            )
            .as_str(),
        );
        the_string.push('\n');
    }

    the_string.push_str(
        format!(
            "{VERTICAL_BAR:>SONG_PADDING$} {TRACK_FIELD_STR:<FIELD_PADDING$}{d_title}
{VERTICAL_BAR:>SONG_PADDING$} {ARTIST_FIELD_STR:<FIELD_PADDING$}{d_artist}
{VERTICAL_BAR:>SONG_PADDING$} {ALBUM_FIELD_STR:<FIELD_PADDING$}{d_album}
{VERTICAL_BAR:>SONG_PADDING$} {TRACK_FIELD_STR:<FIELD_PADDING$}{d_track}
{VERTICAL_BAR:>SONG_PADDING$} {YEAR_FIELD_STR:<FIELD_PADDING$}{d_year}
{VERTICAL_BAR:>SONG_PADDING$} {PLAY_COUNT_FIELD_STR:<FIELD_PADDING$}{d_playcount}
{VERTICAL_BAR:>SONG_PADDING$} {RATING_FIELD_STR:<FIELD_PADDING$}{d_rating}
{VERTICAL_BAR:>SONG_PADDING$} {STARRED_FIELD_STR:<FIELD_PADDING$}{d_starred}",
        )
        .as_str(),
    );
    the_string.push('\n');
    the_string.push_str(CANDIDATE_SEP);

    the_string
}

pub fn comparison_string<T: std::cmp::Eq + std::fmt::Display>(
    left: &Option<T>,
    right: &Option<T>,
) -> String {
    let left_display = match &left {
        Some(t) => t.to_string(),
        None => "None".to_string(),
    };
    let right_display = match &right {
        Some(t) => t.to_string(),
        None => "None".to_string(),
    };

    if left == right {
        format!("= {}{}{}", Attribute::Dim, right_display, Attribute::Reset)
    } else {
        format!(
            "? {}{}{} -> {}{}{}",
            Attribute::Underlined,
            left_display,
            Attribute::Reset,
            Attribute::Bold,
            right_display,
            Attribute::Reset,
        )
    }
}
