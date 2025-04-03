use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use chrono::Local;
use std::io::BufRead;
use std::io::BufReader;
use std::collections::HashMap;

use crate::score::Score;
use crate::pitch::Tone;
use crate::pitch::Pitch;

pub struct SongFile {
    current_path: Option<PathBuf>,
}

impl SongFile {
    pub fn new() -> Self {
        SongFile {
            current_path: None,
        }
    }

    fn generate_default_filename(&self) -> PathBuf {
        let date = Local::now().format("%Y%m%d");
        PathBuf::from(format!("song_{}.txt", date))
    }

    pub fn save(&mut self, score: &Score) -> io::Result<()> {
        let path = self.current_path.clone()
            .unwrap_or_else(|| self.generate_default_filename());
        
        let mut file = File::create(&path)?;
        
        // Write BPM
        writeln!(file, "BPM: {}", score.bpm)?;
        
        // Write notes
        let mut sorted_times: Vec<_> = score.notes.keys().collect();
        sorted_times.sort();
        
        for &time in sorted_times {
            if let Some(notes) = score.notes.get(&time) {
                let mut note_strs = Vec::new();
                
                for note in notes {
                    let tone_str = match note.pitch.tone {
                        Tone::C => "C",
                        Tone::Cs => "Cs",
                        Tone::D => "D",
                        Tone::Ds => "Ds",
                        Tone::E => "E",
                        Tone::F => "F",
                        Tone::Fs => "Fs",
                        Tone::G => "G",
                        Tone::Gs => "Gs",
                        Tone::A => "A",
                        Tone::As => "As",
                        Tone::B => "B",
                    };
                    
                    note_strs.push(format!("{}{}-{}", 
                        tone_str,
                        note.pitch.octave,
                        note.duration_b32
                    ));
                }
                
                writeln!(file, "{}: {}", time, note_strs.join(" "))?;
            }
        }
        
        self.current_path = Some(path);
        Ok(())
    }

    pub fn load(path: PathBuf) -> io::Result<Score> {
        let mut score = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };

        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?.trim().to_string();
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
                                _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid tone")),
                            };

                            let octave: u8 = tone_octave[tone_octave.len() - 1..]
                                .parse()
                                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid octave"))?;

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

        Ok(score)
    }
} 