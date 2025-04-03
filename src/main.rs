use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;
use dawww_core::{find_daw_file, read_daw_file};
use dawww_render::AudioEngine;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the song directory
    #[arg(value_name = "SONG_DIR")]
    song_dir: PathBuf,

    /// Path to save the output WAV file
    #[arg(value_name = "OUTPUT_WAV")]
    output_wav: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Find and read the song file
    let song_file = find_daw_file(&args.song_dir)?;
    let daw_file = read_daw_file(&song_file)?;

    // Create audio engine and render
    let engine = AudioEngine::new(daw_file);
    engine.render(&args.output_wav)?;

    println!("Successfully rendered {} to {}", song_file.display(), args.output_wav.display());
    Ok(())
}
