// score.rs

use std::{collections::HashMap, f32::MAX};
use log::debug;

use crate::{
    pitch::{Pitch, Tone},
    selection_buffer,
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
    pub bpm: u16,
    pub notes: HashMap<u64, Vec<Note>>,
    pub active_notes: HashMap<u64, Vec<ActiveNote>>,
}

impl Score {
    pub fn notes_starting_at_time(&self, onset_b32: u64) -> Vec<Note> {
        self.notes
            .get(&onset_b32)
            .unwrap_or(&vec![])
            .iter()
            .map(|note| note.clone())
            .collect()
    }

    pub fn time_within_song(&self, time_point_b32: u64) -> bool {
        let mut last_time_point_in_song = 0;

        for (_, notes_at_onset) in &self.notes {
            for note in notes_at_onset {
                if note.onset_b32 + note.duration_b32 > last_time_point_in_song {
                    last_time_point_in_song = note.onset_b32 + note.duration_b32
                }
            }
        }
        last_time_point_in_song > time_point_b32
    }

    pub fn insert_or_remove(&mut self, pitch: Pitch, onset_b32: u64, duration_b32: u64) {
        let mut notes_starting_at_time = self.notes_starting_at_time(onset_b32);

        let mut note_found_at_index = None;
        for (index, note) in notes_starting_at_time.iter().enumerate() {
            if note.pitch == pitch {
                note_found_at_index = Some(index);
            }
        }
        if let Some(matching_note_index) = note_found_at_index {
            let removed_note = notes_starting_at_time[matching_note_index];
            for t in removed_note.onset_b32..=removed_note.onset_b32 + removed_note.duration_b32 {
                if let Some(notes) = self.active_notes.get_mut(&t) {
                    notes.retain(|active| active.note.pitch != pitch);
                }
            }
            notes_starting_at_time.remove(matching_note_index);
            self.notes.insert(onset_b32, notes_starting_at_time);
            return;
        }

        let note_to_insert = Note {
            pitch,
            onset_b32,
            duration_b32,
        };

        match self.notes.get_mut(&onset_b32) {
            Some(notes_at_onset) => {
                notes_at_onset.push(note_to_insert);
            }
            None => {
                self.notes.insert(onset_b32, vec![note_to_insert]);
            }
        }

        self.update_active_notes(note_to_insert);
    }

    // Creates a new Score with just notes between selection times and pitches.
    pub fn clone_at_selection(&self, selection_range: SelectionRange) -> Score {
        let mut new_score = Score {
            bpm: self.bpm,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };

        for (&onset_b32, notes_at_onset) in &self.notes {
            if onset_b32 >= selection_range.time_point_start_b32 && onset_b32 < selection_range.time_point_end_b32 {
                for note in notes_at_onset {
                    if note.pitch >= selection_range.pitch_low && note.pitch <= selection_range.pitch_high {
                        // Assuming Pitch implements PartialOrd
                        new_score.insert_or_remove(note.pitch, note.onset_b32, note.duration_b32);
                    }
                }
            }
        }

        new_score
    }

    pub fn translate(&self, time_point_start_b32: Option<u64>) -> Score {
        match time_point_start_b32 {
            Some(new_start_time) => {
                let mut new_score = Score {
                    bpm: self.bpm,
                    notes: HashMap::new(),
                    active_notes: HashMap::new(),
                };

                let mut min_onset = u64::MAX;
                for (&onset_b32, _) in &self.notes {
                    min_onset = min_onset.min(onset_b32);
                }

                if min_onset == u64::MAX {
                    // No notes in the original score
                    return self.clone(); // Return a copy if no notes exist
                }

                let time_offset = if min_onset > new_start_time {
                    min_onset - new_start_time
                } else {
                    new_start_time - min_onset
                };

                for (&onset_b32, notes_at_onset) in &self.notes {
                    let new_onset = if min_onset > new_start_time {
                        onset_b32 - time_offset
                    } else {
                        onset_b32 + time_offset
                    };

                    for note in notes_at_onset {
                        new_score.insert_or_remove(note.pitch, new_onset, note.duration_b32);
                    }
                }

                new_score
            }
            None => self.clone(), // Return a copy if no new start time is provided
        }
    }

    pub fn insert(&mut self, pitch: Pitch, onset_b32: u64, duration_b32: u64) {
        let end_b32 = onset_b32 + duration_b32;
        let mut overlapping_notes: Vec<(u64, Note)> = Vec::new();

        // Find all overlapping notes with the same pitch
        for (&existing_onset, notes) in &self.notes {
            for note in notes {
                if note.pitch == pitch {
                    let existing_end = note.onset_b32 + note.duration_b32;
                    // Check if notes strictly overlap (not just adjacent)
                    if !(existing_end <= onset_b32 || note.onset_b32 >= end_b32) {
                        overlapping_notes.push((existing_onset, *note));
                    }
                }
            }
        }

        // Remove all overlapping notes
        for (onset, note) in &overlapping_notes {
            if let Some(notes) = self.notes.get_mut(onset) {
                notes.retain(|n| n.pitch != note.pitch);
                if notes.is_empty() {
                    self.notes.remove(onset);
                }
            }
        }

        // Calculate merged note boundaries
        let merged_onset = if overlapping_notes.is_empty() {
            onset_b32
        } else {
            overlapping_notes
                .iter()
                .map(|(_, note)| note.onset_b32)
                .min()
                .unwrap()
                .min(onset_b32)
        };

        let merged_end = if overlapping_notes.is_empty() {
            end_b32
        } else {
            overlapping_notes
                .iter()
                .map(|(_, note)| note.onset_b32 + note.duration_b32)
                .max()
                .unwrap()
                .max(end_b32)
        };

        // Insert the merged note
        let merged_note = Note {
            pitch,
            onset_b32: merged_onset,
            duration_b32: merged_end - merged_onset,
        };

        match self.notes.get_mut(&merged_onset) {
            Some(notes_at_onset) => {
                notes_at_onset.push(merged_note);
            }
            None => {
                self.notes.insert(merged_onset, vec![merged_note]);
            }
        }

        self.update_active_notes(merged_note);
    }

    pub fn merge_down(&self, other: &Score) -> Score {
        let mut merged_score = self.clone();

        for (&onset_b32, notes_at_onset) in &other.notes {
            for note in notes_at_onset {
                merged_score.insert(note.pitch, onset_b32, note.duration_b32);
            }
        }

        merged_score
    }

    pub fn duration(&self) -> u64 {
        if self.notes.is_empty() {
            return 0; // Return 0 if the score is empty
        }

        let mut first_onset = u64::MAX;
        let mut last_final_time = 0;

        for (&onset_b32, notes_at_onset) in &self.notes {
            first_onset = first_onset.min(onset_b32);
            for note in notes_at_onset {
                last_final_time = last_final_time.max(note.onset_b32 + note.duration_b32);
            }
        }

        if first_onset == u64::MAX {
            // No notes found
            return 0;
        }

        last_final_time - first_onset
    }

    // Helper method to update active_notes when inserting/removing notes
    fn update_active_notes(&mut self, note: Note) {
        // Add new entries
        for t in note.onset_b32..=note.onset_b32 + note.duration_b32 - 1 {
            let state = if t == note.onset_b32 {
                NoteState::Onset
            } else if t == note.onset_b32 + note.duration_b32 - 1 {
                NoteState::Release
            } else {
                NoteState::Sustain
            };

            let active_note = ActiveNote {
                note,
                state,
            };
            
            if let Some(notes) = self.active_notes.get_mut(&t) {
                for note in notes {
                    if note.note.pitch == active_note.note.pitch {
                        if note.state == NoteState::Sustain {
                            continue;
                        }
                    }
                }
            }

            self.active_notes
                .entry(t)
                .or_insert_with(Vec::new)
                .push(active_note);
        }
    }

    // New method to get active notes at a specific time
    pub fn notes_active_at_time(&self, time_point_b32: u64) -> Vec<ActiveNote> {
        let result = self.active_notes
            .get(&time_point_b32)
            .cloned()
            .unwrap_or_default();
        result
    }

    pub fn delete_in_selection(&mut self, selection_range: SelectionRange) {
        debug!("Deleting notes between {} and {} with pitch range {:?} to {:?}", 
            selection_range.time_point_start_b32, selection_range.time_point_end_b32, 
            selection_range.pitch_low, selection_range.pitch_high);

        let mut onsets_to_remove: Vec<u64> = Vec::new();
        let mut notes_to_keep: HashMap<u64, Vec<Note>> = HashMap::new();

        // Identify notes to remove and keep
        for (&onset_b32, notes_at_onset) in &self.notes {
            if onset_b32 >= selection_range.time_point_start_b32 && onset_b32 < selection_range.time_point_end_b32 {
                let (keep, remove): (Vec<Note>, Vec<Note>) = notes_at_onset
                    .iter()
                    .cloned()
                    .partition(|note| note.pitch < selection_range.pitch_low || note.pitch > selection_range.pitch_high);

                debug!("At onset {}: keeping {} notes, removing {} notes", 
                    onset_b32, keep.len(), remove.len());

                if !keep.is_empty() {
                    notes_to_keep.insert(onset_b32, keep);
                } else {
                    onsets_to_remove.push(onset_b32);
                }
            }
        }

        debug!("Total onsets to remove: {}", onsets_to_remove.len());
        debug!("Total onsets with kept notes: {}", notes_to_keep.len());

        // Remove notes and update active_notes
        for onset_b32 in onsets_to_remove {
            self.notes.remove(&onset_b32);
        }

        // Update remaining onsets with kept notes
        for (onset_b32, notes) in notes_to_keep {
            self.notes.insert(onset_b32, notes);
        }

        // Collect all remaining notes first
        let all_notes: Vec<Note> = self.notes.values()
            .flat_map(|notes| notes.iter().cloned())
            .collect();

        debug!("Rebuilding active_notes with {} total notes", all_notes.len());

        // Clear and rebuild active_notes
        self.active_notes.clear();
        for note in all_notes {
            self.update_active_notes(note);
        }

        debug!("Finished rebuilding active_notes");
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    fn create_test_score() -> Score {
        let mut score = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
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
        let mut score = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };

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
        let mut score = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };

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
        let mut score1 = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };
        score1.insert(Pitch::new(Tone::C, 4), 0, 32);

        let mut score2 = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };
        score2.insert(Pitch::new(Tone::E, 4), 0, 32);

        let merged = score1.merge_down(&score2);
        assert_eq!(merged.notes_starting_at_time(0).len(), 2);
    }

    #[test]
    fn test_duration() {
        let empty_score = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };
        assert_eq!(empty_score.duration(), 0);

        let score = create_test_score();
        assert_eq!(score.duration(), 96); // From start of first note to end of last note
    }

    #[test]
    fn test_note_states() {
        let mut score = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };

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
        let mut score = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };

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
        let mut score = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };

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
        let mut score = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };

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
