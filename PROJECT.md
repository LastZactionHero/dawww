This project represents a full-featured DAW in early development. Refer to the README for a complete description of what's been implemented so far.

I'd like you to write a basic synth tracker UI using the existing crates.

## Basic Project Details
- Named something like dawww-ui
- Requires a path to a song directory. If it exists, loads it. Otherwise, creates a default song
- On new song, name is just the folder name
- Displays a tracker interface as a grid.
- Grid rows are Pitches. Refer to Pitch.rs for a complete list of pitches
- Grid columsn are notes, as 32nd notes

## Tracker Grid
- Implemented in the termianl with ncurses. Refer to `reference_only_kitty_grid` for a prior implementation of how to render a grid in the terminal (and note issues about chunking)
- Grid should be shown in cyan color
- Currently selected grid square should be yellow
- Set notes should be magenta
- Grid defaults to 64 columns, 24 px squares
- Grid defaults to two octaves of pitches, centered at middle C.
- Prevent scrolling past the beginning of the song, or outside of the grid generally. We'll improve this later

## Interface
- User navigates around the grid with arrow keys
- Grid
- When navigating, the currently selected grid square highlights yellow
- Pressing 'enter' sets or removes a note at that square. Notes will all just be be 32nd notes for now
- Song is saved automatically whenever a note is modified

## Build Notes
- Make sure the project builds, but don't try to run it. I'll validate, since I'll be using Kitty.
- Don't worry about playback yet