// pitch.rs

use std::cmp::Ordering;
use std::fmt;
pub static OCTAVE_MAX: u16 = 8;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Tone {
    C,
    Cs,
    D,
    Ds,
    E,
    F,
    Fs,
    G,
    Gs,
    A,
    As,
    B,
}

impl Tone {
    pub fn from_index(index: u16) -> Tone {
        match index {
            0 => Tone::C,
            1 => Tone::Cs,
            2 => Tone::D,
            3 => Tone::Ds,
            4 => Tone::E,
            5 => Tone::F,
            6 => Tone::Fs,
            7 => Tone::G,
            8 => Tone::Gs,
            9 => Tone::A,
            10 => Tone::As,
            11 => Tone::B,
            _ => panic!("Invalid tone index!"),
        }
    }
    pub fn index(&self) -> u16 {
        match self {
            Tone::C => 0,
            Tone::Cs => 1,
            Tone::D => 2,
            Tone::Ds => 3,
            Tone::E => 4,
            Tone::F => 5,
            Tone::Fs => 6,
            Tone::G => 7,
            Tone::Gs => 8,
            Tone::A => 9,
            Tone::As => 10,
            Tone::B => 11,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Tone::C => "C",
            Tone::Cs => "C#",
            Tone::D => "D",
            Tone::Ds => "D#",
            Tone::E => "E",
            Tone::F => "F",
            Tone::Fs => "F#",
            Tone::G => "G",
            Tone::Gs => "G#",
            Tone::A => "A",
            Tone::As => "A#",
            Tone::B => "B",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Pitch {
    pub tone: Tone,
    pub octave: u16,
}

impl Pitch {
    pub fn new(tone: Tone, octave: u16) -> Pitch {
        Pitch { tone, octave }
    }

    pub fn all() -> Vec<Pitch> {
        let mut pitches = vec![];
        for octave in 0..=OCTAVE_MAX {
            pitches.push(Pitch::new(Tone::C, octave));
            pitches.push(Pitch::new(Tone::Cs, octave));
            pitches.push(Pitch::new(Tone::D, octave));
            pitches.push(Pitch::new(Tone::Ds, octave));
            pitches.push(Pitch::new(Tone::E, octave));
            pitches.push(Pitch::new(Tone::F, octave));
            pitches.push(Pitch::new(Tone::Fs, octave));
            pitches.push(Pitch::new(Tone::G, octave));
            pitches.push(Pitch::new(Tone::Gs, octave));
            pitches.push(Pitch::new(Tone::A, octave));
            pitches.push(Pitch::new(Tone::As, octave));
            pitches.push(Pitch::new(Tone::B, octave));
        }
        pitches
    }

    pub fn next(&self) -> Option<Pitch> {
        if self.tone == Tone::B && self.octave == OCTAVE_MAX {
            return None;
        }
        if self.tone == Tone::B {
            return Some(Pitch::new(Tone::C, self.octave + 1));
        }
        Some(Pitch::new(
            Tone::from_index(self.tone.index() + 1),
            self.octave,
        ))
    }

    pub fn prev(&self) -> Option<Pitch> {
        if self.tone == Tone::C && self.octave == 0 {
            return None;
        }
        if self.tone == Tone::C {
            return Some(Pitch::new(Tone::B, self.octave - 1));
        }
        Some(Pitch::new(
            Tone::from_index(self.tone.index() - 1),
            self.octave,
        ))
    }

    pub fn frequency(&self, octave: u16) -> f64 {
        // Calculate the number of half steps from A4 (440 Hz)
        let half_steps_from_a4 = (octave as i32 - 4) * 12 + self.tone.index() as i32 - 9;

        // Calculate the frequency using the formula: 440 * 2^(n/12)
        440.0 * 2_f64.powf(half_steps_from_a4 as f64 / 12.0)
    }

    pub fn as_str(&self) -> String {
        format!("{}{}", self.tone.as_str(), self.octave)
    }
}

impl fmt::Display for Pitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.tone.as_str(), self.octave)
    }
}

impl PartialOrd for Pitch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.octave < other.octave {
            return Some(Ordering::Less);
        } else if self.octave > other.octave {
            return Some(Ordering::Greater);
        } else if self.tone.index() < other.tone.index() {
            return Some(Ordering::Less);
        } else if self.tone.index() > other.tone.index() {
            return Some(Ordering::Greater);
        }
        Some(Ordering::Equal)
    }
}
