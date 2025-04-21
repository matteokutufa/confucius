// src/include.rs
//! Gestione delle direttive di inclusione nei file di configurazione

use std::fs;
use std::path::Path;
use glob::glob;

use crate::{Config, ConfigError, ConfigFormat};
use crate::utils;
use crate::formats;

/// Processa un'inclusione con pattern glob
pub fn process_glob_include(config: &mut Config, glob_pattern: &str, base_path: &Path) -> Result<(), ConfigError> {
    // Risolviamo il pattern rispetto al percorso base
    let resolved_pattern = utils::resolve_path(base_path, glob_pattern);
    let pattern_str = resolved_pattern.to_string_lossy();

    // Usiamo la libreria glob per trovare tutti i file corrispondenti
    let entries = glob(&pattern_str)
        .map_err(|e| ConfigError::IncludeError(format!("Errore nel pattern glob: {}", e)))?;

    let mut found_any = false;

    // Per ogni file trovato
    for entry in entries {
        match entry {
            Ok(path) => {
                found_any = true;

                // Leggiamo il contenuto del file
                let content = fs::read_to_string(&path)
                    .map_err(|e| ConfigError::IncludeError(format!("Errore di lettura del file incluso {}: {}",
                                                                   path.display(), e)))?;

                // Determiniamo il formato e lo includiamo
                include_content(config, &content, &path)?;
            },
            Err(e) => {
                return Err(ConfigError::IncludeError(format!("Errore nell'espansione del glob: {}", e)));
            }
        }
    }

    if !found_any {
        return Err(ConfigError::IncludeError(format!("Nessun file trovato per il pattern: {}", glob_pattern)));
    }

    Ok(())
}

/// Include il contenuto di un file nella configurazione in base al formato
fn include_content(config: &mut Config, content: &str, path: &Path) -> Result<(), ConfigError> {
    // Determiniamo il formato dal contenuto
    let format = detect_format_from_content(content);

    // Parserizziamo il contenuto in base al formato
    match format {
        ConfigFormat::Ini => formats::ini::parse_ini(config, content, path)?,
        ConfigFormat::Toml => return Err(ConfigError::UnsupportedFormat("TOML".to_string())),
        ConfigFormat::Yaml => return Err(ConfigError::UnsupportedFormat("YAML".to_string())),
        ConfigFormat::Json => return Err(ConfigError::UnsupportedFormat("JSON".to_string())),
        ConfigFormat::Unknown => {
            // Se il formato è sconosciuto, assumiamo INI
            formats::ini::parse_ini(config, content, path)?
        }
    }

    Ok(())
}

/// Rileva il formato dal contenuto del file
fn detect_format_from_content(content: &str) -> ConfigFormat {
    // Leggiamo la prima riga per il formato
    let first_line = content.lines().next().unwrap_or("");

    // Se la prima riga è nel formato #!config/FORMAT
    if first_line.starts_with("#!config/") {
        let format_str = first_line.trim_start_matches("#!config/").trim();
        ConfigFormat::from(format_str)
    } else {
        // Se non specificato, assumiamo INI
        ConfigFormat::Ini
    }
}