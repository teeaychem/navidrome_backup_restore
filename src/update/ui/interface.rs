use crate::update::{
    queries::{get_suggestions, id_match},
    ui::constants::*,
};
use crossterm::style::Attribute;
use elements::{candidate_string, display_missing};

use crate::{
    client::NavidromeClient,
    config::Config,
    input::{self, get_first_char},
    navidrome_db::{structs::NavidromeFile, NavidromeDatabase},
    update::{
        ui::*,
        updaters::{update_playcount, update_rating, update_starred},
    },
    ErrorSink,
};

pub async fn update(
    db: &NavidromeDatabase,
    nd_client: &NavidromeClient,
    config: &Config,
) -> Result<(), ErrorSink> {
    println!("{FIRST_SEARCH_MSG}");
    'track_loop: for track in db.media_files.values() {
        match id_match(nd_client, track).await {
            Ok(true) => continue,
            Ok(false) => {}
            Err(e) => {
                println!("Error: {e:?}");
                continue;
            }
        }

        println!();

        let song_string = display_missing(track);
        println!("Information for a song with the following attributes was found in the backup databse but not the current database:

{CANDIDATE_SEP}
{song_string}
{CANDIDATE_SEP}
");

        match input::get_first_char(&[
            "search for a track to transfer metadata to",
            "continue checking the backup database",
            "quit",
        ]) {
            's' => {}
            'c' => {
                println!("{AGAIN_SEARCH_MSG}");
                continue 'track_loop;
            }
            'q' => {
                std::process::exit(0);
            }
            _ => panic!("!"),
        }

        let mut suggestions = get_suggestions(nd_client, track, true, true, config).await?;
        if suggestions.is_empty() {
            log::info!(
                "Could not find a track with the same title, artist, and album.
So, searching for a track with the same title only…"
            );
            suggestions = get_suggestions(nd_client, track, false, false, config)
                .await
                .expect("oh");
        }
        if suggestions.is_empty() {
            println!("Could not find a candidate track.");
            println!("{AGAIN_SEARCH_MSG}");
            continue 'track_loop;
        }

        println!(
            "
{}Found {} candidate tracks.{}
",
            Attribute::Bold,
            suggestions.len(),
            Attribute::Reset
        );

        let mut candidates: Vec<NavidromeFile> = vec![];

        for (idx, suggestion) in suggestions.iter().enumerate() {
            let client_track = nd_client.get_song(suggestion).await?;
            let candidate_string = candidate_string(Some(idx + 1), track, &client_track);
            println!("{candidate_string}");
            candidates.push(client_track);
        }

        let msg = "
Please enter a number for the candidate you'd like to use.
Or, enter '0' to skip this track.
";

        let mut choice = input::get_number_in_range(msg, 0, suggestions.len());
        println!();
        if choice == 0 {
            println!("{AGAIN_SEARCH_MSG}");
            continue 'track_loop;
        } else {
            choice = choice.saturating_sub(1);
        }
        let choice_id = &suggestions[choice];

        loop {
            let the_choice = nd_client.get_song(choice_id).await?;
            let choice_string = candidate_string(None, track, &the_choice);
            println!(
                "Ok, working with:

{choice_string}
"
            );

            match get_first_char(&[
                "p copy the play count",
                "r copy the rating",
                "s copy whether the track is starred",
                "e copy everything",
                "finish",
                "quit",
            ]) {
                'p' => update_playcount(track, &the_choice, nd_client).await,
                'r' => update_rating(track, &the_choice, nd_client).await,
                's' => update_starred(track, &the_choice, nd_client).await,
                'e' => {
                    update_playcount(track, &the_choice, nd_client).await;
                    update_rating(track, &the_choice, nd_client).await;
                    update_starred(track, &the_choice, nd_client).await;
                }
                'f' => break,
                'q' => {
                    std::process::exit(0);
                }
                _ => panic!("!"),
            }
        }

        let new_id = suggestions.get(choice).unwrap();

        match get_first_char(&[
            "search for playlists which contained the track",
            "continue to the next track",
            "quit",
        ]) {
            's' => {}
            'c' => {
                println!("{AGAIN_SEARCH_MSG}");
                continue 'track_loop;
            }
            'q' => {
                std::process::exit(0);
            }
            _ => panic!("!"),
        }

        playlist_update(db, nd_client, config, track, new_id).await?;
    }
    println!("Finished checking tracks in the current navidrome instance.");
    Ok(())
}

pub async fn playlist_update(
    db: &NavidromeDatabase,
    nd_client: &NavidromeClient,
    _config: &Config,
    track: &NavidromeFile,
    new_id: &str,
) -> Result<(), ErrorSink> {
    'playlist_loop: for playlist in db.playlists.values() {
        for (idx, id) in playlist.track_ids.iter().enumerate() {
            if id == &track.id {
                println!(
                    "
The track was found in {} at position {}.
",
                    &playlist.name, idx
                );

                let client_playlist_result = nd_client.get_playlist(&playlist.id).await;

                match client_playlist_result {
                    Ok(client_playlist) => {
                        print!("The corresponding navidrome playlist ");
                        match client_playlist.track_ids.contains(&track.id) {
                            true => {
                                println!("contains the track, so skipping.")
                            }
                            false => {
                                println!("does not contain the track.");
                                let mut suggestion = 0;

                                for (backup_idx, backup_id) in playlist.track_ids.iter().enumerate()
                                {
                                    for (client_idx, client_id) in
                                        client_playlist.track_ids.iter().enumerate()
                                    {
                                        if idx <= backup_idx {
                                            break;
                                        } else if backup_id == client_id {
                                            suggestion = client_idx;
                                        }
                                    }

                                    let prev_id = &playlist.track_ids[idx.saturating_sub(1)];
                                    let prev_find = client_playlist
                                        .track_ids
                                        .iter()
                                        .position(|idx| idx == prev_id);
                                    if let Some(prev_idx) = prev_find {
                                        suggestion = prev_idx + 1;
                                    }
                                }

                                if suggestion == 0 {
                                    println!(
                                        "No suggestion of where to place the track could be made."
                                    );
                                    suggestion = client_playlist.track_ids.len();
                                }

                                println!("A suggested playlist (showing ±2 tracks around the change) is:");
                                let mut suggested_playlist = client_playlist.track_ids.clone();

                                suggested_playlist.insert(suggestion, new_id.to_owned());

                                let floor = suggestion.saturating_sub(2);
                                let ceil = std::cmp::min(
                                    suggested_playlist.len(),
                                    suggestion.saturating_add(3),
                                );

                                println!(
                                    "
{}{}{}
",
                                    Attribute::Underlined,
                                    client_playlist.name,
                                    Attribute::Reset
                                );

                                if 0 < floor {
                                    println!("{:>SONG_PADDING$}", "⋮");
                                }

                                for idx in floor..ceil {
                                    let track_id = &suggested_playlist[idx];
                                    let one_line = nd_client.get_song(track_id).await?.one_line();
                                    if idx == suggestion {
                                        println!(
                                            "{:>SONG_PADDING$}{}{} {one_line}{}",
                                            idx,
                                            Attribute::Underlined,
                                            Attribute::Bold,
                                            Attribute::Reset
                                        );
                                    } else {
                                        println!("{:>SONG_PADDING$} {one_line}", idx);
                                    }
                                }
                                if ceil < client_playlist.track_ids.len() {
                                    println!("{:>SONG_PADDING$}", "⋮");
                                }
                                println!();

                                match get_first_char(&[
                                    "b add the track to the beginning of the playlist",
                                    "e add the track to the end of the playlist",
                                    "s add the track to the suggested position in the playlist",
                                    "finish",
                                    "quit",
                                ]) {
                                    'b' => {
                                        let mut new_playlist = vec![new_id.to_owned()];
                                        new_playlist.extend_from_slice(&client_playlist.track_ids);
                                        nd_client
                                            .update_playlist(&client_playlist.id, &new_playlist)
                                            .await?;
                                    }
                                    'e' => {
                                        let mut new_playlist = client_playlist.track_ids.clone();
                                        new_playlist.push(new_id.to_owned());
                                        nd_client
                                            .update_playlist(&client_playlist.id, &new_playlist)
                                            .await?;
                                    }
                                    's' => {
                                        nd_client
                                            .update_playlist(
                                                &client_playlist.id,
                                                &suggested_playlist,
                                            )
                                            .await?;
                                    }
                                    'f' => break,
                                    'q' => {
                                        std::process::exit(0);
                                    }
                                    _ => panic!("!"),
                                }
                            }
                        }
                    }

                    Err(ErrorSink::MissingJSONPlaylist) => {
                        println!("However, the same playlist could not be found in the current navidrome instance.");
                    }
                    Err(e) => {
                        println!("However, some error occurred when attempting to find the playlist in the current navidrome instance.
{e:?}");
                    }
                }
                println!("{AGAIN_SEARCH_MSG}");
                continue 'playlist_loop;
            }
        }
    }
    println!("Finished checking playlists.");
    Ok(())
}
