use std::{fs, io, path::PathBuf, time};

use rusqlite::Connection;

use crate::{config::Config, ErrorSink};

pub fn create_backup(
    navidrome_db: &Connection,
    backup_dir: &str,
    config: &Config,
) -> Result<(), ErrorSink> {
    if config.backup_stdout {
        println!("Creating backup");
    }

    if !PathBuf::from(backup_dir).exists() {
        fs::create_dir(backup_dir)?;
    }
    let now = chrono::Local::now().timestamp().to_string();
    let backup_file = format!("navidrome_backup_{now}.db");
    let mut backup_path = PathBuf::from(backup_dir);
    backup_path = backup_path.join(PathBuf::from(backup_file));
    let backup_path_string = match backup_path.to_str() {
        Some(s) => s,
        None => return Err(ErrorSink::BackupPath),
    };

    let mut destination = Connection::open(&backup_path)?;
    let backup = rusqlite::backup::Backup::new(navidrome_db, &mut destination)?;

    let print_percentage = |x: rusqlite::backup::Progress| {
        print!(
            "{:}% ",
            ((x.pagecount.saturating_sub(x.remaining)) as f32 / x.pagecount as f32 * 100_f32)
                as usize
        );
        io::Write::flush(&mut io::stdout()).unwrap();
    };

    match config.backup_stdout {
        true => {
            backup.run_to_completion(
                config.backup_pages_per_step,
                time::Duration::from_millis(config.backup_pause_between_pages),
                Some(print_percentage),
            )?;
            println!();
        }
        false => {
            backup.run_to_completion(
                config.backup_pages_per_step,
                time::Duration::from_millis(config.backup_pause_between_pages),
                None,
            )?;
        }
    };
    if config.backup_stdout {
        println!("Backup written to {}", backup_path_string);
    }

    Ok(())
}
