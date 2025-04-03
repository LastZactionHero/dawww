use std::io;
use std::any::Any;
use crate::drawing::{Position, draw_box, draw_line, draw_label};

pub trait DrawableComponent: Any {
    fn render(&self, image_data: &mut [u8], width: usize, height: usize, parent_pos: Position, font_data: &[u8], font_size: f32) -> io::Result<()>;
    fn get_size(&self) -> (usize, usize);
    fn as_any(&mut self) -> &mut dyn Any;
}

#[derive(Clone)]
pub struct GridComponent {
    pub position: Position,
    pub size: (usize, usize),
    pub cell_size: (usize, usize),
    pub color: (u8, u8, u8, u8),
    pub selected_cell: (i32, i32),
    pub notes: Vec<(i32, i32)>,
}

impl GridComponent {
    pub fn new(position: Position, size: (usize, usize), cell_size: (usize, usize), color: (u8, u8, u8, u8)) -> Self {
        Self {
            position,
            size,
            cell_size,
            color,
            selected_cell: (0, 0),
            notes: Vec::new(),
        }
    }

    pub fn move_selection(&mut self, dx: i32, dy: i32) {
        let (x, y) = self.selected_cell;
        self.selected_cell = (x + dx, y + dy);
    }

    pub fn toggle_note(&mut self) {
        let note_pos = self.selected_cell;
        if let Some(pos) = self.notes.iter().position(|&pos| pos == note_pos) {
            self.notes.remove(pos);
        } else {
            self.notes.push(note_pos);
        }
    }
}

impl DrawableComponent for GridComponent {
    fn render(&self, image_data: &mut [u8], width: usize, height: usize, parent_pos: Position, font_data: &[u8], font_size: f32) -> io::Result<()> {
        let abs_pos = Position {
            x: parent_pos.x + self.position.x,
            y: parent_pos.y + self.position.y,
        };

        // Draw grid lines
        let (cols, rows) = (self.size.0 / self.cell_size.0, self.size.1 / self.cell_size.1);
        
        // Draw vertical lines
        for i in 0..=cols {
            let x = abs_pos.x + i * self.cell_size.0;
            draw_line(
                image_data,
                width,
                height,
                Position { x, y: abs_pos.y },
                Position { x, y: abs_pos.y + self.size.1 },
                1,
                self.color,
            );
        }
        
        // Draw horizontal lines
        for i in 0..=rows {
            let y = abs_pos.y + i * self.cell_size.1;
            draw_line(
                image_data,
                width,
                height,
                Position { x: abs_pos.x, y },
                Position { x: abs_pos.x + self.size.0, y },
                1,
                self.color,
            );
        }
        
        // Draw notes
        for &(x, y) in &self.notes {
            let cell_x = abs_pos.x + (x as usize * self.cell_size.0);
            let cell_y = abs_pos.y + (y as usize * self.cell_size.1);
            
            // Fill cell with magenta
            for py in cell_y..cell_y + self.cell_size.1 {
                for px in cell_x..cell_x + self.cell_size.0 {
                    if px < width && py < height {
                        let idx = (py * width + px) * 4;
                        if idx + 3 < image_data.len() {
                            image_data[idx] = 255;     // R
                            image_data[idx + 1] = 0;   // G
                            image_data[idx + 2] = 255; // B
                            image_data[idx + 3] = 255; // A
                        }
                    }
                }
            }
        }
        
        // Draw selected cell with yellow border
        let (sel_x, sel_y) = self.selected_cell;
        let cell_x = abs_pos.x + (sel_x as usize * self.cell_size.0);
        let cell_y = abs_pos.y + (sel_y as usize * self.cell_size.1);
        
        // Fill selected cell with semi-transparent yellow
        for py in cell_y..cell_y + self.cell_size.1 {
            for px in cell_x..cell_x + self.cell_size.0 {
                if px < width && py < height {
                    let idx = (py * width + px) * 4;
                    if idx + 3 < image_data.len() {
                        // Semi-transparent yellow overlay
                        image_data[idx] = 255;     // R
                        image_data[idx + 1] = 255; // G
                        image_data[idx + 2] = 0;   // B
                        image_data[idx + 3] = 128; // A (semi-transparent)
                    }
                }
            }
        }
        
        // Draw thicker border around selected cell
        draw_box(
            image_data,
            width,
            height,
            Position { x: cell_x, y: cell_y },
            Position { 
                x: cell_x + self.cell_size.0, 
                y: cell_y + self.cell_size.1 
            },
            2,
            (255, 255, 0, 255), // Yellow
        );

        // Draw pitch labels
        let pitch_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        for y in 0..rows {
            let pitch_idx = y % 12;
            let octave = if y / 12 >= 4 { 0 } else { 4 - (y / 12) };
            let pitch = format!("{}{}", pitch_names[pitch_idx], octave);
            
            draw_label(
                image_data,
                width,
                height,
                &pitch,
                Position {
                    x: abs_pos.x - 40,
                    y: abs_pos.y + (y * self.cell_size.1) + (self.cell_size.1 / 2) - (font_size / 2.0) as usize,
                },
                (255, 255, 255, 255),
                font_data,
                font_size,
            )?;
        }

        Ok(())
    }

    fn get_size(&self) -> (usize, usize) {
        self.size
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
} 
