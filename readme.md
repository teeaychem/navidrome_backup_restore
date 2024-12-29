# Navidrome Backup/Restore

A cli application to make an online backup a navidrome database and (partially) restore information from a backup database to a running navidrome instance.

_*The UI is quite rough!*_

Configuration is done through a `config.toml` file, detailed below with an example included in this repo.
To create a backup use the argument `-b` or `--backup` and to restore use the argument `-r` or `--restore`.
The location of the config can be specified using `-c` or `--config`.

For example, to create a backup using the `config.toml` in the same directory the application is run from:

```
./navidrome_backup_restore -b
```

Or, to backup and then restore using a config from a different location;

```
./navidrome_backup_restore -r --backup --config ../config.toml
```

Note, a backup is always created before restoration if both `backup` and `restore` arguments are passed to the application.

## Use case

An example use case is taking a track from an album, moving it to a custom compilation, and then removing the remaining tracks.

If the file itself is moved, the navidrome instance will 'forget' play count, rating, etc. along with which playlists the track appeared in and where the track appeared.

With this application, a backup can be made before the move, and then used to update the metadata of the moved track, restoring play count, rating, and playlist inclusion, etc. (with some caveats).

An example interaction can be found in the `example_interaction.txt` file.
Here, a track has been moved to a different compilation, and it's attributes and then playlist positions are restored.

It looks awful, and it is!
Still, in a terminal there's some use of text styling to help make things easier to parse.

A brief explanation of the following:

```
Ok, working with:

       ------------------------------﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘
       | Track     = Overworld Theme (Unused)
       | Artist    = Super Mario World
       | Album     ? Various · 0 …  -> Various · ゲーム
       | Track     ? 0 -> None
       | Year      = 2016
       | Play count? 206 -> 0
       | Rating    ? 0 -> ???
       | Starred   = false
       ------------------------------﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘

What would you like to do?

        [p] copy the play count
        [r] copy the rating
        [s] copy whether the track is starred
        [e] copy everything
        [f]inish
        [q]uit

e
Ok, working with:

       ------------------------------﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘
       | Track     = Overworld Theme (Unused)
       | Artist    = Super Mario World
       | Album     ? Various · 0 …  -> Various · ゲーム
       | Track     ? 0 -> None
       | Year      = 2016
       | Play count= 206
       | Rating    ? 0 -> ???
       | Starred   = false
       ------------------------------﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘﹘
```

Here, a track has been selected for restoration, and some attributes are shown.
The equals sign (`=`) on an attribute means these attributes match, e.g. the track and artist.
The question mark (`?`) means the attributes do not match, e.g. the album and initially the play count.
After updating 'everything' by typing `e` the play count matches.

The rating also matches, but the subsonic api doesn't offer a way to obtain the rating for a track.

The 'last played' attribute isn't listed, but is updated with the playcount.
It is *somewhat* accurate (in the example, the date is correct, but time ends up off by a couple of hours).

## Notes

An 'online' backup here only means the backup happens while the navidrome instance is running.
No internet connection is required, or used --- in this sense the backup is entirely 'offline'.

Restoration is done using the opensonic api, and so is limited to:
+ The play count of a track.
+ When the track was last played.
+ The rating of a track.
+ Wheter the track is starred.
+ The inclusion of the track in a playlist, and perhaps the position of the track in the playlist.

Some metadata cannot be recovered, though is present in the backup.
For example, navidrome stores information about when a track was starred, but then is no way to restore this information without directly modifying the navidrome database.

# Overview

## Backup

The online backup is made in line with the example from [sqlite.org](https://www.sqlite.org/backup.html), following [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/index.html)'s [equivalent](https://docs.rs/rusqlite/latest/rusqlite/backup/index.html).

Backups are saved to a directory as specified in the config.toml file, and are marked with a unix time stap in miliseconds.

## Restoration

- The restore process is as follows:
  - Search the backup database for a missing track which does not appear in the running navidrome instance.
  - For any such missing track, search for candidates by using:
     * A track with the same title, artist, and album as the missing track.
     * And if that fails, a track with the same title as the missing track.
  - A candidate to transfer data from is then chosen, or the missing track is skipped.
  - Given a candidate, a choice to copy play count, rating, etc. is offered.
  - After, an optional search over playlists is offered.
    - Here, if the missing track belonged to a playlist and the candidate does not belong to the playlist and option to update the playlist is offered.
    - The song can be added:
      1. To the beggining of the playlist.
      2. To the, end of the playlist.
      3. After the first common track before the missing track --- i.e. if the playlist only contained unique tracks, the relative position of the missing track in the original playlist, ignoring any other missing tracks.

### Restoration caveats

- Playback restoration works by taking the difference of playcounts between the missing and candidate track, and then scrobbling a play to make up for the difference.
  - If you also scrobble to last.fm, etc. this might have some unintended consequences.
  - Unfortunately, there is no other way to update the play count of a track via the opensonic api, nor does the api allow 'local only' scrobbles to avoid this.

# Config

An example config is below.
If the config is saved as a `.toml` file in the same folder the application is run from it is loaded automatically.
Otherwise, it can be passed as an argument using `-c` or `--config`.

```
# URL to the current navidrome instance
navidrome_url = "http://localhost:4533"

# The full path to the navidrome database file.
navidrome_db_in_use = "…/Music/navidrome/navidrome.db"

# The full directory to write backup navidrome databases to.
backup_dir = "…/Music/navidrome/backups"

# The backup navidrome database to restore from.
navidrome_db_to_restore_from = "…/Music/navidrome/restore/navidrome.db"

# The username of the user to restore track information for.
user = "user"

# The password of the user to restore track information for.
password = "password"

# The limit of how many similar tracks to return when searching for a track to restore information to.
similar_track_search_limit = 5

# If true then some information about a backup and how it's progressing is printed.
# Set to false to not do this.
backup_stdout = true

# Configuration of pages per backup step as outlined in https://www.sqlite.org/backup.html
# Smaller is slower, larger is faster
backup_pages_per_step = 100

# Configuration of pause between pages/backup step as outlined in https://www.sqlite.org/backup.html
# The value is in milliseconds, smaller is faster, larger is slower.
backup_pause_between_pages = 25
```

# Issues

Aside from general improvements, of which there are many, a few things could do with fixing, if you're interested!

1. Updates to last played are not completely accurate --- somethings off with conversion between UTC and unix time, I think.
2. Updates to last played require a scrobble --- it'd be nice if this could be separated.
3. If a track was at the start of the playlist suggestion fails and puts it at the end --- this should be easy to fix!
