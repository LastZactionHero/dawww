use std::sync::{Arc, Mutex};

use super::{DrawComponent, DrawResult};
use crate::cursor::Cursor;
use crate::draw_components::Position;
use crate::score_viewport::ScoreViewport;
use crate::loop_state::{LoopState, LoopMode};

pub struct StatusBarComponent {
    cursor: Cursor,
    score_viewport: ScoreViewport,
    loop_state: LoopState,
}

impl DrawComponent for StatusBarComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) -> Vec<DrawResult> {
        self.wb_string(buffer, pos, 0, 0, "|".repeat(pos.w));
        
        let loop_str = match self.loop_state.mode {
            LoopMode::Disabled => "[LOOP:OFF]".to_string(),
            LoopMode::Looping => {
                match (self.loop_state.start_time_b32, self.loop_state.end_time_b32) {
                    (Some(start), Some(end)) => format!("[LOOP:ON {}-{}]", start, end),
                    (Some(start), None) => format!("[LOOP:SET {}]", start),
                    _ => "[LOOP:ON]".to_string()
                }
            }
        };

        let status_str = format!(
            "{} [Cursor: {}] [Score Viewport: {}]",
            loop_str, self.cursor, self.score_viewport
        );
        self.wb_string(buffer, pos, 0, 0, status_str);
        vec![]
    }
}

impl StatusBarComponent {
    pub fn new(
        cursor: Cursor,
        score_viewport: ScoreViewport,
        loop_state: LoopState,
    ) -> StatusBarComponent {
        StatusBarComponent {
            cursor,
            score_viewport,
            loop_state,
        }
    }
}
