// song.rs

use crate::pitch::{Pitch, Tone};
use crate::score::{Note, Score, NoteState};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn create_song() -> Score {
    let mut score = Score {
        bpm: 120, // Default BPM
        notes: HashMap::new(),
        active_notes: HashMap::new(),
    };

    let file = File::open("song.txt").expect("Could not open song.txt");
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.expect("Error reading line");
        let line = line.trim();

        if line.starts_with("BPM:") {
            score.bpm = line[4..].trim().parse().expect("Invalid BPM format");
        } else if !line.is_empty() {
            let parts: Vec<&str> = line.split(':').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let onset: u64 = parts[0].parse().expect("Invalid onset format");
                let notes_str: Vec<&str> = parts[1].split_whitespace().collect();

                for note_str in notes_str {
                    let note_parts: Vec<&str> = note_str.split('-').collect();
                    if note_parts.len() == 2 {
                        let tone_octave = &note_parts[0];
                        let duration: u64 = note_parts[1].parse().expect("Invalid duration format");

                        let tone = match &tone_octave[..tone_octave.len() - 1] {
                            "C" => Tone::C,
                            "Cs" => Tone::Cs,
                            "D" => Tone::D,
                            "Ds" => Tone::Ds,
                            "E" => Tone::E,
                            "F" => Tone::F,
                            "Fs" => Tone::Fs,
                            "G" => Tone::G,
                            "Gs" => Tone::Gs,
                            "A" => Tone::A,
                            "As" => Tone::As,
                            "B" => Tone::B,
                            _ => panic!("Invalid tone: {}", &tone_octave[..tone_octave.len() - 1]),
                        };

                        let octave: u8 = tone_octave[tone_octave.len() - 1..]
                            .parse()
                            .expect("Invalid octave format");

                        // Use insert_or_remove instead of directly manipulating the notes HashMap
                        score.insert_or_remove(
                            Pitch::new(tone, octave as u16),
                            onset,
                            duration
                        );
                    }
                }
            }
        }
    }

    score
}
