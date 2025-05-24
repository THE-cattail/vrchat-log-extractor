use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    time::{Duration, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::event::{Event, KeyEvent};

const LOG_PREFIX_PLAYER: &str = "[Behaviour] Initialized player ";
const LOG_PREFIX_ROOM: &str = "[Behaviour] Entering Room: ";

#[derive(Parser, Debug)]
struct Args {
    #[arg(value_name = "LOG_DIR")]
    log_path: Option<PathBuf>,
}

fn main() -> Result<()> {
    food_log_rs::logger()
        .init()
        .context("failed to init logger")?;

    let args = Args::parse();

    let log_path = if let Some(path) = args.log_path {
        path
    } else {
        let mut log_files: Vec<_> = std::fs::read_dir(PathBuf::from(format!(
            "{}\\..\\LocalLow\\VRChat\\VRChat",
            std::env::var("APPDATA").context("env APPDATA not set")?
        )))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name()?.to_str() {
                    if name.starts_with("output_log_")
                        && std::path::Path::new(name)
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
                    {
                        return Some(path);
                    }
                }
            }
            None
        })
        .collect();

        log_files.sort_by_key(|path| {
            std::fs::metadata(path)
                .and_then(|meta| meta.modified())
                .unwrap_or(UNIX_EPOCH)
        });

        log_files.last().context("no log file found")?.clone()
    };

    let mut room_players = HashMap::new();
    let mut current_room = String::new();

    for line in BufReader::new(std::fs::File::open(log_path)?)
        .lines()
        .collect::<Result<Vec<String>, _>>()?
    {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        let _date = parts[0];
        let _time = parts[1];
        let content = parts[4..].join(" ");

        tracing::debug!("handling line: {content}");
        if content.starts_with(LOG_PREFIX_ROOM) {
            tracing::debug!("handling room: {content}");
            current_room = content.replace(LOG_PREFIX_ROOM, "");
        } else if content.starts_with(LOG_PREFIX_PLAYER) {
            tracing::debug!("handling player: {content}");
            room_players
                .entry(current_room.clone())
                .or_insert_with(HashSet::new)
                .insert(content.replace(LOG_PREFIX_PLAYER, ""));
        }
    }

    let mut rooms: Vec<_> = room_players.keys().cloned().collect();
    rooms.sort();

    for room in rooms {
        println!("| 房间：{room} |");
        if let Some(players) = room_players.get(&room) {
            let mut players: Vec<_> = players.iter().collect();
            players.sort();
            for player in players {
                println!("{player}");
            }
        }
    }

    print!("按任意键退出...");
    std::io::stdout().flush().ok();
    loop {
        if crossterm::event::poll(Duration::from_millis(500))? {
            if let Event::Key(KeyEvent { .. }) = crossterm::event::read()? {
                break;
            }
        }
    }

    Ok(())
}
