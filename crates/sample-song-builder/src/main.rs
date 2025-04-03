use anyhow::Result;
use dawww_core::{DawFile, instrument::Instrument, pitch::{Pitch, Tone}, Note};
use std::path::PathBuf;

fn main() -> Result<()> {
    // Create a new song
    let mut song = DawFile::new("Mary Had a Little Lamb".to_string());

    // Set up basic parameters
    song.set_bpm(120);
    song.set_mixdown_settings(44100, 16);

    // Create the synth instrument
    let mut params = serde_json::Map::new();
    params.insert("oscillator_wave".to_string(), serde_json::Value::String("sine".to_string()));
    params.insert("filter_type".to_string(), serde_json::Value::String("lowpass".to_string()));
    params.insert("filter_cutoff".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(880.0).unwrap()));
    params.insert("filter_resonance".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.3).unwrap()));
    params.insert("envelope_attack".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.01).unwrap()));
    params.insert("envelope_decay".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.2).unwrap()));
    params.insert("envelope_sustain".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.7).unwrap()));
    params.insert("envelope_release".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.1).unwrap()));

    let synth = Instrument::new_synth("subtractive", params);
    song.add_instrument("synth1".to_string(), synth)?;

    // Define the melody notes
    let melody = vec![
        ("1.0", Tone::E, 4),   // Bar 1
        ("1.8", Tone::D, 4),
        ("1.16", Tone::C, 4),
        ("1.24", Tone::D, 4),
        ("2.0", Tone::E, 4),   // Bar 2
        ("2.8", Tone::E, 4),
        ("2.16", Tone::E, 4),
        ("3.0", Tone::D, 4),   // Bar 3
        ("3.8", Tone::D, 4),
        ("3.16", Tone::D, 4),
        ("4.0", Tone::E, 4),   // Bar 4
        ("4.8", Tone::G, 4),
        ("4.16", Tone::G, 4),
    ];

    // Add all notes to the song
    for (time, tone, octave) in melody {
        let note = Note::new(Pitch::new(tone, octave), 8);
        song.add_note(time, "synth1", note)?;
    }

    // Save the song
    let output_path = PathBuf::from("sample_song/song.daw.json");
    song.save(&output_path)?;

    println!("Song has been created and saved to {}", output_path.display());
    Ok(())
}
