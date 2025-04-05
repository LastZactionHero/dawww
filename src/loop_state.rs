#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoopMode {
    Disabled,
    Looping,
}

#[derive(Debug, Clone, Copy)]
pub struct LoopState {
    pub start_time_b32: Option<u64>,
    pub end_time_b32: Option<u64>,
    pub mode: LoopMode,
}

impl LoopState {
    pub fn new() -> Self {
        Self {
            start_time_b32: None,
            end_time_b32: None,
            mode: LoopMode::Disabled,
        }
    }

    pub fn mark(&self, time_b32: u64) -> Self {
        let mut new_state = *self;
        match (new_state.start_time_b32, new_state.end_time_b32) {
            (None, _) => {
                // First mark
                new_state.start_time_b32 = Some(time_b32);
            }
            (Some(start), None) => {
                // Second mark
                if time_b32 < start {
                    new_state.end_time_b32 = new_state.start_time_b32;
                    new_state.start_time_b32 = Some(time_b32);
                } else {
                    new_state.end_time_b32 = Some(time_b32);
                }
            }
            (Some(_), Some(_)) => {
                // If both times are set, clear and start over
                new_state.start_time_b32 = Some(time_b32);
                new_state.end_time_b32 = None;
            }
        }
        new_state
    }

    pub fn set_mode(&self, mode: LoopMode) -> Self {
        let mut new_state = *self;
        new_state.mode = mode;
        new_state
    }

    pub fn toggle_mode(&self) -> Self {
        let mut new_state = *self;
        new_state.mode = match new_state.mode {
            LoopMode::Disabled => LoopMode::Looping,
            LoopMode::Looping => LoopMode::Disabled,
        };
        new_state
    }

    pub fn clear(&self) -> Self {
        Self::new()
    }

    pub fn is_looping(&self) -> bool {
        self.mode == LoopMode::Looping
            && self.start_time_b32.is_some()
            && self.end_time_b32.is_some()
    }
}

impl Default for LoopState {
    fn default() -> Self {
        Self::new()
    }
}
