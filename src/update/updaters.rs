use crate::{client::NavidromeClient, navidrome_db::structs::NavidromeFile};

pub async fn update_playcount(
    original: &NavidromeFile,
    target: &NavidromeFile,
    client: &NavidromeClient,
) {
    let backup_play_count = original.playCount;
    let client_play_count = target.playCount;

    if client_play_count < backup_play_count {
        for _ in client_play_count..backup_play_count.saturating_sub(1) {
            let result = client.scrobble(&target.id, None).await;
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!(
                        "Failed to scrobble.
{e:?}"
                    );
                }
            }
        }
    }
    if original.playCount > 0 {
        if let Some(date) = &original.played {
            if let Ok(utc) = <chrono::DateTime<chrono::Utc> as std::str::FromStr>::from_str(date) {
                let since_epoch = utc.timestamp();
                let result = client.scrobble(&target.id, Some(since_epoch)).await;
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        println!(
                            "Failed to scrobble.
{e:?}"
                        );
                    }
                }
                return;
            }
        }
        // if one of the previous conditions for obtaining a timestamp fails, scrobble anyway
        let result = client.scrobble(&target.id, None).await;
        match result {
            Ok(_) => {}
            Err(e) => {
                println!(
                    "Failed to scrobble.
{e:?}"
                );
            }
        }
    }
}

pub async fn update_rating(
    original: &NavidromeFile,
    target: &NavidromeFile,
    client: &NavidromeClient,
) {
    let result = client.set_rating(&target.id, original.rating).await;
    match result {
        Ok(_) => {}
        Err(e) => {
            println!(
                "Failed to set rating.
{e:?}"
            );
        }
    }
}

pub async fn update_starred(
    original: &NavidromeFile,
    target: &NavidromeFile,
    client: &NavidromeClient,
) {
    let result = match original.starred {
        true => client.star(&target.id).await,
        false => client.unstar(&target.id).await,
    };
    match result {
        Ok(_) => {}
        Err(e) => {
            println!(
                "Failed to update starred.
{e:?}"
            );
        }
    }
}
