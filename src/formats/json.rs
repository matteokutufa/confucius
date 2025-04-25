//! Implementation of the parser and writer for the JSON format.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;
use serde_json::{Value as JsonValue, Map as JsonMap};

use crate::{Config, ConfigError, ConfigValue};
use crate::include;
use crate::utils;

/// Parses a JSON file and updates the provided configuration.
///
/// This function reads the content of a JSON file, processes its sections, key-value pairs,
/// and include directives, and updates the given `Config` instance accordingly.
///
/// # Arguments
///
/// * `config` - A mutable reference to the `Config` instance to update.
/// * `content` - The content of the JSON file as a string.
/// * `path` - The path to the JSON file being parsed.
///
/// # Returns
///
/// * `Ok(())` - If the parsing is successful.
/// * `Err(ConfigError)` - If an error occurs during parsing.
pub fn parse_json(config: &mut Config, content: &str, path: &Path) -> Result<(), ConfigError> {
    let content_to_parse = if content.lines().next().unwrap_or("").starts_with("#!config/") {
        content.lines().skip(1).collect::<Vec<_>>().join("\n")
    } else {
        content.to_string()
    };

    let parsed_json: JsonValue = serde_json::from_str(&content_to_parse)
        .map_err(|e| ConfigError::ParseError(format!("JSON parsing error: {}", e)))?;

    if let JsonValue::Object(obj) = parsed_json {
        if let Some(include_value) = obj.get("include") {
            process_includes(config, include_value, path)?;
        }

        for (section_name, section_value) in &obj {
            if section_name == "include" {
                continue;
            }

            match section_value {
                JsonValue::Object(section_obj) => {
                    for (key, value) in section_obj {
                        let config_value = json_value_to_config_value(value);
                        config.set(section_name, key, config_value);
                    }
                },
                _ => {
                    let config_value = json_value_to_config_value(section_value);
                    config.set("default", section_name, config_value);
                }
            }
        }
    } else {
        return Err(ConfigError::ParseError("The JSON file must have an object structure at the root".to_string()));
    }

    Ok(())
}

/// Converts a JSON value into a `ConfigValue`.
///
/// This function maps JSON types (e.g., string, number, boolean, array, object) to
/// their corresponding `ConfigValue` representation.
///
/// # Arguments
///
/// * `value` - A reference to the JSON value to convert.
///
/// # Returns
///
/// A `ConfigValue` representing the converted value.
fn json_value_to_config_value(value: &JsonValue) -> ConfigValue {
    match value {
        JsonValue::String(s) => ConfigValue::String(s.clone()),
        JsonValue::Number(n) => {
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

/// Processes include directives in a JSON file.
///
/// This function handles both single file includes and arrays of include paths,
/// resolving the paths and parsing the included files.
///
/// # Arguments
///
/// * `config` - A mutable reference to the `Config` instance to update.
/// * `include_value` - The JSON value representing the include directive.
/// * `base_path` - The base path of the current JSON file.
///
/// # Returns
///
/// * `Ok(())` - If the include is processed successfully.
/// * `Err(ConfigError)` - If an error occurs during processing.
fn process_includes(config: &mut Config, include_value: &JsonValue, base_path: &Path) -> Result<(), ConfigError> {
    match include_value {
        JsonValue::String(include_path) => {
            process_single_include(config, include_path, base_path)?;
        },
        JsonValue::Array(includes) => {
            for include_item in includes {
                if let JsonValue::String(include_path) = include_item {
                    process_single_include(config, include_path, base_path)?;
                } else {
                    return Err(ConfigError::IncludeError(
                        "Includes must be strings".to_string()
                    ));
                }
            }
        },
        _ => {
            return Err(ConfigError::IncludeError(
                "Invalid include format. Must be a string or an array of strings".to_string()
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
/// * `base_path` - The base path of the current JSON file.
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
                .map_err(|e| ConfigError::IncludeError(format!("Error reading included file {}: {}",
                                                               resolved_path.display(), e)))?;

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
                let extension = resolved_path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");

                match extension {
                    "json" => parse_json(config, &content, &resolved_path)?,
                    "yaml" | "yml" => crate::formats::yaml::parse_yaml(config, &content, &resolved_path)?,
                    "toml" => crate::formats::toml::parse_toml(config, &content, &resolved_path)?,
                    "ini" => crate::formats::ini::parse_ini(config, &content, &resolved_path)?,
                    _ => {
                        parse_json(config, &content, &resolved_path)?;
                    }
                }
            }
        } else {
            return Err(ConfigError::IncludeError(format!("Included file not found: {}",
                                                         resolved_path.display())));
        }
    }

    Ok(())
}

/// Converts a `ConfigValue` into a JSON value.
///
/// This function maps `ConfigValue` types (e.g., string, integer, float, boolean, array, table)
/// to their corresponding JSON representation.
///
/// # Arguments
///
/// * `value` - A reference to the `ConfigValue` to convert.
///
/// # Returns
///
/// A `JsonValue` representing the converted value.
fn config_value_to_json_value(value: &ConfigValue) -> JsonValue {
    match value {
        ConfigValue::String(s) => JsonValue::String(s.clone()),
        ConfigValue::Integer(i) => JsonValue::Number((*i).into()),
        ConfigValue::Float(f) => {
            match serde_json::Number::from_f64(*f) {
                Some(num) => JsonValue::Number(num),
                None => JsonValue::String(f.to_string()),
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

/// Writes the configuration to a JSON file.
///
/// This function serializes the given `Config` instance into the JSON format
/// and writes it to the specified file path.
///
/// # Arguments
///
/// * `config` - A reference to the `Config` instance to serialize.
/// * `path` - The path to the output JSON file.
///
/// # Returns
///
/// * `Ok(())` - If the writing is successful.
/// * `Err(ConfigError)` - If an error occurs during writing.
pub fn write_json(config: &Config, path: &Path) -> Result<(), ConfigError> {
    let mut file = File::create(path).map_err(ConfigError::Io)?;

    writeln!(file, "#!config/json").map_err(ConfigError::Io)?;

    let mut root_obj = JsonMap::new();

    for (section, values) in &config.values {
        if section == "default" {
            for (key, value) in values {
                root_obj.insert(key.clone(), config_value_to_json_value(value));
            }
        } else {
            let mut section_obj = JsonMap::new();
            for (key, value) in values {
                section_obj.insert(key.clone(), config_value_to_json_value(value));
            }

            if !section_obj.is_empty() {
                root_obj.insert(section.clone(), JsonValue::Object(section_obj));
            }
        }
    }

    let json_string = serde_json::to_string_pretty(&JsonValue::Object(root_obj))
        .map_err(|e| ConfigError::Generic(format!("JSON serialization error: {}", e)))?;

    write!(file, "{}", json_string).map_err(ConfigError::Io)?;

    Ok(())
}