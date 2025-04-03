use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::{Result, bail};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Instrument {
    #[serde(rename = "type")]
    pub instrument_type: String,
    pub parameters: serde_json::Value,
}

impl Instrument {
    /// Create a new sampler instrument
    pub fn new_sampler(sample_path: PathBuf) -> Self {
        let mut parameters = serde_json::Map::new();
        parameters.insert("sample_file".to_string(), serde_json::Value::String(sample_path.to_string_lossy().into_owned()));

        Self {
            instrument_type: "sampler".to_string(),
            parameters: serde_json::Value::Object(parameters),
        }
    }

    /// Create a new synth instrument
    pub fn new_synth(subtype: &str, parameters: serde_json::Map<String, serde_json::Value>) -> Self {
        let mut params = parameters;
        params.insert("subtype".to_string(), serde_json::Value::String(subtype.to_string()));

        Self {
            instrument_type: "synth".to_string(),
            parameters: serde_json::Value::Object(params),
        }
    }

    /// Validate the instrument configuration
    pub fn validate(&self) -> Result<()> {
        match self.instrument_type.as_str() {
            "sampler" => {
                let params = self.parameters.as_object()
                    .ok_or_else(|| anyhow::anyhow!("Sampler parameters must be an object"))?;
                
                if !params.contains_key("sample_file") {
                    bail!("Sampler must have a sample_file parameter");
                }
            }
            "synth" => {
                let params = self.parameters.as_object()
                    .ok_or_else(|| anyhow::anyhow!("Synth parameters must be an object"))?;
                
                if !params.contains_key("subtype") {
                    bail!("Synth must have a subtype parameter");
                }

                match params["subtype"].as_str() {
                    Some("subtractive") => {
                        // Validate required parameters for subtractive synth
                        let required = vec!["oscillator_wave", "filter_type", "filter_cutoff", 
                                         "filter_resonance", "envelope_attack", "envelope_decay", 
                                         "envelope_sustain", "envelope_release"];
                        for param in required {
                            if !params.contains_key(param) {
                                bail!("Subtractive synth missing required parameter: {}", param);
                            }
                        }
                    }
                    Some(other) => bail!("Unsupported synth subtype: {}", other),
                    None => bail!("Invalid synth subtype"),
                }
            }
            _ => bail!("Invalid instrument type: {}", self.instrument_type),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sampler() {
        let sample_path = PathBuf::from("audio/kick.wav");
        let sampler = Instrument::new_sampler(sample_path.clone());
        
        assert_eq!(sampler.instrument_type, "sampler");
        
        let params = sampler.parameters.as_object().unwrap();
        assert_eq!(params["sample_file"], sample_path.to_string_lossy().to_string());
    }

    #[test]
    fn test_new_synth() {
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
        
        assert_eq!(synth.instrument_type, "synth");
        let params = synth.parameters.as_object().unwrap();
        assert_eq!(params["subtype"], "subtractive");
        assert_eq!(params["oscillator_wave"], "sine");
    }

    #[test]
    fn test_validate_sampler() {
        let valid_sampler = Instrument::new_sampler(
            PathBuf::from("test.wav")
        );
        assert!(valid_sampler.validate().is_ok());

        let invalid_sampler = Instrument {
            instrument_type: "sampler".to_string(),
            parameters: serde_json::json!({}),
        };
        assert!(invalid_sampler.validate().is_err());
    }

    #[test]
    fn test_validate_synth() {
        let mut params = serde_json::Map::new();
        params.insert("oscillator_wave".to_string(), serde_json::Value::String("sine".to_string()));
        params.insert("filter_type".to_string(), serde_json::Value::String("lowpass".to_string()));
        params.insert("filter_cutoff".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(880.0).unwrap()));
        params.insert("filter_resonance".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.3).unwrap()));
        params.insert("envelope_attack".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.01).unwrap()));
        params.insert("envelope_decay".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.2).unwrap()));
        params.insert("envelope_sustain".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.7).unwrap()));
        params.insert("envelope_release".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.1).unwrap()));

        let valid_synth = Instrument::new_synth("subtractive", params);
        assert!(valid_synth.validate().is_ok());

        let invalid_synth = Instrument {
            instrument_type: "synth".to_string(),
            parameters: serde_json::json!({}),
        };
        assert!(invalid_synth.validate().is_err());
    }
} 