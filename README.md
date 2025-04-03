# Dawww - Digital Audio Workstation Project

## Project Overview
Dawww is a modern digital audio workstation (DAW) implemented in Rust. The project is structured as a modular system with multiple crates, each handling specific aspects of the audio workstation functionality.

## Project Structure

### Core Components

1. **dawww-core** (`crates/dawww-core/`)
   - Core data structures and file operations
   - Handles DAW file format (.daw.json)
   - Manages instruments, events, and musical data
   - Provides fundamental operations for file management, metadata, and event handling

2. **dawww-render** (`crates/dawww-render/`)
   - Audio rendering and mixdown functionality
   - Handles audio synthesis and processing
   - Manages WAV file output
   - Provides basic stereo support

3. **sample-song-builder** (`crates/sample-song-builder/`)
   - Utility for creating example songs
   - Demonstrates the DAW's capabilities
   - Provides reference implementations

### File Format
The project uses a JSON-based file format (.daw.json) with the following structure:
- Metadata (title, dates, revision)
- BPM (tempo)
- Mixdown settings (sample rate, bit depth)
- Instruments (synth and sampler definitions)
- Events (musical events with timing and pitch information)

## Current Features

### Implemented Features
1. **File Management**
   - Create, read, and save DAW files
   - File format validation
   - Revision tracking

2. **Metadata Management**
   - Title management
   - BPM control
   - Mixdown settings configuration

3. **Instrument Management**
   - Add/remove/rename instruments
   - Support for synth and sampler instruments
   - Instrument parameter management

4. **Event/Note Management**
   - Add/remove/update events
   - Note management within events
   - Event querying by time range and instrument

5. **Pitch Operations**
   - Pitch creation and manipulation
   - Frequency calculation
   - Pitch sequence navigation

6. **Audio Engine**
   - Basic sine wave synthesis
   - WAV file output
   - Stereo support

## Technical Implementation

### Architecture
- Modular design with separate crates for different functionalities
- Clear separation of concerns between core data structures and audio processing
- JSON-based file format for easy integration and debugging

### Testing
- Unit tests for core functionality
- Integration tests for file operations
- Edge case testing
- Performance benchmarking (planned)

### Performance Considerations
- Event lookup optimization
- Storage optimization
- Caching strategies
- Audio rendering performance

## Future Development

### Planned Features
1. **Import/Export**
   - MIDI file support
   - Additional format support

2. **Advanced Musical Features**
   - Undo/redo support
   - MIDI CC and automation
   - Microtonal support

3. **Audio Processing**
   - More sophisticated synthesis
   - Advanced effects
   - Real-time processing

4. **User Interface**
   - Graphical interface
   - Real-time editing
   - Visualization tools