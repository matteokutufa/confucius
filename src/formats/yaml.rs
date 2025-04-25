//! Implementation of the parser and writer for the YAML format.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;
use serde_yaml::{Value as YamlValue, Mapping as YamlMapping};

use crate::{Config, ConfigError, ConfigValue};
use crate::include;
use crate::utils;

/// Parses a YAML file and updates the provided configuration.
///
/// This function reads the content of a YAML file, processes its sections, key-value pairs,
/// and include directives, and updates the given `Config` instance accordingly.
///
/// # Arguments
///
/// * `config` - A mutable reference to the `Config` instance to update.
/// * `content` - The content of the YAML file as a string.
/// * `path` - The path to the YAML file being parsed.
///
/// # Returns
///
/// * `Ok(())` - If the parsing is successful.
/// * `Err(ConfigError)` - If an error occurs during parsing.
pub fn parse_yaml(config: &mut Config, content: &str, path: &Path) -> Result<(), ConfigError> {
    let content_to_parse = if content.lines().next().unwrap_or("").starts_with("#!config/") {
        content.lines().skip(1).collect::<Vec<_>>().join("\n")
    } else {
        content.to_string()
    };

    let parsed_yaml: YamlValue = serde_yaml::from_str(&content_to_parse)
        .map_err(|e| ConfigError::ParseError(format!("Errore nel parsing YAML: {}", e)))?;

    if let YamlValue::Mapping(mapping) = parsed_yaml {
        if let Some(include_value) = mapping.get(&YamlValue::String("include".to_string())) {
            process_includes(config, include_value, path)?;
        }

        for (key_value, value) in &mapping {
            if let YamlValue::String(section_name) = key_value {
                if section_name == "include" {
                    continue;
                }

                match value {
                    YamlValue::Mapping(section_mapping) => {
                        for (sub_key_value, sub_value) in section_mapping {
                            if let YamlValue::String(key) = sub_key_value {
                                let config_value = yaml_value_to_config_value(sub_value);
                                config.set(section_name, key, config_value);
                            }
                        }
                    },
                    _ => {
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

/// Converts a YAML value into a `ConfigValue`.
///
/// This function maps YAML types (e.g., string, number, boolean, array, mapping) to
/// their corresponding `ConfigValue` representation.
///
/// # Arguments
///
/// * `value` - A reference to the YAML value to convert.
///
/// # Returns
///
/// A `ConfigValue` representing the converted value.
fn yaml_value_to_config_value(value: &YamlValue) -> ConfigValue {
    match value {
        YamlValue::String(s) => ConfigValue::String(s.clone()),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                ConfigValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                ConfigValue::Float(f)
            } else {
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
                    config_map.insert(k.as_str().unwrap().to_string(), yaml_value_to_config_value(v));
                }
            }
            ConfigValue::Table(config_map)
        },
        YamlValue::Null => ConfigValue::String("".to_string()),
        _ => ConfigValue::String("".to_string()),
    }
}

/// Processes include directives in a YAML file.
///
/// This function handles both single file includes and arrays of include paths,
/// resolving the paths and parsing the included files.
///
/// # Arguments
///
/// * `config` - A mutable reference to the `Config` instance to update.
/// * `include_value` - The YAML value representing the include directive.
/// * `base_path` - The base path of the current YAML file.
///
/// # Returns
///
/// * `Ok(())` - If the include is processed successfully.
/// * `Err(ConfigError)` - If an error occurs during processing.
fn process_includes(config: &mut Config, include_value: &YamlValue, base_path: &Path) -> Result<(), ConfigError> {
    match include_value {
        YamlValue::String(include_path) => {
            process_single_include(config, include_path, base_path)?;
        },
        YamlValue::Sequence(includes) => {
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

/// Processes a single include directive.
///
/// This function resolves the path of the included file, determines its format,
/// and parses it into the configuration.
///
/// # Arguments
///
/// * `config` - A mutable reference to the `Config` instance to update.
/// * `include_path` - The path of the file to include.
/// * `base_path` - The base path of the current YAML file.
///
/// # Returns
///
/// * `Ok(())` - If the include is processed successfully.
/// * `Err(ConfigError)` - If an error occurs during processing.
fn process_single_include(config: &mut Config, include_path: &str, base_path: &Path) -> Result<(), ConfigError> {
    if include_path.contains('*') {
        include::process_glob_include(config, include_path, base_path)?;
    } else {
        let resolved_path = utils::resolve_path(base_path, include_path);
        if resolved_path.exists() {
            let content = fs::read_to_string(&resolved_path)
                .map_err(|e| ConfigError::IncludeError(format!("Errore di lettura del file incluso {}: {}",
                                                               resolved_path.display(), e)))?;

            let first_line = content.lines().next().unwrap_or("");
            if first_line.starts_with("#!config/yaml") {
                parse_yaml(config, &content, &resolved_path)?;
            } else if first_line.starts_with("#!config/toml") {
                crate::formats::toml::parse_toml(config, &content, &resolved_path)?;
            } else if first_line.starts_with("#!config/ini") {
                crate::formats::ini::parse_ini(config, &content, &resolved_path)?;
            } else {
                let extension = resolved_path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");

                match extension {
                    "yaml" | "yml" => parse_yaml(config, &content, &resolved_path)?,
                    "toml" => crate::formats::toml::parse_toml(config, &content, &resolved_path)?,
                    "ini" => crate::formats::ini::parse_ini(config, &content, &resolved_path)?,
                    _ => {
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

/// Converts a `ConfigValue` into a YAML value.
///
/// This function maps `ConfigValue` types (e.g., string, integer, float, boolean, array, table)
/// to their corresponding YAML representation.
///
/// # Arguments
///
/// * `value` - A reference to the `ConfigValue` to convert.
///
/// # Returns
///
/// A `YamlValue` representing the converted value.
fn config_value_to_yaml_value(value: &ConfigValue) -> YamlValue {
    match value {
        ConfigValue::String(s) => YamlValue::String(s.clone()),
        ConfigValue::Integer(i) => {
            serde_yaml::to_value(i).unwrap_or(YamlValue::Null)
        },
        ConfigValue::Float(f) => {
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

/// Writes the configuration to a YAML file.
///
/// This function serializes the given `Config` instance into the YAML format
/// and writes it to the specified file path.
///
/// # Arguments
///
/// * `config` - A reference to the `Config` instance to serialize.
/// * `path` - The path to the output YAML file.
///
/// # Returns
///
/// * `Ok(())` - If the writing is successful.
/// * `Err(ConfigError)` - If an error occurs during writing.
pub fn write_yaml(config: &Config, path: &Path) -> Result<(), ConfigError> {
    let mut file = File::create(path).map_err(ConfigError::Io)?;

    writeln!(file, "#!config/yaml").map_err(ConfigError::Io)?;

    let mut root_mapping = YamlMapping::new();

    for (section, values) in &config.values {
        if section == "default" {
            for (key, value) in values {
                root_mapping.insert(
                    YamlValue::String(key.clone()),
                    config_value_to_yaml_value(value)
                );
            }
        } else {
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

    let yaml_string = serde_yaml::to_string(&YamlValue::Mapping(root_mapping))
        .map_err(|e| ConfigError::Generic(format!("Errore nella serializzazione YAML: {}", e)))?;

    write!(file, "{}", yaml_string).map_err(ConfigError::Io)?;

    Ok(())
}