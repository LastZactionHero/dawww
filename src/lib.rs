use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct DawFile {
    pub metadata: Metadata,
    pub bpm: u32,
    pub mixdown: MixdownSettings,
    pub instruments: HashMap<String, Instrument>,
    pub events: Vec<Event>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,
    pub creation_date: String,
    pub modification_date: String,
    pub revision: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MixdownSettings {
    pub sample_rate: u32,
    pub bit_depth: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Instrument {
    #[serde(rename = "type")]
    pub instrument_type: String,
    pub subtype: Option<String>,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub time: String,
    pub instrument: String,
    pub pitches: Vec<Pitch>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pitch {
    pub pitch: String,
    pub duration: u32,
}

pub mod audio;
pub mod synth;
pub mod wav; 