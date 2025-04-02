# DAWWW - DAW Mixdown Application

A command-line tool for mixing down DAW song files to WAV format. Currently supports basic sine wave synthesis, with plans to add more sophisticated synthesis capabilities.

## Installation

This project requires Rust to be installed on your system. You can install Rust from [rustup.rs](https://rustup.rs/).

To build the project:

```bash
cargo build --release
```

The binary will be available at `target/release/dawww`.

## Usage

The basic command format is:

```bash
dawww <input_directory> <output_wav_file>
```

For example:

```bash
dawww ./sample_song ./output.wav
```

### Input Directory Structure

The input directory should contain a `.daw.json` file that describes the song. The file format is documented in `FORMAT.md`.

### Output

The program will generate a WAV file at the specified output path. The WAV file will be:
- 16-bit PCM
- 44.1kHz sample rate (configurable in the DAW file)
- Stereo

## Development

This is a work in progress. Currently implemented features:
- Basic sine wave synthesis
- Support for the DAW file format
- Command-line interface

Planned features:
- More sophisticated synthesis types
- Sample playback
- Effects processing
- Better pitch parsing
- MIDI export 