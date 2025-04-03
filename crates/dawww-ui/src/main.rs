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
mod cursor;
mod draw_components;
mod events;
mod loop_state;
mod pitch;
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
use std::collections::HashMap;
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

    let score = if let Some(path) = env::args().nth(1) {
        info!("Loading song from {}", path);
        match SongFile::load(PathBuf::from(&path)) {
            Ok(loaded_score) => {
                info!("Successfully loaded song from {}", path);
                Arc::new(Mutex::new(loaded_score))
            }
            Err(e) => {
                eprintln!("Failed to load song from {}: {}", path, e);
                std::process::exit(1);
            }
        }
    } else {
        info!("Starting with blank song");
        Arc::new(Mutex::new(Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        }))
    };
    
    let mut app_state = AppState::new(score);
    app_state.run()?;

    Ok(())
}
