// src/formats/toml.rs
//! Implementazione del parser e writer per il formato TOML

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;
use toml::{Value as TomlValue, Table as TomlTable};

use crate::{Config, ConfigError, ConfigValue};
use crate::include;
use crate::utils;

/// Parser per file TOML
pub fn parse_toml(config: &mut Config, content: &str, path: &Path) -> Result<(), ConfigError> {
    // Saltare la prima riga se contiene il formato (#!config/...)
    let content_to_parse = if content.lines().next().unwrap_or("").starts_with("#!config/") {
        // Prendiamo tutte le righe tranne la prima
        content.lines().skip(1).collect::<Vec<_>>().join("\n")
    } else {
        content.to_string()
    };

    // Parserizziamo il contenuto TOML
    let parsed_toml: TomlTable = content_to_parse.parse()
        .map_err(|e| ConfigError::ParseError(format!("Errore nel parsing TOML: {}", e)))?;

    // Processiamo le inclusioni se presenti
    if let Some(include_value) = parsed_toml.get("include") {
        process_includes(config, include_value, path)?;
    }

    // Convertiamo ogni sezione e valore nel formato di Config
    for (section_name, section_value) in &parsed_toml {
        // Saltiamo la sezione "include" che abbiamo già processato
        if section_name == "include" {
            continue;
        }

        match section_value {
            TomlValue::Table(table) => {
                // È una sezione standard, processiamo ogni coppia chiave-valore
                for (key, value) in table {
                    let config_value = toml_value_to_config_value(value);
                    config.set(section_name, key, config_value);
                }
            },
            _ => {
                // È un valore nella sezione root (default)
                let config_value = toml_value_to_config_value(section_value);
                config.set("default", section_name, config_value);
            }
        }
    }

    Ok(())
}

/// Converte un valore TOML in un ConfigValue
fn toml_value_to_config_value(value: &TomlValue) -> ConfigValue {
    match value {
        TomlValue::String(s) => ConfigValue::String(s.clone()),
        TomlValue::Integer(i) => ConfigValue::Integer(*i),
        TomlValue::Float(f) => ConfigValue::Float(*f),
        TomlValue::Boolean(b) => ConfigValue::Boolean(*b),
        TomlValue::Array(arr) => {
            let values: Vec<ConfigValue> = arr.iter()
                .map(toml_value_to_config_value)
                .collect();
            ConfigValue::Array(values)
        },
        TomlValue::Table(table) => {
            let mut map = HashMap::new();
            for (k, v) in table {
                map.insert(k.clone(), toml_value_to_config_value(v));
            }
            ConfigValue::Table(map)
        },
        // Trattiamo Datetime come stringa
        TomlValue::Datetime(dt) => ConfigValue::String(dt.to_string()),
    }
}

/// Processa le inclusioni nel file TOML
fn process_includes(config: &mut Config, include_value: &TomlValue, base_path: &Path) -> Result<(), ConfigError> {
    match include_value {
        TomlValue::String(include_path) => {
            // Caso singolo path
            process_single_include(config, include_path, base_path)?;
        },
        TomlValue::Array(includes) => {
            // Caso array di paths
            for include_item in includes {
                if let TomlValue::String(include_path) = include_item {
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
            if first_line.starts_with("#!config/toml") {
                parse_toml(config, &content, &resolved_path)?;
            } else if first_line.starts_with("#!config/ini") {
                crate::formats::ini::parse_ini(config, &content, &resolved_path)?;
            } else if first_line.starts_with("#!config/yaml") {
                // Quando aggiungeremo il supporto YAML
                return Err(ConfigError::UnsupportedFormat("YAML".to_string()));
            } else {
                // Se il formato non è specificato, proviamo a capirlo dall'estensione
                let extension = resolved_path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");

                match extension {
                    "toml" => parse_toml(config, &content, &resolved_path)?,
                    "ini" => crate::formats::ini::parse_ini(config, &content, &resolved_path)?,
                    "yaml" | "yml" => return Err(ConfigError::UnsupportedFormat("YAML".to_string())),
                    _ => {
                        // Se non riusciamo a determinare il formato, assumiamo TOML
                        parse_toml(config, &content, &resolved_path)?;
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

/// Converte un ConfigValue in TomlValue
fn config_value_to_toml_value(value: &ConfigValue) -> TomlValue {
    match value {
        ConfigValue::String(s) => TomlValue::String(s.clone()),
        ConfigValue::Integer(i) => TomlValue::Integer(*i),
        ConfigValue::Float(f) => TomlValue::Float(*f),
        ConfigValue::Boolean(b) => TomlValue::Boolean(*b),
        ConfigValue::Array(arr) => {
            let values: Vec<TomlValue> = arr.iter()
                .map(config_value_to_toml_value)
                .collect();
            TomlValue::Array(values)
        },
        ConfigValue::Table(table) => {
            let mut toml_table = TomlTable::new();
            for (k, v) in table {
                toml_table.insert(k.clone(), config_value_to_toml_value(v));
            }
            TomlValue::Table(toml_table)
        },
    }
}

/// Scrive la configurazione in formato TOML
pub fn write_toml(config: &Config, path: &Path) -> Result<(), ConfigError> {
    let mut file = File::create(path).map_err(ConfigError::Io)?;

    // Scriviamo l'intestazione del formato
    writeln!(file, "#!config/toml").map_err(ConfigError::Io)?;

    // Creiamo una struttura TOML
    let mut root_table = TomlTable::new();

    // Per ogni sezione
    for (section, values) in &config.values {
        if section == "default" {
            // Valori nella sezione default vanno nella root
            for (key, value) in values {
                root_table.insert(key.clone(), config_value_to_toml_value(value));
            }
        } else {
            // Altre sezioni diventano tabelle
            let mut section_table = TomlTable::new();
            for (key, value) in values {
                section_table.insert(key.clone(), config_value_to_toml_value(value));
            }

            if !section_table.is_empty() {
                root_table.insert(section.clone(), TomlValue::Table(section_table));
            }
        }
    }

    // Convertiamo in stringa e scriviamo sul file
    let toml_string = toml::to_string_pretty(&root_table)
        .map_err(|e| ConfigError::Generic(format!("Errore nella serializzazione TOML: {}", e)))?;

    writeln!(file, "{}", toml_string).map_err(ConfigError::Io)?;

    Ok(())
}