// score.rs

use std::collections::HashMap;
use std::path::PathBuf;
use dawww_core::{
    pitch::{Pitch, Tone},
    DawFile, Note as DawNote, Instrument,
};
use crate::selection_range::SelectionRange;

#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub pitch: Pitch,
    pub onset_b32: u64,
    pub duration_b32: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteState {
    Onset,
    Sustain,
    Release
}

#[derive(Debug, Clone)]
pub struct ActiveNote {
    pub note: Note,
    pub state: NoteState,
}

#[derive(Debug, Clone)]
pub struct Score {
    daw_file: DawFile,
    save_path: Option<PathBuf>,
}

impl Score {
    pub fn new() -> Self {
        let mut daw_file = DawFile::new("Untitled".to_string());
        daw_file.add_instrument("default".to_string(), Instrument::new_sampler("default".into())).unwrap();
        
        Self {
            daw_file,
            save_path: None,
        }
    }

    pub fn from_daw_file(mut daw_file: DawFile) -> Self {
        // Ensure the default instrument exists
        if daw_file.get_instrument("default").is_none() {
            daw_file.add_instrument("default".to_string(), Instrument::new_sampler("default".into()))
                .expect("Failed to add default instrument");
        }

        Self {
            daw_file,
            save_path: None,
        }
    }

    fn b32_to_time_str(&self, b32: u64) -> String {
        // Convert b32 to bar.32nd format for DawFile
        let bar = (b32 / 32) + 1;
        let thirty_second = b32 % 32;
        format!("{}.{}", bar, thirty_second)
    }

    fn time_str_to_b32(&self, time: &str) -> u64 {
        // Convert bar.32nd format from DawFile to b32
        let parts: Vec<&str> = time.split('.').collect();
        let bar = parts[0].parse::<u64>().unwrap();
        let thirty_second = parts[1].parse::<u64>().unwrap();
        ((bar - 1) * 32) + thirty_second
    }

    pub fn get_bpm(&self) -> u16 {
        self.daw_file.bpm as u16
    }

    pub fn set_bpm(&mut self, bpm: u16) {
        self.daw_file.set_bpm(bpm as u32);
        self.try_save();
    }

    pub fn set_save_path(&mut self, path: PathBuf) {
        self.save_path = Some(path);
    }

    fn try_save(&mut self) {
        if let Some(path) = &self.save_path {
            if let Err(e) = self.daw_file.save(path) {
                log::error!("Auto-save failed: {}", e);
            }
        }
    }

    pub fn notes_starting_at_time(&self, onset_b32: u64) -> Vec<Note> {
        let time_str = self.b32_to_time_str(onset_b32);
        let events = self.daw_file.get_events_by_instrument("default");
        
        events.iter()
            .filter(|e| e.time == time_str)
            .flat_map(|e| e.notes.iter().map(|n| Note {
                pitch: n.pitch,
                onset_b32,
                duration_b32: n.duration as u64,
            }))
            .collect()
    }

    pub fn time_within_song(&self, time_point_b32: u64) -> bool {
        let events = self.daw_file.get_events_by_instrument("default");
        if events.is_empty() {
            return false;
        }

        let last_event = events.last().unwrap();
        let last_time = self.time_str_to_b32(&last_event.time);
        let last_duration = events.iter()
            .flat_map(|e| e.notes.iter().map(|n| n.duration as u64))
            .max()
            .unwrap_or(0);

        time_point_b32 < last_time + last_duration
    }

    pub fn insert_or_remove(&mut self, pitch: Pitch, onset_b32: u64, duration_b32: u64) {
        let time_str = self.b32_to_time_str(onset_b32);
        let daw_note = DawNote::new(pitch, duration_b32 as u32);

        // Check if note exists
        let events = self.daw_file.get_events_by_instrument("default");
        let note_exists = events.iter()
            .filter(|e| e.time == time_str)
            .flat_map(|e| &e.notes)
            .any(|n| n.pitch == pitch && n.duration == duration_b32 as u32);

        if note_exists {
            // Remove the note
            self.daw_file.remove_note(&time_str, "default", &daw_note).unwrap();
        } else {
            // Add the note
            self.daw_file.add_note(&time_str, "default", daw_note).unwrap();
        }

        self.try_save();
    }

    pub fn clone_at_selection(&self, selection_range: SelectionRange) -> Score {
        let mut new_score = Score::new();

        let start_time = self.b32_to_time_str(selection_range.time_point_start_b32);
        let end_time = self.b32_to_time_str(selection_range.time_point_end_b32);

        if let Ok(events) = self.daw_file.get_events_in_range(&start_time, &end_time) {
            for event in events {
                if event.instrument == "default" {
                    for note in &event.notes {
                        if note.pitch >= selection_range.pitch_low && note.pitch <= selection_range.pitch_high {
                            let onset_b32 = self.time_str_to_b32(&event.time);
                            new_score.insert_or_remove(note.pitch, onset_b32, note.duration as u64);
                        }
                    }
                }
            }
        }

        new_score
    }

    pub fn translate(&self, time_point_start_b32: Option<u64>) -> Score {
        match time_point_start_b32 {
            Some(new_start_time) => {
                let mut new_score = Score::new();
                let events = self.daw_file.get_events_by_instrument("default");

                if events.is_empty() {
                    return self.clone();
                }

                let min_onset = events.iter()
                    .map(|e| self.time_str_to_b32(&e.time))
                    .min()
                    .unwrap();

                let time_offset = if min_onset > new_start_time {
                    min_onset - new_start_time
                } else {
                    new_start_time - min_onset
                };

                for event in events {
                    let old_onset = self.time_str_to_b32(&event.time);
                    let new_onset = if min_onset > new_start_time {
                        old_onset - time_offset
                    } else {
                        old_onset + time_offset
                    };

                    for note in &event.notes {
                        new_score.insert_or_remove(note.pitch, new_onset, note.duration as u64);
                    }
                }

                new_score
            }
            None => self.clone(),
        }
    }

    pub fn insert(&mut self, pitch: Pitch, onset_b32: u64, duration_b32: u64) {
        let time_str = self.b32_to_time_str(onset_b32);
        let end_b32 = onset_b32 + duration_b32;

        // Find all overlapping notes with the same pitch
        let events = self.daw_file.get_events_by_instrument("default");
        let mut overlapping_notes = Vec::new();

        for event in events {
            let event_onset = self.time_str_to_b32(&event.time);
            for note in &event.notes {
                if note.pitch == pitch {
                    let event_end = event_onset + note.duration as u64;
                    if !(event_end <= onset_b32 || event_onset >= end_b32) {
                        overlapping_notes.push((event.time.clone(), note.clone()));
                    }
                }
            }
        }

        // Remove all overlapping notes
        for (time, note) in &overlapping_notes {
            self.daw_file.remove_note(time, "default", note).unwrap();
        }

        // Calculate merged note boundaries
        let merged_onset = if overlapping_notes.is_empty() {
            onset_b32
        } else {
            overlapping_notes.iter()
                .map(|(time, _)| self.time_str_to_b32(time))
                .min()
                .unwrap()
                .min(onset_b32)
        };

        let merged_end = if overlapping_notes.is_empty() {
            end_b32
        } else {
            overlapping_notes.iter()
                .map(|(time, note)| self.time_str_to_b32(time) + note.duration as u64)
                .max()
                .unwrap()
                .max(end_b32)
        };

        // Add the merged note
        let merged_time = self.b32_to_time_str(merged_onset);
        let merged_duration = merged_end - merged_onset;
        let daw_note = DawNote::new(pitch, merged_duration as u32);
        self.daw_file.add_note(&merged_time, "default", daw_note).unwrap();
        self.try_save();
    }

    pub fn merge_down(&self, other: &Score) -> Score {
        let mut merged_score = self.clone();
        let other_events = other.daw_file.get_events_by_instrument("default");

        for event in other_events {
            for note in &event.notes {
                let onset_b32 = self.time_str_to_b32(&event.time);
                merged_score.insert(note.pitch, onset_b32, note.duration as u64);
            }
        }

        merged_score
    }

    pub fn duration(&self) -> u64 {
        let events = self.daw_file.get_events_by_instrument("default");
        if events.is_empty() {
            return 0;
        }

        let first_onset = events.iter()
            .map(|e| self.time_str_to_b32(&e.time))
            .min()
            .unwrap();

        let last_final_time = events.iter()
            .map(|e| {
                let onset = self.time_str_to_b32(&e.time);
                let max_duration = e.notes.iter()
                    .map(|n| n.duration as u64)
                    .max()
                    .unwrap_or(0);
                onset + max_duration
            })
            .max()
            .unwrap();

        last_final_time - first_onset
    }

    pub fn notes_active_at_time(&self, time_point_b32: u64) -> Vec<ActiveNote> {
        let events = self.daw_file.get_events_by_instrument("default");
        
        let mut active_notes = Vec::new();
        
        for event in events {
            let event_time = self.time_str_to_b32(&event.time);
            for note in &event.notes {
                let note_end = event_time + note.duration as u64;
                
                if time_point_b32 >= event_time && time_point_b32 <= note_end {
                    let state = if time_point_b32 == event_time {
                        NoteState::Onset
                    } else if time_point_b32 == note_end {
                        NoteState::Release
                    } else {
                        NoteState::Sustain
                    };

                    active_notes.push(ActiveNote {
                        note: Note {
                            pitch: note.pitch,
                            onset_b32: event_time,
                            duration_b32: note.duration as u64,
                        },
                        state,
                    });
                }
            }
        }

        active_notes
    }

    pub fn delete_in_selection(&mut self, selection_range: SelectionRange) {
        let start_time = self.b32_to_time_str(selection_range.time_point_start_b32);
        let end_time = self.b32_to_time_str(selection_range.time_point_end_b32);

        // First collect all notes to remove
        let mut notes_to_remove = Vec::new();
        if let Ok(events) = self.daw_file.get_events_in_range(&start_time, &end_time) {
            for event in events {
                if event.instrument == "default" {
                    for note in &event.notes {
                        if note.pitch >= selection_range.pitch_low && note.pitch <= selection_range.pitch_high {
                            notes_to_remove.push((event.time.clone(), note.clone()));
                        }
                    }
                }
            }
        }

        // Then remove them
        for (time, note) in notes_to_remove {
            self.daw_file.remove_note(&time, "default", &note).unwrap();
        }

        self.try_save();
    }

    pub fn save_to_file(&mut self, path: &PathBuf) -> Result<(), anyhow::Error> {
        let result = self.daw_file.save(path);
        if result.is_ok() {
            self.save_path = Some(path.clone());
        }
        result
    }

    pub fn get_notes(&self) -> HashMap<u64, Vec<Note>> {
        let mut notes = HashMap::new();
        let events = self.daw_file.get_events_by_instrument("default");
        
        for event in events {
            let onset_b32 = self.time_str_to_b32(&event.time);
            let notes_at_time = event.notes.iter().map(|n| Note {
                pitch: n.pitch,
                onset_b32,
                duration_b32: n.duration as u64,
            }).collect();
            notes.insert(onset_b32, notes_at_time);
        }
        
        notes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_score() -> Score {
        let mut daw_file = DawFile::new("Test Song".to_string());
        daw_file.add_instrument("default".to_string(), Instrument::new_sampler("default".into())).unwrap();
        
        let mut score = Score {
            daw_file,
            save_path: None,
        };
        
        // Add some test notes
        score.insert(Pitch::new(Tone::C, 4), 0, 32); // C4 (MIDI 60)
        score.insert(Pitch::new(Tone::E, 4), 32, 32); // E4 (MIDI 64)
        score.insert(Pitch::new(Tone::G, 4), 64, 32); // G4 (MIDI 67)
        score
    }

    #[test]
    fn test_notes_starting_at_time() {
        let score = create_test_score();

        let notes = score.notes_starting_at_time(0);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].pitch, Pitch::new(Tone::C, 4));

        let empty_notes = score.notes_starting_at_time(16);
        assert!(empty_notes.is_empty());
    }

    #[test]
    fn test_time_within_song() {
        let score = create_test_score();

        assert!(score.time_within_song(0));
        assert!(score.time_within_song(64));
        assert!(score.time_within_song(95));
        assert!(!score.time_within_song(96)); // Last note ends at 96
        assert!(!score.time_within_song(128));
    }

    #[test]
    fn test_insert_or_remove() {
        let mut score = Score::new();

        // Test insertion
        score.insert_or_remove(Pitch::new(Tone::C, 4), 0, 32);
        assert_eq!(score.notes_starting_at_time(0).len(), 1);

        // Test removal
        score.insert_or_remove(Pitch::new(Tone::C, 4), 0, 32);
        assert_eq!(score.notes_starting_at_time(0).len(), 0);
    }

    #[test]
    fn test_clone_at_selection() {
        let score = create_test_score();

        let selection_range = SelectionRange {
            time_point_start_b32: 0,
            time_point_end_b32: 64,
            pitch_low: Pitch::new(Tone::C, 4),
            pitch_high: Pitch::new(Tone::E, 4),
        };

        let selected = score.clone_at_selection(selection_range);

        assert_eq!(selected.notes_starting_at_time(0).len(), 1);
        assert_eq!(selected.notes_starting_at_time(32).len(), 1);
        assert_eq!(selected.notes_starting_at_time(64).len(), 0); // G4 is outside pitch range
    }

    #[test]
    fn test_translate() {
        let score = create_test_score();

        // Test translation to later time
        let translated = score.translate(Some(32));
        assert!(translated.notes_starting_at_time(0).is_empty());
        assert_eq!(
            translated.notes_starting_at_time(32)[0].pitch,
            Pitch::new(Tone::C, 4)
        );

        // Test translation with None
        let no_translation = score.translate(None);
        assert_eq!(no_translation.notes_starting_at_time(0).len(), 1);
    }

    #[test]
    fn test_insert() {
        let mut score = Score::new();

        // Test basic insertion
        score.insert(Pitch::new(Tone::C, 4), 0, 32);
        assert_eq!(score.notes_starting_at_time(0).len(), 1);

        // Test overlapping notes merge
        score.insert(Pitch::new(Tone::C, 4), 16, 32);
        let notes = score.notes_starting_at_time(0);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].duration_b32, 48); // Notes should merge
    }

    #[test]
    fn test_merge_down() {
        let mut score1 = Score::new();
        score1.insert(Pitch::new(Tone::C, 4), 0, 32);

        let mut score2 = Score::new();
        score2.insert(Pitch::new(Tone::E, 4), 0, 32);

        let merged = score1.merge_down(&score2);
        assert_eq!(merged.notes_starting_at_time(0).len(), 2);
    }

    #[test]
    fn test_duration() {
        let empty_score = Score::new();
        assert_eq!(empty_score.duration(), 0);

        let score = create_test_score();
        assert_eq!(score.duration(), 96); // From start of first note to end of last note
    }

    #[test]
    fn test_note_states() {
        let mut score = Score::new();

        // Add a note from time 0 to 32
        score.insert(Pitch::new(Tone::C, 4), 0, 32);

        // Test onset
        let notes_at_0 = score.notes_active_at_time(0);
        assert_eq!(notes_at_0.len(), 1);
        assert_eq!(notes_at_0[0].state, NoteState::Onset);
        assert_eq!(notes_at_0[0].note.pitch, Pitch::new(Tone::C, 4));

        // Test sustain
        let notes_at_16 = score.notes_active_at_time(16);
        assert_eq!(notes_at_16.len(), 1);
        assert_eq!(notes_at_16[0].state, NoteState::Sustain);
        assert_eq!(notes_at_16[0].note.pitch, Pitch::new(Tone::C, 4));

        // Test release
        let notes_at_32 = score.notes_active_at_time(32);
        assert_eq!(notes_at_32.len(), 1);
        assert_eq!(notes_at_32[0].state, NoteState::Release);
        assert_eq!(notes_at_32[0].note.pitch, Pitch::new(Tone::C, 4));

        // Test no notes active
        let notes_at_33 = score.notes_active_at_time(33);
        assert_eq!(notes_at_33.len(), 0);
    }

    #[test]
    fn test_overlapping_notes() {
        let mut score = Score::new();

        // Add two overlapping notes of the same pitch
        score.insert(Pitch::new(Tone::C, 4), 0, 32);
        score.insert(Pitch::new(Tone::C, 4), 16, 32);

        // Should be merged into one longer note
        let notes_at_0 = score.notes_active_at_time(0);
        assert_eq!(notes_at_0.len(), 1);
        assert_eq!(notes_at_0[0].state, NoteState::Onset);

        let notes_at_48 = score.notes_active_at_time(48);
        assert_eq!(notes_at_48.len(), 1);
        assert_eq!(notes_at_48[0].state, NoteState::Release);

        // Test that the note persists through the middle
        let notes_at_24 = score.notes_active_at_time(24);
        assert_eq!(notes_at_24.len(), 1);
        assert_eq!(notes_at_24[0].state, NoteState::Sustain);
    }

    #[test]
    fn test_remove_note() {
        let mut score = Score::new();

        // Add and then remove a note
        score.insert_or_remove(Pitch::new(Tone::C, 4), 0, 32);
        
        // Verify note exists
        assert_eq!(score.notes_active_at_time(16).len(), 1);
        
        // Remove the note
        score.insert_or_remove(Pitch::new(Tone::C, 4), 0, 32);
        
        // Verify note is gone from all time points
        assert_eq!(score.notes_active_at_time(0).len(), 0);
        assert_eq!(score.notes_active_at_time(16).len(), 0);
        assert_eq!(score.notes_active_at_time(32).len(), 0);
    }

    #[test]
    fn test_multiple_pitches() {
        let mut score = Score::new();

        // Add two notes at different pitches at the same time
        score.insert(Pitch::new(Tone::C, 4), 0, 32);
        score.insert(Pitch::new(Tone::E, 4), 0, 32);

        let notes_at_0 = score.notes_active_at_time(0);
        assert_eq!(notes_at_0.len(), 2);
        assert!(notes_at_0.iter().all(|n| n.state == NoteState::Onset));
        
        // Verify pitches are different
        let pitches: Vec<Pitch> = notes_at_0.iter().map(|n| n.note.pitch).collect();
        assert!(pitches.contains(&Pitch::new(Tone::C, 4)));
        assert!(pitches.contains(&Pitch::new(Tone::E, 4)));
    }
}
