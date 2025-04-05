// main.rs
use log::*;
use simplelog::*;
use std::fs::File;
use std::{
    io,
    sync::{Arc, Mutex},
};
use std::env;
use std::path::PathBuf;
mod app_state;
mod audio;
mod cursor;
mod draw_components;
mod events;
mod loop_state;
mod player;
mod resolution;
mod score;
mod score_viewport;
mod selection_buffer;
mod selection_range;
mod song;
mod song_file;

use app_state::AppState;
use crate::score::Score;
use crate::song_file::SongFile;

fn main() -> io::Result<()> {
    // Initialize logging
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Debug,
        Config::default(),
        File::create("debug.log").unwrap(),
    )])
    .unwrap();

    info!("Application starting...");

    let mut song_file = SongFile::new();
    if let Some(path) = env::args().nth(1) {
        info!("Loading song from {}", path);
        match song_file.load(PathBuf::from(&path)) {
            Ok(score) => {
                info!("Successfully loaded song from {}", path);
                let score = Arc::new(Mutex::new(score));
                let mut app_state = AppState::new(score);
                app_state.run()?;
            }
            Err(e) => {
                error!("Error loading file: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        info!("Starting with blank song");
        let score = Arc::new(Mutex::new(Score::new()));
        let mut app_state = AppState::new(score);
        app_state.run()?;
    }

    Ok(())
}
