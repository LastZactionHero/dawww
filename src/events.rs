use crossterm::event::{poll, read, Event, KeyCode};
use std::io;
use std::sync::mpsc;
use std::time::Duration;

pub enum InputEvent {
    ViewerBarNext,
    ViewerBarPrevious,
    ViewerResolutionIncrease,
    ViewerResolutionDecrease,
    ViewerOctaveIncrease,
    ViewerOctaveDecrease,
    PlayerTogglePlayback,
    Quit,
    PlayerBeatChange(u64),
    CursorUp,
    CursorDown,
    CursorLeft,
    CursorRight,
    InsertNote,
    Cancel,
    Yank,
    Cut,
    Paste,
    Delete,
    ToggleLoopMode,
    SetLoopTimes,
    SaveSong,
    SelectIn,
}

pub fn capture_input(tx: &mpsc::Sender<InputEvent>) -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut alt_pressed = false;

    loop {
        if poll(Duration::from_millis(500))? {
            if let Event::Key(event) = read()? {
                // Unmapped:
                // 3, 4, q, w, x
                match event.code {
                    // Core navigation and alt key
                    KeyCode::Char('1') => tx.send(InputEvent::Cancel).unwrap(),
                    KeyCode::Char('2') => alt_pressed = !alt_pressed,

                    // Arrow keys - Cursor movement or Viewport navigation
                    KeyCode::Left => {
                        tx.send(if alt_pressed {
                            InputEvent::ViewerBarPrevious
                        } else {
                            InputEvent::CursorLeft
                        })
                        .unwrap();
                    }
                    KeyCode::Right => {
                        tx.send(if alt_pressed {
                            InputEvent::ViewerBarNext
                        } else {
                            InputEvent::CursorRight
                        })
                        .unwrap();
                    }
                    KeyCode::Up => {
                        tx.send(if alt_pressed {
                            InputEvent::ViewerResolutionIncrease
                        } else {
                            InputEvent::CursorUp
                        })
                        .unwrap();
                    }
                    KeyCode::Down => {
                        tx.send(if alt_pressed {
                            InputEvent::ViewerResolutionDecrease
                        } else {
                            InputEvent::CursorDown
                        })
                        .unwrap();
                    }

                    // Most common operations - top row right side
                    KeyCode::Char('r') => tx.send(InputEvent::InsertNote).unwrap(),
                    // TODO: Delete is not working for a single note.
                    KeyCode::Char('f') => tx.send(InputEvent::Delete).unwrap(),

                    // Selection controls - grouped together
                    KeyCode::Char('e') => tx.send(InputEvent::SelectIn).unwrap(),

                    // Clipboard operations - grouped on left side
                    KeyCode::Char('a') => tx.send(InputEvent::Yank).unwrap(),
                    KeyCode::Char('s') => tx.send(InputEvent::Cut).unwrap(),
                    KeyCode::Char('d') => tx.send(InputEvent::Paste).unwrap(),

                    // Loop controls - grouped together
                    KeyCode::Char('c') => tx.send(InputEvent::ToggleLoopMode).unwrap(),
                    KeyCode::Char('v') => tx.send(InputEvent::SetLoopTimes).unwrap(),

                    // Save and quit - bottom row
                    KeyCode::Char('z') => tx.send(InputEvent::SaveSong).unwrap(),

                    KeyCode::Char('p') => {
                        tx.send(InputEvent::Quit).unwrap();
                        break;
                    }

                    // Playback control
                    KeyCode::Char('\\') => tx.send(InputEvent::PlayerTogglePlayback).unwrap(),

                    _ => (),
                }
            }
        }
    }
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
