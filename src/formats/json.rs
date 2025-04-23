// src/formats/json.rs
//! Implementazione del parser e writer per il formato JSON

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;
use serde_json::{Value as JsonValue, Map as JsonMap};

use crate::{Config, ConfigError, ConfigValue};
use crate::include;
use crate::utils;

/// Parser per file JSON
pub fn parse_json(config: &mut Config, content: &str, path: &Path) -> Result<(), ConfigError> {
    // Saltare la prima riga se contiene il formato (#!config/...)
    let content_to_parse = if content.lines().next().unwrap_or("").starts_with("#!config/") {
        // Prendiamo tutte le righe tranne la prima
        content.lines().skip(1).collect::<Vec<_>>().join("\n")
    } else {
        content.to_string()
    };

    // Parserizziamo il contenuto JSON
    let parsed_json: JsonValue = serde_json::from_str(&content_to_parse)
        .map_err(|e| ConfigError::ParseError(format!("Errore nel parsing JSON: {}", e)))?;

    // Ci aspettiamo che il root sia un oggetto
    if let JsonValue::Object(obj) = parsed_json {
        // Processiamo le inclusioni se presenti
        if let Some(include_value) = obj.get("include") {
            process_includes(config, include_value, path)?;
        }

        // Convertiamo ogni sezione e valore nel formato di Config
        for (section_name, section_value) in &obj {
            // Saltiamo la sezione "include" che abbiamo già processato
            if section_name == "include" {
                continue;
            }

            match section_value {
                JsonValue::Object(section_obj) => {
                    // È una sezione standard, processiamo ogni coppia chiave-valore
                    for (key, value) in section_obj {
                        let config_value = json_value_to_config_value(value);
                        config.set(section_name, key, config_value);
                    }
                },
                _ => {
                    // È un valore diretto nella sezione default
                    let config_value = json_value_to_config_value(section_value);
                    config.set("default", section_name, config_value);
                }
            }
        }
    } else {
        return Err(ConfigError::ParseError("Il file JSON deve avere una struttura ad oggetto nella root".to_string()));
    }

    Ok(())
}

/// Converte un valore JSON in un ConfigValue
fn json_value_to_config_value(value: &JsonValue) -> ConfigValue {
    match value {
        JsonValue::String(s) => ConfigValue::String(s.clone()),
        JsonValue::Number(n) => {
            // I numeri JSON possono essere interi o float
            if n.is_i64() {
                ConfigValue::Integer(n.as_i64().unwrap())
            } else {
                ConfigValue::Float(n.as_f64().unwrap())
            }
        },
        JsonValue::Bool(b) => ConfigValue::Boolean(*b),
        JsonValue::Array(arr) => {
            let values: Vec<ConfigValue> = arr.iter()
                .map(json_value_to_config_value)
                .collect();
            ConfigValue::Array(values)
        },
        JsonValue::Object(obj) => {
            let mut config_map = HashMap::new();
            for (k, v) in obj {
                config_map.insert(k.clone(), json_value_to_config_value(v));
            }
            ConfigValue::Table(config_map)
        },
        JsonValue::Null => ConfigValue::String("".to_string()),
    }
}

/// Processa le inclusioni nel file JSON
fn process_includes(config: &mut Config, include_value: &JsonValue, base_path: &Path) -> Result<(), ConfigError> {
    match include_value {
        JsonValue::String(include_path) => {
            // Caso singolo path
            process_single_include(config, include_path, base_path)?;
        },
        JsonValue::Array(includes) => {
            // Caso array di paths
            for include_item in includes {
                if let JsonValue::String(include_path) = include_item {
                    process_single_include(config, include_path, base_path)?;
                } else {
                    return Err(ConfigError::IncludeError(
                        "Le inclusioni devono essere stringhe".to_string()
                    ));
                }
            }
        },
        _ => {
            return Err(ConfigError::IncludeError(
                "Il formato dell'inclusione non è valido. Deve essere una stringa o un array di stringhe".to_string()
            ));
        }
    }

    Ok(())
}

/// Processa una singola inclusione
fn process_single_include(config: &mut Config, include_path: &str, base_path: &Path) -> Result<(), ConfigError> {
    // Se l'include è un glob pattern, includiamo tutti i file corrispondenti
    if include_path.contains('*') {
        include::process_glob_include(config, include_path, base_path)?;
    } else {
        // Altrimenti includiamo un singolo file
        let resolved_path = utils::resolve_path(base_path, include_path);
        if resolved_path.exists() {
            let content = fs::read_to_string(&resolved_path)
                .map_err(|e| ConfigError::IncludeError(format!("Errore di lettura del file incluso {}: {}",
                                                               resolved_path.display(), e)))?;

            // Determiniamo il formato e processiamo il file incluso
            let first_line = content.lines().next().unwrap_or("");
            if first_line.starts_with("#!config/json") {
                parse_json(config, &content, &resolved_path)?;
            } else if first_line.starts_with("#!config/yaml") {
                crate::formats::yaml::parse_yaml(config, &content, &resolved_path)?;
            } else if first_line.starts_with("#!config/toml") {
                crate::formats::toml::parse_toml(config, &content, &resolved_path)?;
            } else if first_line.starts_with("#!config/ini") {
                crate::formats::ini::parse_ini(config, &content, &resolved_path)?;
            } else {
                // Se il formato non è specificato, proviamo a capirlo dall'estensione
                let extension = resolved_path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");

                match extension {
                    "json" => parse_json(config, &content, &resolved_path)?,
                    "yaml" | "yml" => crate::formats::yaml::parse_yaml(config, &content, &resolved_path)?,
                    "toml" => crate::formats::toml::parse_toml(config, &content, &resolved_path)?,
                    "ini" => crate::formats::ini::parse_ini(config, &content, &resolved_path)?,
                    _ => {
                        // Se non riusciamo a determinare il formato, assumiamo JSON
                        parse_json(config, &content, &resolved_path)?;
                    }
                }
            }
        } else {
            return Err(ConfigError::IncludeError(format!("File incluso non trovato: {}",
                                                         resolved_path.display())));
        }
    }

    Ok(())
}

/// Converte un ConfigValue in JsonValue
fn config_value_to_json_value(value: &ConfigValue) -> JsonValue {
    match value {
        ConfigValue::String(s) => JsonValue::String(s.clone()),
        ConfigValue::Integer(i) => JsonValue::Number((*i).into()),
        ConfigValue::Float(f) => {
            // Gestione sicura della conversione float -> JSON Number
            match serde_json::Number::from_f64(*f) {
                Some(num) => JsonValue::Number(num),
                None => JsonValue::String(f.to_string()), // Fallback a stringa
            }
        },
        ConfigValue::Boolean(b) => JsonValue::Bool(*b),
        ConfigValue::Array(arr) => {
            let values: Vec<JsonValue> = arr.iter()
                .map(config_value_to_json_value)
                .collect();
            JsonValue::Array(values)
        },
        ConfigValue::Table(table) => {
            let mut json_obj = JsonMap::new();
            for (k, v) in table {
                json_obj.insert(k.clone(), config_value_to_json_value(v));
            }
            JsonValue::Object(json_obj)
        },
    }
}

/// Scrive la configurazione in formato JSON
pub fn write_json(config: &Config, path: &Path) -> Result<(), ConfigError> {
    let mut file = File::create(path).map_err(ConfigError::Io)?;

    // Scriviamo l'intestazione del formato
    writeln!(file, "#!config/json").map_err(ConfigError::Io)?;

    // Creiamo una struttura JSON
    let mut root_obj = JsonMap::new();

    // Per ogni sezione
    for (section, values) in &config.values {
        if section == "default" {
            // Valori nella sezione default vanno nella root
            for (key, value) in values {
                root_obj.insert(key.clone(), config_value_to_json_value(value));
            }
        } else {
            // Altre sezioni diventano oggetti annidati
            let mut section_obj = JsonMap::new();
            for (key, value) in values {
                section_obj.insert(key.clone(), config_value_to_json_value(value));
            }

            if !section_obj.is_empty() {
                root_obj.insert(section.clone(), JsonValue::Object(section_obj));
            }
        }
    }

    // Convertiamo in stringa formattata e scriviamo sul file
    let json_string = serde_json::to_string_pretty(&JsonValue::Object(root_obj))
        .map_err(|e| ConfigError::Generic(format!("Errore nella serializzazione JSON: {}", e)))?;

    write!(file, "{}", json_string).map_err(ConfigError::Io)?;

    Ok(())
}