use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};

use super::{DrawComponent, DrawResult, ViewportDrawResult};
use crate::cursor::Cursor;
use crate::draw_components::Position;
use crate::events::InputEvent;
use crate::pitch::Pitch;
use crate::player::PlayState;
use crate::score::{ActiveNote, NoteState, Score};
use crate::score_viewport::ScoreViewport;
use crate::selection_buffer::SelectionBuffer;
use log::debug;
use crate::loop_state::{LoopState, LoopMode};

pub struct ScoreDrawComponent {
    score: Arc<Mutex<Score>>,
    play_state: PlayState,
    score_viewport: ScoreViewport,
    event_tx: mpsc::Sender<InputEvent>,
    cursor: Cursor,
    selection_buffer: SelectionBuffer,
    loop_state: LoopState,
}

impl DrawComponent for ScoreDrawComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &super::Position) -> Vec<DrawResult> {
        debug!(
            "Drawing score at position: x={}, y={}, w={}, h={}",
            pos.x, pos.y, pos.w, pos.h
        );

        self.draw_pitches(buffer, pos);
        let viewport_draw_result = self.draw_score(
            buffer,
            &Position {
                x: pos.x + 4,
                y: pos.y,
                w: pos.w - 4,
                h: pos.h,
            },
        );
        vec![DrawResult::ViewportDrawResult(viewport_draw_result)]
    }
}

impl ScoreDrawComponent {
    pub fn new(
        score: Arc<Mutex<Score>>,
        play_state: PlayState,
        score_viewport: ScoreViewport,
        tx: mpsc::Sender<InputEvent>,
        cursor: Cursor,
        selection_buffer: SelectionBuffer,
        loop_state: LoopState,
    ) -> ScoreDrawComponent {
        ScoreDrawComponent {
            score,
            play_state,
            score_viewport,
            event_tx: tx,
            cursor,
            selection_buffer,
            loop_state,
        }
    }

    fn visible_pitches(&self, pos: &Position) -> Vec<Pitch> {
        let num_pitches_to_display = pos.h - 1;

        let middle_pitch = self.score_viewport.middle_pitch;
        let mut pitches = vec![middle_pitch];
        for _ in 0..(num_pitches_to_display / 2) {
            if let Some(prev_pitch) = pitches.last().unwrap().prev() {
                pitches.push(prev_pitch);
            }
        }
        pitches.reverse();
        for _ in 0..(num_pitches_to_display / 2) - 1 {
            if let Some(next_pitch) = pitches.last().unwrap().next() {
                pitches.push(next_pitch);
            }
        }
        pitches.reverse();
        pitches
    }

    fn draw_score(&self, buffer: &mut Vec<Vec<char>>, pos: &super::Position) -> ViewportDrawResult {
        let pitches = self.visible_pitches(pos);
        let mut time_point = self.score_viewport.time_point;
        debug!("Drawing score with {} visible pitches", pitches.len());

        // Draw the empty score.
        for col in 0..pos.w - 1 {
            let bar_col = col % (self.score_viewport.resolution.bar_length_in_beats()) == 0;
            for (row, _pitch) in pitches.iter().enumerate() {
                let draw_char = if bar_col { '⎸' } else { '.' };
                self.wb(buffer, pos, col, row, draw_char);
            }

            if bar_col {
                let time_point_at_col = self.score_viewport.time_point
                    + (col as u64) * self.score_viewport.resolution.duration_b32();
                self.wb_string(
                    buffer,
                    pos,
                    col,
                    pitches.len(),
                    (time_point_at_col / (32)).to_string(),
                );
            }
        }

        // Draw the playhead and loop markers
        let mut time_point = self.score_viewport.time_point;
        for col in 0..pos.w - 1 {
            for _ in 0..self.score_viewport.resolution.duration_b32() {
                for (row, _pitch) in pitches.iter().enumerate() {
                    if time_point == self.score_viewport.playback_time_point {
                        self.wb(buffer, pos, col, row, '░');
                    } else if self.loop_state.mode == LoopMode::Looping {
                        // Show loop start/end markers if loop mode is enabled
                        if let Some(start_time) = self.loop_state.start_time_b32 {
                            if time_point == start_time {
                                self.wb(buffer, pos, col, row, '░');
                            }
                        }
                        if let Some(end_time) = self.loop_state.end_time_b32 {
                            if time_point == end_time {
                                self.wb(buffer, pos, col, row, '░');
                            }
                        }
                    }
                }
                time_point += 1;
            }
        }

        let mut time_point = self.score_viewport.time_point;
        for col in 0..pos.w - 1 {
            let mut col_states: HashMap<(usize, Pitch), NoteState> = HashMap::new();

            for _ in 0..self.score_viewport.resolution.duration_b32() {
                let active_notes = self.score.lock().unwrap().notes_active_at_time(time_point);

                for (row, pitch) in pitches.iter().enumerate() {
                    if let Some(active_note) =
                        active_notes.iter().find(|note| note.note.pitch == *pitch)
                    {
                        let current_state = col_states
                            .entry((row, *pitch))
                            .or_insert(NoteState::Sustain);
                        match active_note.state {
                            NoteState::Onset | NoteState::Release => {
                                *current_state = active_note.state
                            }
                            NoteState::Sustain => {
                                if *current_state == NoteState::Sustain {
                                    *current_state = NoteState::Sustain
                                }
                            }
                        }
                    }
                }

                if let SelectionBuffer::Score(ref selection_buffer_score) = self.selection_buffer {
                    let selected_notes = selection_buffer_score.notes_active_at_time(time_point);
                    let selected_notes_map: HashMap<Pitch, ActiveNote> = selected_notes
                        .into_iter()
                        .map(|active_note| (active_note.note.pitch, active_note))
                        .collect();

                    for (row, pitch) in pitches.iter().enumerate() {
                        if let Some(active_note) = selected_notes_map.get(pitch) {
                            let current_state = col_states
                                .entry((row, *pitch))
                                .or_insert(NoteState::Sustain);
                            *current_state = active_note.state;
                            match active_note.state {
                                NoteState::Onset => *current_state = NoteState::Onset,
                                NoteState::Sustain => *current_state = NoteState::Sustain,
                                NoteState::Release => *current_state = NoteState::Release,
                            }
                        }
                    }
                }

                time_point += 1;
            }

            for ((row, pitch), state) in col_states {
                let note_char = match state {
                    NoteState::Onset => '█',
                    NoteState::Sustain => '░',
                    NoteState::Release => '▒',
                };
                self.wb(buffer, pos, col, row, note_char);
            }
        }

        // Draw the cursor - iterate over each time point and pitch.
        let mut time_point = self.score_viewport.time_point;
        for col in 0..pos.w - 1 {
            for (row, pitch) in pitches.iter().enumerate() {
                if self.cursor.visible_at(*pitch, time_point) {
                    self.wb(buffer, pos, col, row, 'C');
                }
            }
            time_point += self.score_viewport.resolution.duration_b32();
        }

        ViewportDrawResult {
            pitch_low: *pitches.last().unwrap(),
            pitch_high: *pitches.first().unwrap(),
            time_point_start: self.score_viewport.time_point,
            time_point_end: time_point,
        }
    }

    fn draw_pitches(&self, buffer: &mut Vec<Vec<char>>, pos: &super::Position) {
        for (i, pitch) in self.visible_pitches(pos).iter().enumerate() {
            self.wb_string(buffer, pos, 0, i, pitch.as_str());
        }
    }
}
