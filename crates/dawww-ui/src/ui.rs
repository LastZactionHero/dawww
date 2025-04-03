use crate::grid::TrackerGrid;
use crate::song::Song;
use ncurses::*;
use std::path::PathBuf;

pub struct TrackerUI {
    grid: TrackerGrid,
    song: Song,
}

impl TrackerUI {
    pub fn new(song_dir: PathBuf) -> Self {
        TrackerUI {
            grid: TrackerGrid::new(),
            song: Song::new(song_dir),
        }
    }

    pub fn run(&mut self) {
        loop {
            self.grid.draw();
            
            match getch() {
                KEY_LEFT => self.grid.move_cursor(-1, 0),
                KEY_RIGHT => self.grid.move_cursor(1, 0),
                KEY_UP => self.grid.move_cursor(0, -1),
                KEY_DOWN => self.grid.move_cursor(0, 1),
                10 => { // Enter key
                    self.grid.toggle_note();
                    self.song.set_notes(self.grid.get_notes().clone());
                },
                27 => break, // Escape key
                _ => (),
            }
        }
    }
} 