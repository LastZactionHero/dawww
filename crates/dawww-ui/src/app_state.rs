// app_state.rs
use crate::cursor::Cursor;
use crate::draw_components::ViewportDrawResult;
use crate::loop_state::LoopState;
use dawww_core::pitch::{Pitch, Tone};
use crate::player::Player;
use crate::resolution::Resolution;
use crate::score::Score;
use crate::score_viewport::ScoreViewport;
use crate::{
    cursor::CursorMode,
    draw_components::{
        self, score_draw_component::ScoreDrawComponent, status_bar_component::StatusBarComponent,
        BoxDrawComponent, DrawComponent, DrawResult, NullComponent, Position, VSplitDrawComponent,
        Window,
    },
};
use crate::{
    events::{capture_input, InputEvent},
    selection_buffer::SelectionBuffer,
};
use crossterm::{
    cursor::{self},
    style::{self},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::io::{self, Write};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use crate::song_file::SongFile;
use log::error;

pub struct AppState {
    score: Arc<Mutex<Score>>,
    score_viewport: ScoreViewport,
    player: Arc<Mutex<Player>>,
    input_tx: mpsc::Sender<InputEvent>,
    input_rx: mpsc::Receiver<InputEvent>,
    input_thread: Option<JoinHandle<()>>,
    audio_thread: Option<JoinHandle<()>>,
    buffer: Option<Vec<Vec<char>>>,
    cursor: Cursor,
    selection_buffer: SelectionBuffer,
    viewport_draw_result: Option<ViewportDrawResult>,
    loop_state: LoopState,
    song_file: SongFile,
}

impl AppState {
    pub fn new(score: Arc<Mutex<Score>>) -> AppState {
        let (tx, rx) = mpsc::channel();

        let player = Player::create(Arc::clone(&score), 44100);
        let shared_player = Arc::new(Mutex::new(player));

        AppState {
            score,
            score_viewport: ScoreViewport::new(Pitch::new(Tone::C, 4), Resolution::Time1_16, 0, 0),
            player: shared_player,
            input_tx: tx,
            input_rx: rx,
            input_thread: None,
            audio_thread: None,
            buffer: None,
            cursor: Cursor::new(Pitch::new(Tone::C, 4), 0),
            selection_buffer: SelectionBuffer::None,
            viewport_draw_result: None,
            loop_state: LoopState::new(),
            song_file: SongFile::new(),
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        // Setup terminal
        let mut stdout = io::stdout();
        stdout.execute(terminal::Clear(ClearType::All))?;

        // Start input thread
        let input_tx = self.input_tx.clone();
        self.input_thread = Some(thread::spawn(move || {
            let _ = capture_input(&input_tx);
        }));

        // Main loop
        self.draw()?;
        self.event_loop()?;

        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn event_loop(&mut self) -> io::Result<()> {
        loop {
            match self.input_rx.recv() {
                Ok(msg) => {
                    match msg {
                        InputEvent::Quit => break,
                        
                        // Viewer navigation
                        InputEvent::ViewerOctaveIncrease => {
                            self.score_viewport = self.score_viewport.next_octave();
                        }
                        InputEvent::ViewerOctaveDecrease => {
                            self.score_viewport = self.score_viewport.prev_octave();
                        }
                        InputEvent::ViewerBarNext => {
                            let current_time = self.player.lock().unwrap().current_time_b32();
                            let next_time = current_time + 32 - current_time % 32;
                            self.player.lock().unwrap().set_time_b32(next_time);
                            self.score_viewport = self.score_viewport.set_playback_time(next_time);
                            self.score_viewport = self.score_viewport.next_bar(&self.viewport_draw_result.unwrap());
                        }
                        InputEvent::ViewerBarPrevious => {
                            let current_time = self.player.lock().unwrap().current_time_b32();
                            let prev_time = if current_time < 32 {
                                0
                            } else if current_time % 32 == 0 {
                                current_time - 32
                            } else {
                                current_time - (current_time % 32)
                            };
                            self.player.lock().unwrap().set_time_b32(prev_time);
                            self.score_viewport = self.score_viewport.set_playback_time(prev_time);
                            self.score_viewport = self.score_viewport.prev_bar(&self.viewport_draw_result.unwrap());
                        }
                        
                        // Resolution controls
                        InputEvent::ViewerResolutionIncrease => {
                            self.score_viewport = self.score_viewport.increase_resolution();
                            self.cursor = self.cursor.resolution_align(self.score_viewport.resolution.duration_b32());
                        }
                        InputEvent::ViewerResolutionDecrease => {
                            self.score_viewport = self.score_viewport.decrease_resolution();
                            self.cursor = self.cursor.resolution_align(self.score_viewport.resolution.duration_b32());
                        }
                        
                        // Playback controls
                        InputEvent::PlayerTogglePlayback => {
                            let mut player_guard = self.player.lock().unwrap();
                            player_guard.toggle_playback();
                        }
                        InputEvent::PlayerBeatChange(playback_time_point_b32) => {
                            self.score_viewport = self.score_viewport.set_playback_time(playback_time_point_b32);
                        }
                        
                        // Cursor movement
                        InputEvent::CursorUp => {
                            self.cursor = self.cursor.up();
                            match self.score_viewport.middle_pitch.next() {
                                Some(next_pitch) => self.score_viewport.middle_pitch = next_pitch,
                                None => (),
                            }
                            self.player.lock().unwrap().preview_note(self.cursor.pitch());
                        }
                        InputEvent::CursorDown => {
                            self.cursor = self.cursor.down();
                            match self.score_viewport.middle_pitch.prev() {
                                Some(prev_pitch) => self.score_viewport.middle_pitch = prev_pitch,
                                None => (),
                            }
                            self.player.lock().unwrap().preview_note(self.cursor.pitch());
                        }
                        InputEvent::CursorLeft => {
                            self.cursor = self.cursor.left(self.score_viewport.resolution.duration_b32());
                            self.selection_buffer = self.selection_buffer.translate_to(self.cursor.time_point());
                        }
                        InputEvent::CursorRight => {
                            self.cursor = self.cursor.right(self.score_viewport.resolution.duration_b32());
                            self.selection_buffer = self.selection_buffer.translate_to(self.cursor.time_point());
                        }
                        
                        // Note editing
                        InputEvent::InsertNote => {
                            match self.cursor.mode() {
                                CursorMode::Select(_, _) => {
                                    // Insert notes for the entire selection
                                    let selection_range = self.cursor.selection_range().unwrap();
                                    let pitch = self.cursor.pitch();
                                    let mut score_guard = self.score.lock().unwrap();
                                    
                                    // Calculate duration based on selection time points
                                    let duration = selection_range.time_point_end_b32 - selection_range.time_point_start_b32;
                                    score_guard.insert_or_remove(pitch, selection_range.time_point_start_b32, duration);
                                    
                                    // Move cursor to end of selection and clear selection mode
                                    self.cursor = self.cursor.end_select();
                                }
                                _ => {
                                    // Regular single note insertion
                                    self.score.lock().unwrap().insert_or_remove(
                                        self.cursor.pitch(),
                                        self.cursor.time_point(),
                                        self.score_viewport.resolution.duration_b32(),
                                    );
                                    self.cursor = self.cursor.right(self.score_viewport.resolution.duration_b32());
                                }
                            }
                        }
                        // Selection and clipboard
                        InputEvent::Cancel => {
                            self.cursor = self.cursor.cancel();
                            self.selection_buffer = SelectionBuffer::None;
                        }
                        InputEvent::Yank => {
                            if let CursorMode::Select(_, _) = self.cursor.mode() {
                                let selection_range = self.cursor.selection_range().unwrap();
                                let selection_score = self.score.lock().unwrap().clone_at_selection(selection_range);
                                self.cursor = self.cursor.yank().right(self.score_viewport.resolution.duration_b32());
                                self.selection_buffer = SelectionBuffer::Score(
                                    selection_score.translate(Some(self.cursor.time_point())),
                                );
                            }
                        }
                        InputEvent::Cut => {
                            if let CursorMode::Select(_, _) = self.cursor.mode() {
                                let selection_range = self.cursor.selection_range().unwrap();
                                let selection_score = self.score.lock().unwrap().clone_at_selection(selection_range);
                                self.score.lock().unwrap().delete_in_selection(selection_range);
                                self.cursor = self.cursor.end_select();
                                self.selection_buffer = SelectionBuffer::Score(
                                    selection_score.translate(Some(self.cursor.time_point())),
                                );
                            }
                        }
                        InputEvent::Paste => {
                            if let SelectionBuffer::Score(ref selection_buffer_score) = self.selection_buffer {
                                let mut score_guard = self.score.lock().unwrap();
                                *score_guard = score_guard.merge_down(selection_buffer_score);
                                let duration = selection_buffer_score.duration();
                                self.cursor = self.cursor.right(duration);
                                self.selection_buffer = SelectionBuffer::Score(
                                    selection_buffer_score.translate(Some(self.cursor.time_point())),
                                );
                            }
                        }
                        InputEvent::Delete => {
                            if let Some(selection_range) = self.cursor.selection_range() {
                                self.score.lock().unwrap().delete_in_selection(selection_range);
                                self.cursor = self.cursor.end_select();
                            }
                        }
                        
                        // Loop controls
                        InputEvent::ToggleLoopMode => {
                            self.loop_state = self.loop_state.toggle_mode();
                            self.player.lock().unwrap().set_loop_state(self.loop_state);
                        }
                        InputEvent::SetLoopTimes => {
                            self.loop_state = self.loop_state.mark(self.score_viewport.playback_time_point);
                            self.player.lock().unwrap().set_loop_state(self.loop_state);
                        }
                        
                        // File operations
                        InputEvent::SaveSong => {
                            let mut score = self.score.lock().unwrap();
                            if let Err(e) = self.song_file.save(&mut score) {
                                error!("Failed to save song: {}", e);
                            }
                        }
                        
                        InputEvent::SelectIn => {
                            self.cursor = self.cursor.start_select();
                        }
                    }
                    self.draw()?;
                }
                Err(e) => {
                    eprintln!("Error in event loop: {e}");
                    break;
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self) -> io::Result<()> {
        let (width, height) = terminal::size()?;
        
        // Ensure minimum terminal size
        const MIN_WIDTH: u16 = 80;
        const MIN_HEIGHT: u16 = 24;
        
        if width < MIN_WIDTH || height < MIN_HEIGHT {
            let mut stdout = io::stdout();
            stdout.execute(terminal::Clear(ClearType::All))?;
            stdout.queue(cursor::MoveTo(0, 0))?;
            stdout.queue(style::Print(format!(
                "Terminal too small. Minimum size required: {}x{} (current: {}x{})",
                MIN_WIDTH, MIN_HEIGHT, width, height
            )))?;
            stdout.flush()?;
            return Ok(());
        }

        let mut buffer = vec![vec![' '; width as usize]; height as usize];

        let mut stdout = io::stdout();
        if self.buffer.is_none() {
            stdout.execute(terminal::Clear(ClearType::All))?;
        }

        let base_component = Window::new(vec![Box::new(BoxDrawComponent::new(Box::new(
            VSplitDrawComponent::new(
                draw_components::VSplitStyle::HalfWithDivider,
                Box::new(ScoreDrawComponent::new(
                    Arc::clone(&self.score),
                    self.player.lock().unwrap().state(),
                    self.score_viewport,
                    self.input_tx.clone(),
                    self.cursor,
                    self.selection_buffer.clone(),
                    self.loop_state,
                )),
                Box::new(VSplitDrawComponent::new(
                    draw_components::VSplitStyle::StatusBarNoDivider,
                    Box::new(NullComponent {}),
                    Box::new(StatusBarComponent::new(
                        self.cursor,
                        self.score_viewport,
                        self.loop_state,
                    )),
                )),
            ),
        )))]);

        let position = Position {
            x: 0,
            y: 0,
            w: width as usize,
            h: height as usize,
        };
        let draw_results = base_component.draw(&mut buffer, &position);
        for draw_result in draw_results {
            match draw_result {
                DrawResult::ViewportDrawResult(viewport_draw_result) => {
                    self.viewport_draw_result = Some(viewport_draw_result);
                    let player = self.player.lock().unwrap();
                    if player.is_playing()
                        && (player.current_time_b32() < viewport_draw_result.time_point_start
                            || player.current_time_b32() >= viewport_draw_result.time_point_end)
                    {
                        let new_time = player.current_time_b32() - player.current_time_b32() % 32;
                        self.score_viewport = self.score_viewport.set_time_point(new_time);
                    }
                    
                    if self.cursor.time_point() < viewport_draw_result.time_point_start
                        || self.cursor.time_point() >= viewport_draw_result.time_point_end - 2
                    {
                        let new_time = self.cursor.time_point() - self.cursor.time_point() % 32;
                        self.score_viewport = self.score_viewport.set_time_point(new_time);
                    }
                }
            }
        }

        for y in 0..height {
            for x in 0..width {
                let char = buffer[y as usize][x as usize];
                if self.buffer.is_none()
                    || char != self.buffer.as_ref().unwrap()[y as usize][x as usize]
                {
                    stdout
                        .queue(cursor::MoveTo(x, y))?
                        .queue(style::Print(char))?;
                }
            }
        }
        stdout.flush()?;

        self.buffer = Some(buffer);
        Ok(())
    }
}
