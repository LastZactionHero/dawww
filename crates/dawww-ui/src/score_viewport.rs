use crate::draw_components::ViewportDrawResult;
use crate::pitch::Pitch;
use crate::resolution::Resolution;
use std::fmt;

#[derive(Clone, Copy)]
pub struct ScoreViewport {
    pub middle_pitch: Pitch,
    pub resolution: Resolution,
    pub time_point: u64,
    pub playback_time_point: u64,
}

impl ScoreViewport {
    pub fn new(
        middle_pitch: Pitch,
        resolution: Resolution,
        time_point: u64,
        playback_time_point: u64,
    ) -> ScoreViewport {
        ScoreViewport {
            middle_pitch,
            resolution,
            time_point,
            playback_time_point,
        }
    }

    pub fn next_octave(&self) -> ScoreViewport {
        let mut new_viewport = *self;
        if let Some(next_pitch) = self.middle_pitch.next() {
            new_viewport.middle_pitch = next_pitch;
        }
        new_viewport
    }

    pub fn prev_octave(&self) -> ScoreViewport {
        let mut new_viewport = *self;
        if let Some(prev_pitch) = self.middle_pitch.prev() {
            new_viewport.middle_pitch = prev_pitch;
        }
        new_viewport
    }

    pub fn next_bar(&self, viewport_draw_result: &ViewportDrawResult) -> ScoreViewport {
        let mut new_viewport = *self;

        let is_more_than_playhead_halfway_through_viewport = self.playback_time_point
            > (viewport_draw_result.time_point_end - viewport_draw_result.time_point_start) / 2 + viewport_draw_result.time_point_start;
        if is_more_than_playhead_halfway_through_viewport {
            new_viewport.time_point += 32;
        }

        new_viewport
    }

    pub fn prev_bar(&self, viewport_draw_result: &ViewportDrawResult) -> ScoreViewport {
        let mut new_viewport = *self;

        let is_less_than_playhead_halfway_through_viewport = self.playback_time_point
            < (viewport_draw_result.time_point_end - viewport_draw_result.time_point_start) / 2 + viewport_draw_result.time_point_start;
        if is_less_than_playhead_halfway_through_viewport && self.time_point >= 32 {
            new_viewport.time_point -= 32;
        }

        new_viewport
    }

    pub fn increase_resolution(&self) -> ScoreViewport {
        let mut new_viewport = *self;
        new_viewport.resolution = self.resolution.next_up();
        new_viewport
    }

    pub fn decrease_resolution(&self) -> ScoreViewport {
        let mut new_viewport = *self;
        new_viewport.resolution = self.resolution.next_down();
        new_viewport
    }

    pub fn set_playback_time(&self, time: u64) -> ScoreViewport {
        let mut new_viewport = *self;
        new_viewport.playback_time_point = time;
        new_viewport
    }

    pub fn set_time_point(&self, time: u64) -> ScoreViewport {
        let mut new_viewport = *self;
        new_viewport.time_point = time;
        new_viewport
    }
}

impl fmt::Display for ScoreViewport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.middle_pitch, self.time_point, self.playback_time_point
        )
    }
}
