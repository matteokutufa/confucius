// src/parser.rs
//! Parser generico per i file di configurazione

// use std::collections::HashMap;
use std::path::Path;

//use crate::{Config, ConfigError, ConfigFormat, ConfigValue};
use crate::{Config, ConfigError, ConfigFormat};
use crate::formats;

/// Parse un file di configurazione in base al suo formato
pub fn parse_file(config: &mut Config, path: &Path) -> Result<(), ConfigError> {
    let content = std::fs::read_to_string(path).map_err(ConfigError::Io)?;

    // Determiniamo il formato dal contenuto
    let format = detect_format(&content);

    match format {
        ConfigFormat::Ini => formats::ini::parse_ini(config, &content, path),
        ConfigFormat::Toml => Err(ConfigError::UnsupportedFormat("TOML".to_string())),
        ConfigFormat::Yaml => Err(ConfigError::UnsupportedFormat("YAML".to_string())),
        ConfigFormat::Json => Err(ConfigError::UnsupportedFormat("JSON".to_string())),
        ConfigFormat::Unknown => Err(ConfigError::UnsupportedFormat("Sconosciuto".to_string())),
    }
}

/// Rileva il formato dal contenuto
fn detect_format(content: &str) -> ConfigFormat {
    // Leggiamo la prima riga
    if let Some(first_line) = content.lines().next() {
        if first_line.starts_with("#!config/") {
            let format_str = first_line.trim_start_matches("#!config/").trim();
            return ConfigFormat::from(format_str);
        }
    }

    // Per default, assumiamo INI
    ConfigFormat::Ini
}