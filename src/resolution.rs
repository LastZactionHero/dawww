#[derive(Clone, Copy)]
pub enum Resolution {
    Time1_4,
    Time1_8,
    Time1_16,
    Time1_32,
}

impl Resolution {
    pub fn as_str(&self) -> &str {
        match self {
            Resolution::Time1_4 => "1/4",
            Resolution::Time1_8 => "1/8",
            Resolution::Time1_16 => "1/16",
            Resolution::Time1_32 => "1/32",
        }
    }

    pub fn bar_length_in_beats(&self) -> usize {
        match self {
            Resolution::Time1_4 => 4,
            Resolution::Time1_8 => 8,
            Resolution::Time1_16 => 16,
            Resolution::Time1_32 => 32,
        }
    }

    pub fn duration_b32(&self) -> u64 {
        match self {
            Resolution::Time1_4 => 8,
            Resolution::Time1_8 => 4,
            Resolution::Time1_16 => 2,
            Resolution::Time1_32 => 1,
        }
    }

    pub fn next_down(&self) -> Resolution {
        match self {
            Resolution::Time1_32 => Resolution::Time1_16,
            Resolution::Time1_16 => Resolution::Time1_8,
            Resolution::Time1_8 => Resolution::Time1_4,
            Resolution::Time1_4 => Resolution::Time1_4,
        }
    }

    pub fn next_up(&self) -> Resolution {
        match self {
            Resolution::Time1_4 => Resolution::Time1_8,
            Resolution::Time1_8 => Resolution::Time1_16,
            Resolution::Time1_16 => Resolution::Time1_32,
            Resolution::Time1_32 => Resolution::Time1_32,
        }
    }
}
