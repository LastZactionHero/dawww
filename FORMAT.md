# DAW File Format Specification

This document outlines the file format for a synthesizer tracker application. This file is intended to be produced by the UI application and consumed by the mixdown application to generate a wave file. The format uses JSON for the main data and supports associated WAV files within a directory structure (optionally compressed as a ZIP archive).

**File Extension:** It is recommended to use a `.daw.json` extension for these files.

**Top-Level Structure:** The file is a JSON dictionary with the following top-level keys:

```json
{
  "metadata": { ... },
  "bpm": 120,
  "mixdown": { ... },
  "instruments": { ... },
  "events": [ ... ]
}
```

**1. `metadata` (Dictionary):** Contains information about the song.

* **`title` (String):** The title of the song.
* **`creation_date` (String, ISO 8601):** The date and time the song was created in ISO 8601 format (e.g., "2024-03-19T15:30:45Z").
* **`modification_date` (String, ISO 8601):** The date and time the song was last modified in ISO 8601 format.
* **`revision` (Integer):** A monotonically increasing revision number starting at 0. This should be incremented each time the song is saved.

**2. `bpm` (Integer):** Beats per minute for the song's tempo.

**3. `mixdown` (Dictionary):** Contains settings for the audio mixdown process.

* **`sample_rate` (Integer):** The desired sample rate of the output audio in Hz (e.g., 44100, 48000).
* **`bit_depth` (Integer):** The desired bit depth of the output audio in bits (e.g., 16, 24).

**4. `instruments` (Dictionary):** Defines the instruments used in the song. The keys are unique instrument IDs (strings), and the values are instrument definition objects.

* **Synth Instrument Definition (Example):**
    ```json
    {
      "type": "synth",
      "subtype": "fm",
      "parameters": {
        "carrier_wave": "sine",
        "modulator_wave": "sine",
        "modulator_frequency": 3.0,
        "modulator_amplitude": 0.7,
        "attack": 0.1,
        "decay": 0.5,
        "sustain": 0.8,
        "release": 0.3
      }
    }
    ```
    The `parameters` field will vary depending on the `subtype` of the synth.

* **Sampler Instrument Definition (Example):**
    ```json
    {
      "type": "sampler",
      "parameters": {
        "sample_file": "audio/my_sample.wav",
        "loop": false,
        "loop_start": 0.5, // in seconds
        "loop_end": 1.2     // in seconds
      }
    }
    ```
    The `sample_file` path is relative to the location of the `.daw.json` file.

**5. `events` (Array):** A list of musical events, ordered chronologically. Each event is a dictionary with the following structure:

* **`time` (String):** The onset time of the event in `bar.32nd` notation (e.g., "1.0", "1.15", "2.31"). `B.N` where `B` is the bar number (starting from 1) and `N` is the 32nd note index within the bar (starting from 0 to 31).
* **`instrument` (String):** The ID of the instrument (as defined in the `instruments` dictionary) that will play this event.
* **`pitches` (Array of Dictionaries):** A list of pitch and duration pairs for the event. This allows for chords or multiple notes with different durations at the same onset. Each dictionary in the array has:
    * **`pitch` (String):** The pitch in scientific pitch notation (e.g., "C4", "A#5").
    * **`duration` (Integer):** The duration of the note in 32nd notes.

**Timing Calculation:**

The mixdown application should calculate the time in seconds for each event using the following formula:

Given a `bpm` and a `time` string in "B.N" format:

1.  Parse `B` (bar number) and `N` (32nd note index).
2.  Calculate the duration of one 32nd note in seconds: `seconds_per_32nd_note = 60 / (bpm * 8)`
3.  Calculate the time in seconds from the beginning of the song: `time_in_seconds = ((B - 1) * 32 + N) * seconds_per_32nd_note`

**Handling External Audio Files:**

For instruments like samplers or wave playback, the file format uses relative paths to reference audio files. These audio files (e.g., WAV files) should be stored in the same directory as the `.daw.json` file or within a subdirectory. The entire directory can optionally be compressed into a ZIP archive. The mixdown application should be able to handle this directory structure or the ZIP archive.

## Example File

Here's an example of a `.daw.json` file:

```json
{
  "metadata": {
    "title": "My First Song",
    "creation_date": "2025-04-02T15:30:45Z",
    "modification_date": "2025-04-02T15:30:45Z"
  },
  "bpm": 120,
  "mixdown": {
    "sample_rate": 44100,
    "bit_depth": 16
  },
  "instruments": {
    "synth1": {
      "type": "synth",
      "subtype": "subtractive",
      "parameters": {
        "oscillator_wave": "sawtooth",
        "filter_type": "lowpass",
        "filter_cutoff": 880.0,
        "filter_resonance": 0.3,
        "envelope_attack": 0.01,
        "envelope_decay": 0.2,
        "envelope_sustain": 0.7,
        "envelope_release": 0.1
      }
    },
    "sampler1": {
      "type": "sampler",
      "parameters": {
        "sample_file": "audio/kick.wav",
        "loop": false
      }
    }
  },
  "events": [
    {
      "time": "1.0",
      "instrument": "synth1",
      "pitches": [
        { "pitch": "C4", "duration": 8 },
        { "pitch": "E4", "duration": 8 },
        { "pitch": "G4", "duration": 8 }
      ]
    },
    {
      "time": "1.8",
      "instrument": "synth1",
      "pitches": [
        { "pitch": "D4", "duration": 8 }
      ]
    },
    {
      "time": "2.0",
      "instrument": "sampler1",
      "pitches": [
        { "pitch": "C3", "duration": 8 } // Pitch might not be relevant for a sampler, but we need the structure
      ]
    }
  ]
}
```