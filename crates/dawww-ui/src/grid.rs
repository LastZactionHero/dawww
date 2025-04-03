use dawww_core::pitch::{Pitch, Tone};
use ncurses::*;
use std::collections::HashMap;

const GRID_WIDTH: i32 = 64;
const GRID_HEIGHT: i32 = 24;
const SQUARE_SIZE: i32 = 24;

pub struct TrackerGrid {
    cursor_x: i32,
    cursor_y: i32,
    notes: HashMap<(i32, i32), bool>,
    start_pitch: Pitch,
}

impl TrackerGrid {
    pub fn new() -> Self {
        TrackerGrid {
            cursor_x: 0,
            cursor_y: GRID_HEIGHT / 2,
            notes: HashMap::new(),
            start_pitch: Pitch::new(Tone::C, 3), // Start at C3
        }
    }

    pub fn draw(&self) {
        clear();
        
        // Draw grid
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let screen_x = x * SQUARE_SIZE;
                let screen_y = y * SQUARE_SIZE;
                
                // Draw grid lines
                attron(COLOR_PAIR(1));
                for i in 0..SQUARE_SIZE {
                    mvaddch(screen_y + i, screen_x, '|' as u32);
                    mvaddch(screen_y, screen_x + i, '-' as u32);
                }
                attroff(COLOR_PAIR(1));

                // Draw notes
                if *self.notes.get(&(x, y)).unwrap_or(&false) {
                    attron(COLOR_PAIR(3));
                    for i in 1..SQUARE_SIZE-1 {
                        for j in 1..SQUARE_SIZE-1 {
                            mvaddch(screen_y + i, screen_x + j, '█' as u32);
                        }
                    }
                    attroff(COLOR_PAIR(3));
                }

                // Draw cursor
                if x == self.cursor_x && y == self.cursor_y {
                    attron(COLOR_PAIR(2));
                    for i in 1..SQUARE_SIZE-1 {
                        for j in 1..SQUARE_SIZE-1 {
                            mvaddch(screen_y + i, screen_x + j, '█' as u32);
                        }
                    }
                    attroff(COLOR_PAIR(2));
                }
            }
        }

        // Draw pitch labels
        for y in 0..GRID_HEIGHT {
            let pitch = if y < 12 {
                Pitch::new(Tone::from_index(y as u16), 4)
            } else {
                Pitch::new(Tone::from_index((y - 12) as u16), 3)
            };
            mvprintw(y * SQUARE_SIZE + SQUARE_SIZE/2, 0, &format!("{:?}", pitch));
        }

        refresh();
    }

    pub fn move_cursor(&mut self, dx: i32, dy: i32) {
        self.cursor_x = (self.cursor_x + dx).max(0).min(GRID_WIDTH - 1);
        self.cursor_y = (self.cursor_y + dy).max(0).min(GRID_HEIGHT - 1);
    }

    pub fn toggle_note(&mut self) {
        let pos = (self.cursor_x, self.cursor_y);
        let current = self.notes.get(&pos).unwrap_or(&false);
        self.notes.insert(pos, !current);
    }

    pub fn get_notes(&self) -> &HashMap<(i32, i32), bool> {
        &self.notes
    }
} 