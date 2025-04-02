use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use dawww::{DawFile, audio::AudioEngine};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input directory containing the DAW file
    #[arg(required = true)]
    input_dir: PathBuf,

    /// Output WAV file path
    #[arg(required = true)]
    output_file: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Find the .daw.json file in the input directory
    let daw_file = find_daw_file(&args.input_dir)?;
    
    // Read and parse the DAW file
    let daw_content = std::fs::read_to_string(&daw_file)?;
    let daw_data: DawFile = serde_json::from_str(&daw_content)?;

    // Create audio engine and render
    let engine = AudioEngine::new(daw_data);
    engine.render(&args.output_file)?;

    println!("Successfully rendered audio to {}", args.output_file.display());
    Ok(())
}

fn find_daw_file(dir: &PathBuf) -> Result<PathBuf> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_name().to_string_lossy().ends_with(".daw.json") {
            return Ok(entry.path());
        }
    }
    anyhow::bail!("No .daw.json file found in {}", dir.display());
}
