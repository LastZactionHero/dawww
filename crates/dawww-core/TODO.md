# Dawww Core Library TODO

This document outlines the planned API methods for the dawww-core library, which will provide the foundation for both the editor and mixdown applications.

## File Management
- [X] `DawFile::new(title: String) -> Self`
  - Create a new empty song with default settings
- [X] `DawFile::save(&mut self, path: &Path) -> Result<()>`
  - Save to disk, handling the revision increment
- [X] `find_daw_file(dir: &PathBuf) -> Result<PathBuf>`
  - Find the .daw.json file in a directory
- [X] `read_daw_file(path: &PathBuf) -> Result<DawFile>`
  - Read and parse a DAW file

## Metadata Management
- [X] `DawFile::set_title(&mut self, title: String)`
  - Update song title
- [X] `DawFile::set_bpm(&mut self, bpm: u32)`
  - Update song tempo
- [X] `DawFile::set_mixdown_settings(&mut self, sample_rate: u32, bit_depth: u16)`
  - Update audio output settings

## Instrument Management
- [X] `DawFile::add_instrument(&mut self, id: String, instrument: Instrument) -> Result<()>`
  - Add a new instrument
- [X] `DawFile::remove_instrument(&mut self, id: &str) -> Result<()>`
  - Remove an instrument
- [X] `DawFile::rename_instrument(&mut self, old_id: &str, new_id: String) -> Result<()>`
  - Rename an instrument
- [X] `DawFile::get_instrument(&self, id: &str) -> Option<&Instrument>`
  - Get immutable reference to instrument
- [X] `DawFile::get_instrument_mut(&mut self, id: &str) -> Option<&mut Instrument>`
  - Get mutable reference to instrument
- [X] `DawFile::list_instruments(&self) -> Vec<&str>`
  - List all instrument IDs
- [X] `DawFile::create_sampler_instrument(&mut self, id: String, sample_path: PathBuf) -> Result<()>`
  - Create a new sampler instrument

## Event/Note Management
- [X] `DawFile::add_event(&mut self, event: Event) -> Result<()>`
  - Add a new event
- [X] `DawFile::remove_event(&mut self, time: &str, instrument: &str) -> Result<()>`
  - Remove an event
- [X] `DawFile::update_event(&mut self, time: &str, instrument: &str, new_event: Event) -> Result<()>`
  - Update an existing event
- [X] `DawFile::add_note(&mut self, time: &str, instrument: &str, note: Note) -> Result<()>`
  - Add a note to an event
- [X] `DawFile::remove_note(&mut self, time: &str, instrument: &str, note: Note) -> Result<()>`
  - Remove a note from an event
- [X] `DawFile::update_note(&mut self, time: &str, instrument: &str, old_note: Note, new_note: Note) -> Result<())`
  - Update a note's properties
- [X] `DawFile::get_events_in_range(&self, start_time: &str, end_time: &str) -> Vec<&Event>`
  - Get events within a time range
- [X] `DawFile::get_events_by_instrument(&self, instrument_id: &str) -> Vec<&Event>`
  - Get all events for an instrument
- [X] `DawFile::get_events_in_bar(&self, bar: u32) -> Vec<&Event>`
  - Get all events in a specific bar

## Pitch Operations
- [X] `Pitch::new(tone: Tone, octave: u16) -> Pitch`
  - Create a new pitch
- [X] `Pitch::all() -> Vec<Pitch>`
  - Get all possible pitches
- [X] `Pitch::next() -> Option<Pitch>`
  - Get next pitch in sequence
- [X] `Pitch::prev() -> Option<Pitch>`
  - Get previous pitch in sequence
- [X] `Pitch::frequency(octave: u16) -> f64`
  - Get frequency for pitch at octave
- [ ] `DawFile::transpose_note(&mut self, time: &str, instrument: &str, note: Note, semitones: i32) -> Result<()>`
  - Transpose a single note by semitones
- [ ] `DawFile::transpose_events(&mut self, start_time: &str, end_time: &str, semitones: i32) -> Result<()>`
  - Transpose all notes in a time range

## Time and Musical Position Utilities
- [ ] Create `MusicalTime` struct for handling musical time
  - [ ] `MusicalTime::from_str(time_str: &str) -> Result<Self>`
  - [ ] `MusicalTime::to_string(&self) -> String`
  - [ ] `MusicalTime::add_thirty_seconds(&mut self, count: u32)`
  - [ ] `MusicalTime::subtract_thirty_seconds(&mut self, count: u32)`
  - [ ] `MusicalTime::compare(&self, other: &MusicalTime) -> std::cmp::Ordering`
  - [ ] `MusicalTime::to_absolute_thirty_seconds(&self) -> u32`
  - [ ] `MusicalTime::to_seconds(&self, bpm: u32) -> f64`

## Audio Engine
- [X] Basic audio engine implementation
  - [X] Sine wave synthesis
  - [X] WAV file output
  - [X] Basic stereo support
- [ ] Advanced audio features
  - [ ] Multiple waveform types (square, sawtooth, triangle)
  - [ ] ADSR envelope support
  - [ ] Sample playback
  - [ ] Basic effects (reverb, delay)
  - [ ] Volume/pan control
  - [ ] Multi-track mixing

## Validation and Analysis
- [ ] `DawFile::validate(&self) -> Result<()>`
  - Validate entire file structure
- [ ] `DawFile::validate_instrument_references(&self) -> Result<()>`
  - Validate all instrument references in events
- [ ] `DawFile::validate_time_ordering(&self) -> Result<()>`
  - Validate event time ordering
- [ ] `DawFile::get_song_duration_bars(&self) -> u32`
  - Get total duration in bars
- [ ] `DawFile::get_song_duration_seconds(&self) -> f64`
  - Get total duration in seconds
- [ ] `DawFile::get_used_instruments(&self) -> HashSet<&str>`
  - Get set of instruments used in events

## Import/Export Operations
- [ ] `DawFile::import_midi(&mut self, midi_path: &Path) -> Result<()>`
  - Import from MIDI file
- [ ] `DawFile::export_midi(&self, midi_path: &Path) -> Result<()>`
  - Export to MIDI file

## Implementation Notes

### Error Types
- Create custom error types for different failure modes
- Consider using `thiserror` for error handling

### Testing Strategy
- [X] Unit tests for each implemented method
- [X] Integration tests for file operations
- [X] Tests for edge cases and error conditions
- [ ] Tests for musical time calculations
- [ ] Tests for audio engine features
- [ ] Performance benchmarks

### Performance Considerations
- [ ] Consider using indexes for event lookup
- [ ] Optimize event storage for common operations
- [ ] Consider caching for frequently accessed data
- [ ] Profile and optimize audio rendering

### Future Extensions
- [ ] Support for more instrument types
- [ ] Additional import/export formats
- [ ] More sophisticated musical operations
- [ ] Undo/redo support
- [ ] MIDI CC and automation support
- [ ] Microtonal support 