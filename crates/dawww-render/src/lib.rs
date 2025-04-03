use dawww_core::DawFile;
use anyhow::Result;
use std::path::Path;

/// The main audio rendering engine that converts a DawFile into audio output
pub struct AudioEngine {
    daw_file: DawFile,
}

impl AudioEngine {
    /// Create a new AudioEngine instance from a DawFile
    pub fn new(daw_file: DawFile) -> Self {
        Self { daw_file }
    }

    /// Render the song to a WAV file at the specified path
    pub fn render(&self, output_path: &Path) -> Result<()> {
        // Calculate total duration in seconds
        let seconds_per_32nd_note = 60.0 / (self.daw_file.bpm as f64 * 8.0);
        let total_duration = self.calculate_total_duration(seconds_per_32nd_note);

        // Create WAV writer
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: self.daw_file.mixdown.sample_rate,
            bits_per_sample: self.daw_file.mixdown.bit_depth,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(output_path, spec)?;
        let mut buffer = vec![0.0; (total_duration * self.daw_file.mixdown.sample_rate as f64) as usize];

        // Process each event
        for event in &self.daw_file.events {
            let time_in_seconds = self.parse_time(&event.time, seconds_per_32nd_note);
            let sample_index = (time_in_seconds * self.daw_file.mixdown.sample_rate as f64) as usize;

            // For now, just generate a simple sine wave for each note
            for note in &event.notes {
                let frequency = note.pitch.frequency(note.pitch.octave);
                let duration_samples = (note.duration as f64 * seconds_per_32nd_note * self.daw_file.mixdown.sample_rate as f64) as usize;

                for i in 0..duration_samples {
                    let t = i as f64 / self.daw_file.mixdown.sample_rate as f64;
                    let sample = (2.0 * std::f64::consts::PI * frequency * t).sin();
                    
                    if sample_index + i < buffer.len() {
                        buffer[sample_index + i] += sample;
                    }
                }
            }
        }

        // Normalize and write to WAV file
        let max_sample = buffer.iter().fold(0.0_f64, |a, &b| a.max(b.abs()));
        for sample in buffer {
            let normalized = (sample / max_sample * i16::MAX as f64) as i16;
            writer.write_sample(normalized)?;
            writer.write_sample(normalized)?; // Stereo
        }

        writer.finalize()?;
        Ok(())
    }

    /// Calculate the total duration of the song in seconds
    fn calculate_total_duration(&self, seconds_per_32nd_note: f64) -> f64 {
        let mut max_time = 0.0_f64;
        for event in &self.daw_file.events {
            let time = self.parse_time(&event.time, seconds_per_32nd_note);
            for note in &event.notes {
                let duration = note.duration as f64 * seconds_per_32nd_note;
                max_time = max_time.max(time + duration);
            }
        }
        max_time
    }

    /// Parse a time string in the format "bar.32nd" into seconds
    fn parse_time(&self, time: &str, seconds_per_32nd_note: f64) -> f64 {
        let parts: Vec<&str> = time.split('.').collect();
        let bar = parts[0].parse::<f64>().unwrap();
        let thirty_second = parts[1].parse::<f64>().unwrap();
        ((bar - 1.0) * 32.0 + thirty_second) * seconds_per_32nd_note
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dawww_core::{Note, pitch::{Pitch, Tone}, Event};
    use std::path::PathBuf;

    #[test]
    fn test_parse_time() {
        let daw_file = DawFile::new("Test".to_string());
        let engine = AudioEngine::new(daw_file);
        let seconds_per_32nd = 60.0 / (120.0 * 8.0); // At 120 BPM

        assert_eq!(engine.parse_time("1.0", seconds_per_32nd), 0.0);
        assert_eq!(engine.parse_time("1.16", seconds_per_32nd), 16.0 * seconds_per_32nd);
        assert_eq!(engine.parse_time("2.0", seconds_per_32nd), 32.0 * seconds_per_32nd);
    }

    #[test]
    fn test_calculate_duration() {
        let mut daw_file = DawFile::new("Test".to_string());
        daw_file.set_bpm(120);

        let note = Note::new(Pitch::new(Tone::C, 4), 8);
        let event = Event {
            time: "1.0".to_string(),
            instrument: "test".to_string(),
            notes: vec![note],
        };
        daw_file.events.push(event);

        let engine = AudioEngine::new(daw_file);
        let seconds_per_32nd = 60.0 / (120.0 * 8.0);
        
        assert_eq!(engine.calculate_total_duration(seconds_per_32nd), 8.0 * seconds_per_32nd);
    }
}
