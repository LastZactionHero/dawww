use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, bail};
use std::path::PathBuf;
use std::time::SystemTime;

pub mod pitch;
pub mod metadata;
pub mod instrument;

use pitch::Pitch;
use metadata::Metadata;
use instrument::Instrument;

#[derive(Debug, Serialize, Deserialize)]
pub struct DawFile {
    pub metadata: Metadata,
    pub bpm: u32,
    pub mixdown: MixdownSettings,
    pub instruments: HashMap<String, Instrument>,
    pub events: Vec<Event>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MixdownSettings {
    pub sample_rate: u32,
    pub bit_depth: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    pub pitch: Pitch,
    pub duration: u32,  // Duration in 32nd notes
}

impl Note {
    pub fn new(pitch: Pitch, duration: u32) -> Self {
        Self { pitch, duration }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub time: String,
    pub instrument: String,
    pub notes: Vec<Note>,
}

impl DawFile {
    /// Create a new empty song with default settings
    pub fn new(title: String) -> Self {
        Self {
            metadata: Metadata::new(title),
            bpm: 120,
            mixdown: MixdownSettings {
                sample_rate: 44100,
                bit_depth: 16,
            },
            instruments: HashMap::new(),
            events: Vec::new(),
        }
    }

    /// Save to disk, handling the revision increment
    pub fn save(&mut self, path: &PathBuf) -> Result<()> {
        // Update modification date and increment revision
        self.metadata.update_modification_date();
        self.metadata.increment_revision();

        // Serialize and write to file
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Update the song title
    pub fn set_title(&mut self, title: String) {
        self.metadata.set_title(title);
    }

    /// Update the song tempo
    pub fn set_bpm(&mut self, bpm: u32) {
        self.bpm = bpm;
        std::thread::sleep(std::time::Duration::from_secs(1));
        self.metadata.update_modification_date();
    }

    /// Update the mixdown settings
    pub fn set_mixdown_settings(&mut self, sample_rate: u32, bit_depth: u16) {
        self.mixdown.sample_rate = sample_rate;
        self.mixdown.bit_depth = bit_depth;
        std::thread::sleep(std::time::Duration::from_secs(1));
        self.metadata.update_modification_date();
    }

    /// Add a new instrument
    pub fn add_instrument(&mut self, id: String, instrument: Instrument) -> Result<()> {
        // Validate the instrument first
        instrument.validate()?;

        // Check if ID already exists
        if self.instruments.contains_key(&id) {
            bail!("Instrument with ID '{}' already exists", id);
        }

        self.instruments.insert(id, instrument);
        self.metadata.update_modification_date();
        Ok(())
    }

    /// Remove an instrument
    pub fn remove_instrument(&mut self, id: &str) -> Result<()> {
        // Check if instrument exists
        if !self.instruments.contains_key(id) {
            bail!("Instrument with ID '{}' not found", id);
        }

        // Check if instrument is used in any events
        if self.events.iter().any(|e| e.instrument == id) {
            bail!("Cannot remove instrument '{}' as it is used in events", id);
        }

        self.instruments.remove(id);
        self.metadata.update_modification_date();
        Ok(())
    }

    /// Rename an instrument
    pub fn rename_instrument(&mut self, old_id: &str, new_id: String) -> Result<()> {
        // Check if old ID exists
        if !self.instruments.contains_key(old_id) {
            bail!("Instrument with ID '{}' not found", old_id);
        }

        // Check if new ID already exists
        if self.instruments.contains_key(&new_id) {
            bail!("Instrument with ID '{}' already exists", new_id);
        }

        // Remove and reinsert with new key
        let instrument = self.instruments.remove(old_id).unwrap();
        self.instruments.insert(new_id.clone(), instrument);

        // Update all events using this instrument
        for event in &mut self.events {
            if event.instrument == old_id {
                event.instrument = new_id.clone();
            }
        }

        self.metadata.update_modification_date();
        Ok(())
    }

    /// Get immutable reference to instrument
    pub fn get_instrument(&self, id: &str) -> Option<&Instrument> {
        self.instruments.get(id)
    }

    /// Get mutable reference to instrument
    pub fn get_instrument_mut(&mut self, id: &str) -> Option<&mut Instrument> {
        self.instruments.get_mut(id)
    }

    /// List all instrument IDs
    pub fn list_instruments(&self) -> Vec<&str> {
        self.instruments.keys().map(|s| s.as_str()).collect()
    }

    /// Create a new sampler instrument
    pub fn create_sampler_instrument(
        &mut self,
        id: String,
        sample_path: PathBuf,
    ) -> Result<()> {
        let instrument = Instrument::new_sampler(sample_path);
        self.add_instrument(id, instrument)
    }

    /// Add a new event
    pub fn add_event(&mut self, event: Event) -> Result<()> {
        // Validate instrument exists
        if !self.instruments.contains_key(&event.instrument) {
            bail!("Instrument '{}' not found", event.instrument);
        }

        // Validate time format
        self.validate_time_format(&event.time)?;

        // Insert event in correct position to maintain chronological order
        let insert_pos = self.events.partition_point(|e| e.time < event.time);
        self.events.insert(insert_pos, event);
        
        self.metadata.update_modification_date();
        Ok(())
    }

    /// Remove an event at the specified time and instrument
    pub fn remove_event(&mut self, time: &str, instrument: &str) -> Result<()> {
        // Validate time format first
        self.validate_time_format(time)?;

        // Find and remove the event
        let pos = self.events.iter().position(|e| e.time == time && e.instrument == instrument)
            .ok_or_else(|| anyhow::anyhow!("Event not found at time '{}' for instrument '{}'", time, instrument))?;
        
        self.events.remove(pos);
        self.metadata.update_modification_date();
        Ok(())
    }

    /// Update an existing event
    pub fn update_event(&mut self, time: &str, instrument: &str, new_event: Event) -> Result<()> {
        // Validate time format
        self.validate_time_format(time)?;
        self.validate_time_format(&new_event.time)?;

        // Validate new instrument exists
        if !self.instruments.contains_key(&new_event.instrument) {
            bail!("New instrument '{}' not found", new_event.instrument);
        }

        // Find the event
        let pos = self.events.iter().position(|e| e.time == time && e.instrument == instrument)
            .ok_or_else(|| anyhow::anyhow!("Event not found at time '{}' for instrument '{}'", time, instrument))?;

        // If time changed, we need to maintain chronological order
        if new_event.time != time {
            self.events.remove(pos);
            let insert_pos = self.events.partition_point(|e| e.time < new_event.time);
            self.events.insert(insert_pos, new_event);
        } else {
            self.events[pos] = new_event;
        }

        self.metadata.update_modification_date();
        Ok(())
    }

    /// Add a note to an existing event, or create a new event if none exists
    pub fn add_note(&mut self, time: &str, instrument: &str, note: Note) -> Result<()> {
        // Validate time format
        self.validate_time_format(time)?;

        // Validate instrument exists
        if !self.instruments.contains_key(instrument) {
            bail!("Instrument '{}' not found", instrument);
        }

        // Find or create event
        if let Some(event) = self.events.iter_mut().find(|e| e.time == time && e.instrument == instrument) {
            // Add note to existing event
            event.notes.push(note);
            self.metadata.update_modification_date();
            Ok(())
        } else {
            // Create new event
            let event = Event {
                time: time.to_string(),
                instrument: instrument.to_string(),
                notes: vec![note],
            };
            self.add_event(event)
        }
    }

    /// Remove a note from an event
    pub fn remove_note(&mut self, time: &str, instrument: &str, note: &Note) -> Result<()> {
        // Validate time format
        self.validate_time_format(time)?;

        // Find the event
        let event = self.events.iter_mut()
            .find(|e| e.time == time && e.instrument == instrument)
            .ok_or_else(|| anyhow::anyhow!("Event not found at time '{}' for instrument '{}'", time, instrument))?;

        // Find and remove the note
        let pos = event.notes.iter().position(|n| n.pitch == note.pitch && n.duration == note.duration)
            .ok_or_else(|| anyhow::anyhow!("Note not found in event"))?;
        
        event.notes.remove(pos);

        // If event has no more notes, remove it
        if event.notes.is_empty() {
            self.remove_event(time, instrument)?;
        }

        self.metadata.update_modification_date();
        Ok(())
    }

    /// Update a note in an event
    pub fn update_note(&mut self, time: &str, instrument: &str, old_note: &Note, new_note: Note) -> Result<()> {
        // Validate time format
        self.validate_time_format(time)?;

        // Find the event
        let event = self.events.iter_mut()
            .find(|e| e.time == time && e.instrument == instrument)
            .ok_or_else(|| anyhow::anyhow!("Event not found at time '{}' for instrument '{}'", time, instrument))?;

        // Find and update the note
        let pos = event.notes.iter().position(|n| n.pitch == old_note.pitch && n.duration == old_note.duration)
            .ok_or_else(|| anyhow::anyhow!("Note not found in event"))?;
        
        event.notes[pos] = new_note;
        self.metadata.update_modification_date();
        Ok(())
    }

    /// Get events within a time range
    pub fn get_events_in_range(&self, start_time: &str, end_time: &str) -> Result<Vec<&Event>> {
        // Validate time format
        self.validate_time_format(start_time)?;
        self.validate_time_format(end_time)?;

        Ok(self.events.iter()
            .filter(|e| e.time.as_str() >= start_time && e.time.as_str() <= end_time)
            .collect())
    }

    /// Get all events for an instrument
    pub fn get_events_by_instrument(&self, instrument_id: &str) -> Vec<&Event> {
        self.events.iter()
            .filter(|e| e.instrument == instrument_id)
            .collect()
    }

    /// Get all events in a specific bar
    pub fn get_events_in_bar(&self, bar: u32) -> Result<Vec<&Event>> {
        let prefix = format!("{}.", bar);
        Ok(self.events.iter()
            .filter(|e| e.time.starts_with(&prefix))
            .collect())
    }

    /// Validate time format (bar.32nd)
    fn validate_time_format(&self, time: &str) -> Result<()> {
        let parts: Vec<&str> = time.split('.').collect();
        if parts.len() != 2 {
            bail!("Invalid time format '{}'. Expected 'bar.32nd'", time);
        }

        let bar = parts[0].parse::<u32>()
            .map_err(|_| anyhow::anyhow!("Invalid bar number in time '{}'", time))?;
        let thirty_second = parts[1].parse::<u32>()
            .map_err(|_| anyhow::anyhow!("Invalid 32nd note in time '{}'", time))?;

        if bar == 0 {
            bail!("Bar number must be greater than 0");
        }
        if thirty_second >= 32 {
            bail!("32nd note must be between 0 and 31");
        }

        Ok(())
    }
}

/// Find the .daw.json file in the given directory
pub fn find_daw_file(dir: &PathBuf) -> Result<PathBuf> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_name().to_string_lossy().ends_with(".daw.json") {
            return Ok(entry.path());
        }
    }
    anyhow::bail!("No .daw.json file found in {}", dir.display());
}

/// Read and parse a DAW file from the given path
pub fn read_daw_file(path: &PathBuf) -> Result<DawFile> {
    let content = std::fs::read_to_string(path)?;
    let daw_data: DawFile = serde_json::from_str(&content)?;
    Ok(daw_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use pitch::{Pitch, Tone};

    fn create_test_daw_file() -> DawFile {
        let mut daw = DawFile::new("Test Song".to_string());
        
        // Add a test sampler instrument
        let sampler = Instrument::new_sampler(PathBuf::from("test.wav"));
        daw.add_instrument("sampler1".to_string(), sampler).unwrap();
        
        daw
    }

    #[test]
    fn test_daw_file_serialization() {
        let mut daw = create_test_daw_file();

        // Add a test event
        let event = Event {
            time: "1.1".to_string(),
            instrument: "sampler1".to_string(),
            notes: vec![Note::new(Pitch::new(Tone::C, 4), 8)],
        };
        daw.add_event(event).unwrap();

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&daw).unwrap();
        
        // Deserialize back
        let daw2: DawFile = serde_json::from_str(&json).unwrap();
        
        // Compare fields
        assert_eq!(daw.metadata.title, daw2.metadata.title);
        assert_eq!(daw.bpm, daw2.bpm);
        assert_eq!(daw.mixdown.sample_rate, daw2.mixdown.sample_rate);
        assert_eq!(daw.mixdown.bit_depth, daw2.mixdown.bit_depth);
        assert_eq!(daw.instruments.len(), daw2.instruments.len());
        assert_eq!(daw.events.len(), daw2.events.len());
        assert_eq!(daw.events[0].time, daw2.events[0].time);
        assert_eq!(daw.events[0].instrument, daw2.events[0].instrument);
        assert_eq!(daw.events[0].notes.len(), daw2.events[0].notes.len());
        assert_eq!(daw.events[0].notes[0].pitch.tone, daw2.events[0].notes[0].pitch.tone);
        assert_eq!(daw.events[0].notes[0].duration, daw2.events[0].notes[0].duration);
    }

    #[test]
    fn test_find_daw_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.daw.json");
        
        // Create a test DAW file
        let daw_file = create_test_daw_file();
        let content = serde_json::to_string(&daw_file).unwrap();
        fs::write(&file_path, content).unwrap();
        
        // Test finding the file
        let found_path = find_daw_file(&temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(found_path, file_path);
    }

    #[test]
    fn test_find_daw_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let result = find_daw_file(&temp_dir.path().to_path_buf());
        assert!(result.is_err());
    }

    #[test]
    fn test_read_daw_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.daw.json");
        
        // Create a test DAW file
        let original_daw = create_test_daw_file();
        let content = serde_json::to_string(&original_daw).unwrap();
        fs::write(&file_path, content).unwrap();
        
        // Test reading the file
        let read_daw = read_daw_file(&file_path).unwrap();
        assert_eq!(read_daw.metadata.title, original_daw.metadata.title);
        assert_eq!(read_daw.bpm, original_daw.bpm);
        assert_eq!(read_daw.events.len(), original_daw.events.len());
    }

    #[test]
    fn test_read_daw_file_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("invalid.daw.json");
        
        // Create an invalid JSON file
        fs::write(&file_path, "invalid json content").unwrap();
        
        // Test reading invalid file
        let result = read_daw_file(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_daw_file() {
        let title = "New Song".to_string();
        let daw_file = DawFile::new(title.clone());
        
        assert_eq!(daw_file.metadata.title, title);
        assert_eq!(daw_file.metadata.revision, 0);
        assert_eq!(daw_file.bpm, 120);
        assert_eq!(daw_file.mixdown.sample_rate, 44100);
        assert_eq!(daw_file.mixdown.bit_depth, 16);
        assert!(daw_file.instruments.is_empty());
        assert!(daw_file.events.is_empty());
    }

    #[test]
    fn test_save_daw_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.daw.json");
        
        // Create and save a new DAW file
        let mut daw_file = DawFile::new("Test Song".to_string());
        daw_file.save(&file_path).unwrap();
        
        // Verify the file exists and can be read back
        assert!(file_path.exists());
        let read_daw = read_daw_file(&file_path).unwrap();
        
        // Verify metadata was updated
        assert_eq!(read_daw.metadata.revision, 1);
        assert!(chrono::DateTime::parse_from_rfc3339(&read_daw.metadata.modification_date).is_ok());
        
        // Verify other fields remain unchanged
        assert_eq!(read_daw.metadata.title, "Test Song");
        assert_eq!(read_daw.bpm, 120);
        assert_eq!(read_daw.mixdown.sample_rate, 44100);
        assert_eq!(read_daw.mixdown.bit_depth, 16);
    }

    #[test]
    fn test_save_daw_file_increments_revision() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.daw.json");
        
        // Create and save a new DAW file multiple times
        let mut daw_file = DawFile::new("Test Song".to_string());
        daw_file.save(&file_path).unwrap();
        daw_file.save(&file_path).unwrap();
        daw_file.save(&file_path).unwrap();
        
        // Verify the revision was incremented each time
        let read_daw = read_daw_file(&file_path).unwrap();
        assert_eq!(read_daw.metadata.revision, 3);
    }

    #[test]
    fn test_metadata_management() {
        let mut daw_file = DawFile::new("Original Title".to_string());
        let original_date = daw_file.metadata.modification_date.clone();
        
        // Test title update
        daw_file.set_title("New Title".to_string());
        assert_eq!(daw_file.metadata.title, "New Title");
        assert!(daw_file.metadata.modification_date != original_date);
        
        // Test BPM update
        let before_bpm_date = daw_file.metadata.modification_date.clone();
        daw_file.set_bpm(140);
        assert_eq!(daw_file.bpm, 140);
        assert!(daw_file.metadata.modification_date != before_bpm_date);
        
        // Test mixdown settings update
        let before_mixdown_date = daw_file.metadata.modification_date.clone();
        daw_file.set_mixdown_settings(48000, 24);
        assert_eq!(daw_file.mixdown.sample_rate, 48000);
        assert_eq!(daw_file.mixdown.bit_depth, 24);
        assert!(daw_file.metadata.modification_date != before_mixdown_date);
    }

    #[test]
    fn test_add_instrument() {
        let mut daw_file = DawFile::new("Test Song".to_string());
        let sampler = Instrument::new_sampler(PathBuf::from("kick.wav"));

        // Test adding a valid instrument
        assert!(daw_file.add_instrument("sampler1".to_string(), sampler.clone()).is_ok());
        assert!(daw_file.instruments.contains_key("sampler1"));

        // Test adding an instrument with duplicate ID
        assert!(daw_file.add_instrument("sampler1".to_string(), sampler.clone()).is_err());

        // Test adding an invalid instrument
        let invalid_sampler = Instrument {
            instrument_type: "sampler".to_string(),
            parameters: serde_json::json!({}),
        };
        assert!(daw_file.add_instrument("sampler2".to_string(), invalid_sampler).is_err());
    }

    #[test]
    fn test_remove_instrument() {
        let mut daw_file = DawFile::new("Test Song".to_string());
        let sampler = Instrument::new_sampler(PathBuf::from("kick.wav"));

        // Add an instrument
        daw_file.add_instrument("sampler1".to_string(), sampler).unwrap();

        // Test removing a non-existent instrument
        assert!(daw_file.remove_instrument("nonexistent").is_err());

        // Add an event using the instrument
        daw_file.events.push(Event {
            time: "1.1".to_string(),
            instrument: "sampler1".to_string(),
            notes: vec![Note::new(Pitch::new(Tone::C, 4), 8)],
        });

        // Test removing an instrument that is in use
        assert!(daw_file.remove_instrument("sampler1").is_err());

        // Remove the event and try again
        daw_file.events.clear();
        assert!(daw_file.remove_instrument("sampler1").is_ok());
        assert!(!daw_file.instruments.contains_key("sampler1"));
    }

    #[test]
    fn test_rename_instrument() {
        let mut daw_file = DawFile::new("Test Song".to_string());
        let sampler = Instrument::new_sampler(PathBuf::from("kick.wav"));

        // Add an instrument and an event using it
        daw_file.add_instrument("sampler1".to_string(), sampler).unwrap();
        daw_file.events.push(Event {
            time: "1.1".to_string(),
            instrument: "sampler1".to_string(),
            notes: vec![Note::new(Pitch::new(Tone::C, 4), 8)],
        });

        // Test renaming to a new ID
        assert!(daw_file.rename_instrument("sampler1", "new_sampler".to_string()).is_ok());
        assert!(!daw_file.instruments.contains_key("sampler1"));
        assert!(daw_file.instruments.contains_key("new_sampler"));

        // Verify event was updated
        assert_eq!(daw_file.events[0].instrument, "new_sampler");

        // Test renaming non-existent instrument
        assert!(daw_file.rename_instrument("nonexistent", "other".to_string()).is_err());

        // Test renaming to an existing ID
        let another_sampler = Instrument::new_sampler(PathBuf::from("snare.wav"));
        daw_file.add_instrument("sampler2".to_string(), another_sampler).unwrap();
        assert!(daw_file.rename_instrument("sampler2", "new_sampler".to_string()).is_err());
    }

    #[test]
    fn test_instrument_getters() {
        let mut daw_file = DawFile::new("Test Song".to_string());
        let sampler = Instrument::new_sampler(PathBuf::from("kick.wav"));

        // Add an instrument
        daw_file.add_instrument("sampler1".to_string(), sampler).unwrap();

        // Test get_instrument
        let instrument = daw_file.get_instrument("sampler1").unwrap();
        assert_eq!(instrument.instrument_type, "sampler");
        
        // Test get_instrument_mut
        let instrument_mut = daw_file.get_instrument_mut("sampler1").unwrap();
        instrument_mut.parameters = serde_json::json!({ "sample_file": "new_kick.wav" });

        // Verify the change
        assert_eq!(
            daw_file.get_instrument("sampler1").unwrap().parameters,
            serde_json::json!({ "sample_file": "new_kick.wav" })
        );

        // Test non-existent instrument
        assert!(daw_file.get_instrument("nonexistent").is_none());
        assert!(daw_file.get_instrument_mut("nonexistent").is_none());
    }

    #[test]
    fn test_list_instruments() {
        let mut daw_file = DawFile::new("Test Song".to_string());

        // Test empty list
        assert!(daw_file.list_instruments().is_empty());

        // Add some instruments
        let sampler1 = Instrument::new_sampler(PathBuf::from("kick.wav"));
        let sampler2 = Instrument::new_sampler(PathBuf::from("snare.wav"));

        daw_file.add_instrument("sampler1".to_string(), sampler1).unwrap();
        daw_file.add_instrument("sampler2".to_string(), sampler2).unwrap();

        // Test list
        let instruments = daw_file.list_instruments();
        assert_eq!(instruments.len(), 2);
        assert!(instruments.contains(&"sampler1"));
        assert!(instruments.contains(&"sampler2"));
    }

    #[test]
    fn test_create_instruments() {
        let mut daw = DawFile::new("Test".to_string());
        
        // Test creating a sampler instrument
        let sample_path = PathBuf::from("audio/kick.wav");
        assert!(daw.create_sampler_instrument(
            "sampler1".to_string(),
            sample_path.clone()
        ).is_ok());

        // Verify the instrument was created correctly
        let sampler = daw.get_instrument("sampler1").unwrap();
        assert_eq!(sampler.instrument_type, "sampler");
        let params = sampler.parameters.as_object().unwrap();
        assert_eq!(params["sample_file"], sample_path.to_string_lossy().to_string());
    }

    #[test]
    fn test_event_management() {
        let mut daw = create_test_daw_file();
        
        // Create test instrument
        let test_instrument = Instrument::new_sampler(PathBuf::from("test.wav"));
        daw.add_instrument("test_instrument".to_string(), test_instrument).unwrap();
        
        // Test adding events
        let event1 = Event {
            time: "1.0".to_string(),
            instrument: "test_instrument".to_string(),
            notes: vec![Note::new(Pitch::new(Tone::C, 4), 8)],
        };
        daw.add_event(event1.clone()).unwrap();
        println!("After adding event1: {:?}", daw.events);
        assert_eq!(daw.events.len(), 1);

        // Test adding note to existing event
        let note2 = Note::new(Pitch::new(Tone::E, 4), 8);
        daw.add_note("1.0", "test_instrument", note2.clone()).unwrap();
        println!("After adding note2: {:?}", daw.events);
        assert_eq!(daw.events[0].notes.len(), 2);

        // Test removing note
        daw.remove_note("1.0", "test_instrument", &note2).unwrap();
        println!("After removing note2: {:?}", daw.events);
        assert_eq!(daw.events[0].notes.len(), 1);

        // Test updating note
        let old_note = daw.events[0].notes[0].clone();
        let new_note = Note::new(Pitch::new(Tone::G, 4), 16);
        daw.update_note("1.0", "test_instrument", &old_note, new_note.clone()).unwrap();
        println!("After updating note: {:?}", daw.events);
        assert_eq!(daw.events[0].notes[0].pitch.tone, Tone::G);
        assert_eq!(daw.events[0].notes[0].duration, 16);

        // Test getting events by range
        let event2 = Event {
            time: "2.0".to_string(),
            instrument: "test_instrument".to_string(),
            notes: vec![Note::new(Pitch::new(Tone::D, 4), 8)],
        };
        daw.add_event(event2).unwrap();
        println!("After adding event2: {:?}", daw.events);
        let range_events = daw.get_events_in_range("1.0", "2.0").unwrap();
        assert_eq!(range_events.len(), 2);

        // Test getting events by instrument
        let events_by_inst = daw.get_events_by_instrument("test_instrument");
        assert_eq!(events_by_inst.len(), 2);

        // Test getting events by bar
        let events_in_bar = daw.get_events_in_bar(1).unwrap();
        assert_eq!(events_in_bar.len(), 1);

        // Test removing event
        daw.remove_event("1.0", "test_instrument").unwrap();
        println!("After removing event: {:?}", daw.events);
        assert_eq!(daw.events.len(), 1);
    }

    #[test]
    fn test_time_validation() {
        let daw = create_test_daw_file();

        // Valid times
        assert!(daw.validate_time_format("1.0").is_ok());
        assert!(daw.validate_time_format("2.31").is_ok());
        assert!(daw.validate_time_format("10.15").is_ok());

        // Invalid times
        assert!(daw.validate_time_format("0.0").is_err()); // Bar 0
        assert!(daw.validate_time_format("1.32").is_err()); // 32nd note too high
        assert!(daw.validate_time_format("1").is_err()); // Missing 32nd note
        assert!(daw.validate_time_format("1.a").is_err()); // Invalid 32nd note
        assert!(daw.validate_time_format("a.0").is_err()); // Invalid bar
    }
} 