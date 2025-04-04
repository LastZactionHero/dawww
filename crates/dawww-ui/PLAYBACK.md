# WAV File Playback System Specification

## Overview
A system to play WAV files generated from mixdowns, with automatic file detection and position tracking. The system will run continuously in the background and communicate with the UI via an mpsc channel.

## Core Requirements
1. Play WAV files from the mixdown directory
2. Support seeking to specific positions
3. Automatically detect and load new mixdown files
4. Send position updates to the UI
5. Handle playback state (play/pause/stop)

## Architecture

### Components
1. **Playback System**
   - Single struct managing both audio device and file playback
   - Runs in a dedicated thread
   - Uses CPAL for audio output
   - Matches WAV file sample rate and channel count

2. **File Monitor**
   - Runs in a separate thread
   - Checks mixdown directory every 100ms
   - Uses file modification time for change detection
   - Loads highest revision number file

3. **Communication**
   - mpsc channel for position updates and state changes
   - Single mutex for all shared state
   - Simple error handling (log and stop)

### Threading Model
- Separate threads for:
  1. Audio playback
  2. File monitoring
- Synchronization via mutex
- Continuous background operation

## Data Flow

### File Management
- Monitor `/song/mixdown` directory
- Track highest revision number file
- Load entire WAV file into memory
- Reload on file changes
- Pause playback during reload

### Audio Playback
- Use CPAL with default buffer settings
- Match WAV file sample rate and channels
- Fixed volume level (system volume control)
- Send position updates every ~300 samples
- Position updates in seconds

### State Management
- Playback states: Playing, Paused, Stopped
- File state: Current file path, modification time
- Position state: Current playback time

## Communication Protocol

### Message Types
1. Position Update
   - Current time in seconds
   - Playback state

2. File Change
   - Signal that new mixdown is ready

3. State Change
   - Play/Pause/Stop
   - Seek position

## Error Handling
- Simple strategy: log errors and stop playback
- No retry mechanisms
- Error types:
  - File not found
  - Invalid WAV format
  - Audio device errors
  - Playback interruption

## Testing Plan

### Unit Tests
1. File Monitoring
   - Detect new files
   - Handle file changes
   - Load correct revision

2. Audio Playback
   - Start/stop/pause
   - Position seeking
   - Sample rate matching
   - Channel count matching

3. State Management
   - State transitions
   - Position updates
   - File reloading

### Integration Tests
1. End-to-end playback
   - File detection → playback → position updates
   - File change → reload → resume

2. Error Scenarios
   - Missing file
   - Invalid file
   - Device errors

### Performance Tests
1. Latency
   - Position update frequency
   - File change detection
   - Playback start time

2. Resource Usage
   - Memory usage
   - CPU usage
   - Thread behavior

## Implementation Notes
1. Use CPAL for audio output
2. Use hound for WAV file reading
3. Use std::sync::Mutex for synchronization
4. Use std::sync::mpsc for communication
5. Use std::thread for threading
6. Use std::time for timing
7. Use std::fs for file operations

## Dependencies
- cpal
- hound
- std (threading, sync, fs, time)

This specification provides a complete roadmap for implementation while maintaining flexibility for future enhancements like looping and volume control.

=======

## IMPLEMENTATION STEPS

First, let's identify the major building blocks:

1. Core Data Structures
   - Playback State
   - File State
   - Position State
   - Message Types

2. File System Operations
   - Directory Monitoring
   - WAV File Loading
   - Revision Tracking

3. Audio System
   - CPAL Setup
   - Audio Buffer Management
   - Sample Rate/Channel Matching

4. Threading & Communication
   - Thread Management
   - Channel Setup
   - State Synchronization

5. Integration & Testing
   - Unit Tests
   - Integration Tests
   - Performance Tests

Let me break this down into iterative steps, starting with the most fundamental components and building up to the complete system.

# Phase 1: Foundation

## Step 1: Basic Data Structures and Types

```text
Create a new Rust project with the following structure:
- src/
  - lib.rs
  - types.rs
  - error.rs
  - tests/
    - types.rs

First, implement the core data structures and types needed for the playback system:

1. Create an enum `PlaybackState` with variants:
   - Playing
   - Paused
   - Stopped

2. Create a struct `FileState` containing:
   - path: PathBuf
   - modification_time: SystemTime
   - revision: u32

3. Create a struct `PositionState` containing:
   - current_time: f64
   - total_duration: f64

4. Create an enum `Message` for communication:
   - PositionUpdate { time: f64, state: PlaybackState }
   - FileChange { path: PathBuf }
   - StateChange { state: PlaybackState }
   - Seek { position: f64 }

5. Create custom error types in error.rs:
   - FileError
   - AudioError
   - StateError

Write comprehensive unit tests for each type, focusing on:
- State transitions
- Position calculations
- Error handling
- Message serialization/deserialization

Ensure all types implement Debug, Clone, and PartialEq where appropriate.
```

## Step 2: File System Operations

```text
Create a new module `file_system.rs` with tests in `tests/file_system.rs`.

Implement the following functionality:

1. Create a `FileMonitor` struct that:
   - Takes a directory path
   - Implements a method to check for new files
   - Tracks the highest revision number
   - Returns the latest file path

2. Create a `WavFile` struct that:
   - Loads a WAV file using hound
   - Validates the format
   - Extracts metadata (sample rate, channels, duration)
   - Stores the audio data in memory

3. Implement error handling for:
   - File not found
   - Invalid WAV format
   - Permission issues

Write tests that:
- Create temporary test files
- Verify file detection
- Test revision number tracking
- Validate WAV file loading
- Check error cases
```

# Phase 2: Audio System

## Step 3: Basic Audio Setup

```text
Create a new module `audio.rs` with tests in `tests/audio.rs`.

Implement the following:

1. Create an `AudioDevice` struct that:
   - Initializes CPAL
   - Configures the default output device
   - Sets up the audio stream
   - Handles sample rate conversion

2. Create a `BufferManager` struct that:
   - Manages audio buffers
   - Handles sample rate conversion
   - Provides methods for seeking
   - Tracks current position

3. Implement basic error handling for:
   - Device initialization
   - Stream configuration
   - Buffer management

Write tests that:
- Verify device initialization
- Test buffer management
- Check sample rate conversion
- Validate position tracking
```

## Step 4: Threading and Communication

```text
Create a new module `threading.rs` with tests in `tests/threading.rs`.

Implement:

1. Create a `PlaybackThread` struct that:
   - Manages the audio playback thread
   - Handles state transitions
   - Sends position updates
   - Processes seek commands

2. Create a `FileMonitorThread` struct that:
   - Runs the file monitoring loop
   - Sends file change notifications
   - Handles reload requests

3. Implement synchronization using:
   - Mutex for shared state
   - mpsc channels for communication
   - Thread-safe error handling

Write tests that:
- Verify thread creation and shutdown
- Test message passing
- Check state synchronization
- Validate error propagation
```

# Phase 3: Integration

## Step 5: Core Playback System

```text
Create a new module `playback.rs` with tests in `tests/playback.rs`.

Implement:

1. Create a `PlaybackSystem` struct that:
   - Combines all previous components
   - Manages the overall state
   - Handles user commands
   - Coordinates between threads

2. Implement the main playback loop that:
   - Processes audio data
   - Sends position updates
   - Handles state changes
   - Manages file reloading

3. Add integration tests that:
   - Test end-to-end playback
   - Verify file change handling
   - Check position updates
   - Validate error recovery
```

## Step 6: Final Integration and Testing

```text
Create a new module `integration.rs` with tests in `tests/integration.rs`.

Implement:

1. Create a `PlaybackManager` struct that:
   - Provides a public API
   - Handles initialization
   - Manages resources
   - Exposes control methods

2. Add comprehensive integration tests:
   - Full playback scenarios
   - File change scenarios
   - Error scenarios
   - Performance tests

3. Implement logging and monitoring:
   - Error logging
   - Performance metrics
   - State transitions
   - Resource usage

Write tests that:
- Verify the complete system
- Check performance characteristics
- Validate error handling
- Test resource cleanup
```

Each step builds on the previous ones, with clear testing requirements and integration points. The progression moves from simple data structures to complex system integration, ensuring that each component is well-tested before being integrated into the larger system.

