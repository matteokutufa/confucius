// src/formats/yaml.rs
//! Implementazione del parser e writer per il formato YAML

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;
use serde_yaml::{Value as YamlValue, Mapping as YamlMapping};

use crate::{Config, ConfigError, ConfigValue};
use crate::include;
use crate::utils;

/// Parser per file YAML
pub fn parse_yaml(config: &mut Config, content: &str, path: &Path) -> Result<(), ConfigError> {
    // Saltare la prima riga se contiene il formato (#!config/...)
    let content_to_parse = if content.lines().next().unwrap_or("").starts_with("#!config/") {
        // Prendiamo tutte le righe tranne la prima
        content.lines().skip(1).collect::<Vec<_>>().join("\n")
    } else {
        content.to_string()
    };

    // Parserizziamo il contenuto YAML
    let parsed_yaml: YamlValue = serde_yaml::from_str(&content_to_parse)
        .map_err(|e| ConfigError::ParseError(format!("Errore nel parsing YAML: {}", e)))?;

    // Ci aspettiamo che il root sia un mapping (oggetto)
    if let YamlValue::Mapping(mapping) = parsed_yaml {
        // Processiamo le inclusioni se presenti
        if let Some(include_value) = mapping.get(&YamlValue::String("include".to_string())) {
            process_includes(config, include_value, path)?;
        }

        // Convertiamo ogni sezione e valore nel formato di Config
        for (key_value, value) in &mapping {
            // Convertiamo la chiave in stringa
            if let YamlValue::String(section_name) = key_value {
                // Saltiamo la sezione "include" che abbiamo già processato
                if section_name == "include" {
                    continue;
                }

                match value {
                    YamlValue::Mapping(section_mapping) => {
                        // È una sezione standard, processiamo ogni coppia chiave-valore
                        for (sub_key_value, sub_value) in section_mapping {
                            if let YamlValue::String(key) = sub_key_value {
                                let config_value = yaml_value_to_config_value(sub_value);
                                config.set(section_name, key, config_value);
                            }
                        }
                    },
                    _ => {
                        // È un valore diretto nella sezione default
                        let config_value = yaml_value_to_config_value(value);
                        config.set("default", section_name, config_value);
                    }
                }
            }
        }
    } else {
        return Err(ConfigError::ParseError("Il file YAML deve avere una struttura ad oggetto nella root".to_string()));
    }

    Ok(())
}

/// Converte un valore YAML in un ConfigValue
fn yaml_value_to_config_value(value: &YamlValue) -> ConfigValue {
    match value {
        YamlValue::String(s) => ConfigValue::String(s.clone()),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                ConfigValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                ConfigValue::Float(f)
            } else {
                // Fallback per altri tipi numerici
                ConfigValue::String(n.to_string())
            }
        },
        YamlValue::Bool(b) => ConfigValue::Boolean(*b),
        YamlValue::Sequence(seq) => {
            let values: Vec<ConfigValue> = seq.iter()
                .map(yaml_value_to_config_value)
                .collect();
            ConfigValue::Array(values)
        },
        YamlValue::Mapping(map) => {
            let mut config_map = HashMap::new();
            for (k, v) in map {
                if let YamlValue::String(key) = k {
                    config_map.insert(key.clone(), yaml_value_to_config_value(v));
                } else {
                    // Convertiamo chiavi non-stringa in stringa
                    config_map.insert(k.as_str().unwrap().to_string(), yaml_value_to_config_value(v));
                }
            }
            ConfigValue::Table(config_map)
        },
        YamlValue::Null => ConfigValue::String("".to_string()),
        _ => ConfigValue::String("".to_string()),
    }
}

/// Processa le inclusioni nel file YAML
fn process_includes(config: &mut Config, include_value: &YamlValue, base_path: &Path) -> Result<(), ConfigError> {
    match include_value {
        YamlValue::String(include_path) => {
            // Caso singolo path
            process_single_include(config, include_path, base_path)?;
        },
        YamlValue::Sequence(includes) => {
            // Caso array di paths
            for include_item in includes {
                if let YamlValue::String(include_path) = include_item {
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
                "Il formato dell'inclusione non è valido. Deve essere una stringa o una sequenza di stringhe".to_string()
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
            if first_line.starts_with("#!config/yaml") {
                parse_yaml(config, &content, &resolved_path)?;
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
                    "yaml" | "yml" => parse_yaml(config, &content, &resolved_path)?,
                    "toml" => crate::formats::toml::parse_toml(config, &content, &resolved_path)?,
                    "ini" => crate::formats::ini::parse_ini(config, &content, &resolved_path)?,
                    _ => {
                        // Se non riusciamo a determinare il formato, assumiamo YAML
                        parse_yaml(config, &content, &resolved_path)?;
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

/// Converte un ConfigValue in YamlValue
fn config_value_to_yaml_value(value: &ConfigValue) -> YamlValue {
    match value {
        ConfigValue::String(s) => YamlValue::String(s.clone()),
        ConfigValue::Integer(i) => {
            // Convertiamo l'intero in un numero YAML
            serde_yaml::to_value(i).unwrap_or(YamlValue::Null)
        },
        ConfigValue::Float(f) => {
            // Convertiamo il float in un numero YAML
            serde_yaml::to_value(f).unwrap_or(YamlValue::Null)
        },
        ConfigValue::Boolean(b) => YamlValue::Bool(*b),
        ConfigValue::Array(arr) => {
            let values: Vec<YamlValue> = arr.iter()
                .map(config_value_to_yaml_value)
                .collect();
            YamlValue::Sequence(values)
        },
        ConfigValue::Table(table) => {
            let mut yaml_mapping = YamlMapping::new();
            for (k, v) in table {
                yaml_mapping.insert(
                    YamlValue::String(k.clone()),
                    config_value_to_yaml_value(v)
                );
            }
            YamlValue::Mapping(yaml_mapping)
        },
    }
}

/// Scrive la configurazione in formato YAML
pub fn write_yaml(config: &Config, path: &Path) -> Result<(), ConfigError> {
    let mut file = File::create(path).map_err(ConfigError::Io)?;

    // Scriviamo l'intestazione del formato
    writeln!(file, "#!config/yaml").map_err(ConfigError::Io)?;

    // Creiamo una struttura YAML
    let mut root_mapping = YamlMapping::new();

    // Per ogni sezione
    for (section, values) in &config.values {
        if section == "default" {
            // Valori nella sezione default vanno nella root
            for (key, value) in values {
                root_mapping.insert(
                    YamlValue::String(key.clone()),
                    config_value_to_yaml_value(value)
                );
            }
        } else {
            // Altre sezioni diventano oggetti annidati
            let mut section_mapping = YamlMapping::new();
            for (key, value) in values {
                section_mapping.insert(
                    YamlValue::String(key.clone()),
                    config_value_to_yaml_value(value)
                );
            }

            if !section_mapping.is_empty() {
                root_mapping.insert(
                    YamlValue::String(section.clone()),
                    YamlValue::Mapping(section_mapping)
                );
            }
        }
    }

    // Convertiamo in stringa e scriviamo sul file
    let yaml_string = serde_yaml::to_string(&YamlValue::Mapping(root_mapping))
        .map_err(|e| ConfigError::Generic(format!("Errore nella serializzazione YAML: {}", e)))?;

    write!(file, "{}", yaml_string).map_err(ConfigError::Io)?;

    Ok(())
}