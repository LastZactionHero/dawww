use crate::pitch::Pitch;

#[derive(Debug, Clone, Copy)]
pub struct SelectionRange {
    pub time_point_start_b32: u64,
    pub time_point_end_b32: u64,
    pub pitch_low: Pitch,
    pub pitch_high: Pitch,
}