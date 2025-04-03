use dawww_core::{
    DawFile,
    Event,
    instrument::Instrument,
    Note,
    pitch::{Pitch, Tone},
};
use std::collections::HashMap;
use std::path::PathBuf;
use serde_json::Map;

pub struct Song {
    path: PathBuf,
    daw_file: DawFile,
    notes: HashMap<(i32, i32), bool>,
}

impl Song {
    pub fn new(path: PathBuf) -> Self {
        let daw_file = if path.exists() {
            // For now, just create a new file since load isn't implemented
            let mut file = DawFile::new("New Song".to_string());
            let mut params = Map::new();
            params.insert("oscillator_wave".to_string(), serde_json::Value::String("sine".to_string()));
            params.insert("filter_type".to_string(), serde_json::Value::String("lowpass".to_string()));
            params.insert("filter_cutoff".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(880.0).unwrap()));
            params.insert("filter_resonance".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.3).unwrap()));
            params.insert("envelope_attack".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.01).unwrap()));
            params.insert("envelope_decay".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.2).unwrap()));
            params.insert("envelope_sustain".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.7).unwrap()));
            params.insert("envelope_release".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.1).unwrap()));
            let synth = Instrument::new_synth("subtractive", params);
            file.add_instrument("synth".to_string(), synth).unwrap();
            file
        } else {
            let mut file = DawFile::new(path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("New Song")
                .to_string());
            let mut params = Map::new();
            params.insert("oscillator_wave".to_string(), serde_json::Value::String("sine".to_string()));
            params.insert("filter_type".to_string(), serde_json::Value::String("lowpass".to_string()));
            params.insert("filter_cutoff".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(880.0).unwrap()));
            params.insert("filter_resonance".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.3).unwrap()));
            params.insert("envelope_attack".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.01).unwrap()));
            params.insert("envelope_decay".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.2).unwrap()));
            params.insert("envelope_sustain".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.7).unwrap()));
            params.insert("envelope_release".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.1).unwrap()));
            let synth = Instrument::new_synth("subtractive", params);
            file.add_instrument("synth".to_string(), synth).unwrap();
            file
        };

        Song {
            path,
            daw_file,
            notes: HashMap::new(),
        }
    }

    pub fn save(&self) {
        // Convert notes to events
        let mut events = Vec::new();
        for ((x, y), &has_note) in &self.notes {
            if has_note {
                let pitch = if *y < 12 {
                    Pitch::new(Tone::from_index(*y as u16), 4)
                } else {
                    Pitch::new(Tone::from_index((*y - 12) as u16), 3)
                };
                let time = format!("{}.{}", (*x / 32) + 1, *x % 32);
                
                let note = Note::new(pitch, 8); // 32nd note duration (8 = 1/8 of a beat)
                let event = Event {
                    time,
                    instrument: "synth".to_string(),
                    notes: vec![note],
                };
                events.push(event);
            }
        }

        // For now, we'll just print that we would save
        println!("Would save {} events to {:?}", events.len(), self.path);
    }

    pub fn get_notes(&self) -> &HashMap<(i32, i32), bool> {
        &self.notes
    }

    pub fn set_notes(&mut self, notes: HashMap<(i32, i32), bool>) {
        self.notes = notes;
        self.save();
    }
} 