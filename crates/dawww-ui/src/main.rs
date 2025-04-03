use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
    cursor::MoveTo,
};
use crate::components::{DrawableComponent, GridComponent};
use crate::drawing::{Position, get_terminal_dimensions, send_image_data};
use crate::song::Song;

mod components;
mod drawing;
mod song;

const FONT_SIZE: f32 = 14.0;
const CELL_SIZE: (usize, usize) = (40, 30);
const GRID_MARGIN: usize = 60;
const DEBUG: bool = false; // Set to true to enable debug output

struct App {
    song: Song,
    grid: GridComponent,
    font_data: Vec<u8>,
}

impl App {
    fn new(song_dir: PathBuf) -> io::Result<Self> {
        let dimensions = get_terminal_dimensions()?;
        
        if DEBUG {
            eprintln!("Initial terminal dimensions: {}x{}", dimensions.width, dimensions.height);
        }
        
        // Calculate grid dimensions based on cell size
        let grid_width = dimensions.width.saturating_sub(GRID_MARGIN * 2);
        let grid_height = dimensions.height.saturating_sub(GRID_MARGIN * 2);
        
        // Calculate grid dimensions based on cell size
        let grid_cols = grid_width / CELL_SIZE.0;
        let grid_rows = grid_height / CELL_SIZE.1;
        
        // Adjust grid size to fit grid cells exactly
        let grid_width = grid_cols * CELL_SIZE.0;
        let grid_height = grid_rows * CELL_SIZE.1;
        
        if DEBUG {
            eprintln!("Grid dimensions: {}x{} cells", grid_cols, grid_rows);
            eprintln!("Grid size: {}x{} pixels", grid_width, grid_height);
        }
        
        let grid = GridComponent::new(
            Position { x: GRID_MARGIN, y: GRID_MARGIN },
            (grid_width, grid_height),
            CELL_SIZE,
            (0, 255, 255, 255), // Cyan color
        );

        let font_data = include_bytes!("../assets/JetBrainsMono-Regular.ttf").to_vec();

        Ok(Self {
            song: Song::new(song_dir),
            grid,
            font_data,
        })
    }

    fn run(&mut self) -> io::Result<()> {
        // Enable raw mode to prevent key echo
        terminal::enable_raw_mode()?;
        
        // Small delay to ensure terminal is ready
        std::thread::sleep(Duration::from_millis(100));
        
        // Clear the screen and move cursor to top-left
        let mut stdout = io::stdout();
        stdout
            .execute(Clear(ClearType::All))?
            .execute(MoveTo(0, 0))?
            .execute(EnterAlternateScreen)?;

        let dimensions = get_terminal_dimensions()?;
        let mut image_data = vec![0u8; dimensions.width * dimensions.height * 4];

        if DEBUG {
            eprintln!("Terminal dimensions: {}x{}", dimensions.width, dimensions.height);
            eprintln!("Grid size: {}x{}", self.grid.size.0, self.grid.size.1);
            eprintln!("Cell size: {}x{}", CELL_SIZE.0, CELL_SIZE.1);
            eprintln!("Debug output enabled. Press 'q' to quit.");
            eprintln!("----------------------------------------");
        }

        // Track if we need to redraw
        let mut needs_redraw = true;

        loop {
            // Only clear and redraw if needed
            if needs_redraw {
                // Clear image data
                for pixel in image_data.chunks_mut(4) {
                    pixel.copy_from_slice(&[0, 0, 0, 255]);
                }

                // Render grid directly
                self.grid.render(
                    &mut image_data,
                    dimensions.width,
                    dimensions.height,
                    Position { x: 0, y: 0 },
                    &self.font_data,
                    FONT_SIZE,
                )?;

                // Send image data to terminal
                send_image_data(&image_data, dimensions.width, dimensions.height)?;
                
                // Reset redraw flag
                needs_redraw = false;
            }

            // Handle input with a shorter timeout to be more responsive
            if event::poll(Duration::from_millis(10))? {
                match event::read()? {
                    Event::Key(KeyEvent { code, .. }) => {
                        if DEBUG {
                            eprintln!("Key pressed: {:?}", code);
                        }
                        
                        let old_selection = self.grid.selected_cell;
                        
                        match code {
                            KeyCode::Char('q') => break,
                            KeyCode::Left => {
                                if DEBUG {
                                    eprintln!("Moving left");
                                }
                                self.grid.move_selection(-1, 0);
                            },
                            KeyCode::Right => {
                                if DEBUG {
                                    eprintln!("Moving right");
                                }
                                self.grid.move_selection(1, 0);
                            },
                            KeyCode::Up => {
                                if DEBUG {
                                    eprintln!("Moving up");
                                }
                                self.grid.move_selection(0, -1);
                            },
                            KeyCode::Down => {
                                if DEBUG {
                                    eprintln!("Moving down");
                                }
                                self.grid.move_selection(0, 1);
                            },
                            KeyCode::Char(' ') => {
                                if DEBUG {
                                    eprintln!("Toggling note");
                                }
                                self.grid.toggle_note();
                                let _ = self.song.save();
                                needs_redraw = true;
                            }
                            _ => {}
                        }
                        
                        // Check if selection changed
                        if old_selection != self.grid.selected_cell {
                            if DEBUG {
                                eprintln!("Selection changed from {:?} to {:?}", old_selection, self.grid.selected_cell);
                            }
                            needs_redraw = true;
                        }
                    }
                    _ => {}
                }
            }

            // Flush stdout
            stdout.flush()?;
        }

        // Disable raw mode before exiting
        terminal::disable_raw_mode()?;
        stdout.execute(LeaveAlternateScreen)?;
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let song_dir = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from(".")
    };

    let mut app = App::new(song_dir)?;
    app.run()
}
