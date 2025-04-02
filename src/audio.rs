use crate::DawFile;
use anyhow::Result;
use std::path::Path;

pub struct AudioEngine {
    daw_file: DawFile,
}

impl AudioEngine {
    pub fn new(daw_file: DawFile) -> Self {
        Self { daw_file }
    }

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

            // For now, just generate a simple sine wave for each pitch
            for pitch in &event.pitches {
                let frequency = self.pitch_to_frequency(&pitch.pitch);
                let duration_samples = (pitch.duration as f64 * seconds_per_32nd_note * self.daw_file.mixdown.sample_rate as f64) as usize;

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

    fn calculate_total_duration(&self, seconds_per_32nd_note: f64) -> f64 {
        let mut max_time = 0.0_f64;
        for event in &self.daw_file.events {
            let time = self.parse_time(&event.time, seconds_per_32nd_note);
            for pitch in &event.pitches {
                let duration = pitch.duration as f64 * seconds_per_32nd_note;
                max_time = max_time.max(time + duration);
            }
        }
        max_time
    }

    fn parse_time(&self, time: &str, seconds_per_32nd_note: f64) -> f64 {
        let parts: Vec<&str> = time.split('.').collect();
        let bar = parts[0].parse::<f64>().unwrap();
        let thirty_second = parts[1].parse::<f64>().unwrap();
        ((bar - 1.0) * 32.0 + thirty_second) * seconds_per_32nd_note
    }

    fn pitch_to_frequency(&self, pitch: &str) -> f64 {
        // Simple implementation for now - just handle C4 as 261.63 Hz
        // TODO: Implement proper pitch parsing
        match pitch {
            "C4" => 261.63,
            "D4" => 293.66,
            "E4" => 329.63,
            "F4" => 349.23,
            "G4" => 392.00,
            "A4" => 440.00,
            "B4" => 493.88,
            _ => 261.63, // Default to C4
        }
    }
} 