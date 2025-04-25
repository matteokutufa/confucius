//! Implementation of the parser and writer for the TOML format.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;
use toml::{Value as TomlValue, Table as TomlTable};

use crate::{Config, ConfigError, ConfigValue};
use crate::include;
use crate::utils;

/// Parses a TOML file and updates the provided configuration.
///
/// This function reads the content of a TOML file, processes its sections, key-value pairs,
/// and include directives, and updates the given `Config` instance accordingly.
///
/// # Arguments
///
/// * `config` - A mutable reference to the `Config` instance to update.
/// * `content` - The content of the TOML file as a string.
/// * `path` - The path to the TOML file being parsed.
///
/// # Returns
///
/// * `Ok(())` - If the parsing is successful.
/// * `Err(ConfigError)` - If an error occurs during parsing.
pub fn parse_toml(config: &mut Config, content: &str, path: &Path) -> Result<(), ConfigError> {
    let content_to_parse = if content.lines().next().unwrap_or("").starts_with("#!config/") {
        content.lines().skip(1).collect::<Vec<_>>().join("\n")
    } else {
        content.to_string()
    };

    let parsed_toml: TomlTable = content_to_parse.parse()
        .map_err(|e| ConfigError::ParseError(format!("Error in TOML parsing: {}", e)))?;

    if let Some(include_value) = parsed_toml.get("include") {
        process_includes(config, include_value, path)?;
    }

    for (section_name, section_value) in &parsed_toml {
        if section_name == "include" {
            continue;
        }

        match section_value {
            TomlValue::Table(table) => {
                for (key, value) in table {
                    let config_value = toml_value_to_config_value(value);
                    config.set(section_name, key, config_value);
                }
            },
            _ => {
                let config_value = toml_value_to_config_value(section_value);
                config.set("default", section_name, config_value);
            }
        }
    }

    Ok(())
}

/// Converts a TOML value into a `ConfigValue`.
///
/// This function maps TOML types (e.g., string, integer, float, boolean, array, table)
/// to their corresponding `ConfigValue` representation.
///
/// # Arguments
///
/// * `value` - A reference to the TOML value to convert.
///
/// # Returns
///
/// A `ConfigValue` representing the converted value.
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
        TomlValue::Datetime(dt) => ConfigValue::String(dt.to_string()),
    }
}

/// Processes include directives in a TOML file.
///
/// This function handles both single file includes and arrays of include paths,
/// resolving the paths and parsing the included files.
///
/// # Arguments
///
/// * `config` - A mutable reference to the `Config` instance to update.
/// * `include_value` - The TOML value representing the include directive.
/// * `base_path` - The base path of the current TOML file.
///
/// # Returns
///
/// * `Ok(())` - If the include is processed successfully.
/// * `Err(ConfigError)` - If an error occurs during processing.
fn process_includes(config: &mut Config, include_value: &TomlValue, base_path: &Path) -> Result<(), ConfigError> {
    match include_value {
        TomlValue::String(include_path) => {
            process_single_include(config, include_path, base_path)?;
        },
        TomlValue::Array(includes) => {
            for include_item in includes {
                if let TomlValue::String(include_path) = include_item {
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
                "The inclusion format is invalid. It must be a string or an array of strings".to_string()
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
/// * `base_path` - The base path of the current TOML file.
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
                .map_err(|e| ConfigError::IncludeError(format!("Error reading included file {}: {}", resolved_path.display(), e)))?;

            let first_line = content.lines().next().unwrap_or("");
            if first_line.starts_with("#!config/toml") {
                parse_toml(config, &content, &resolved_path)?;
            } else if first_line.starts_with("#!config/ini") {
                crate::formats::ini::parse_ini(config, &content, &resolved_path)?;
            } else if first_line.starts_with("#!config/yaml") {
                return Err(ConfigError::UnsupportedFormat("YAML".to_string()));
            } else {
                let extension = resolved_path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");

                match extension {
                    "toml" => parse_toml(config, &content, &resolved_path)?,
                    "ini" => crate::formats::ini::parse_ini(config, &content, &resolved_path)?,
                    "yaml" | "yml" => return Err(ConfigError::UnsupportedFormat("YAML".to_string())),
                    _ => {
                        parse_toml(config, &content, &resolved_path)?;
                    }
                }
            }
        } else {
            return Err(ConfigError::IncludeError(format!("Included file not found: {}", resolved_path.display())));
        }
    }

    Ok(())
}

/// Converts a `ConfigValue` into a TOML value.
///
/// This function maps `ConfigValue` types (e.g., string, integer, float, boolean, array, table)
/// to their corresponding TOML representation.
///
/// # Arguments
///
/// * `value` - A reference to the `ConfigValue` to convert.
///
/// # Returns
///
/// A `TomlValue` representing the converted value.
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

/// Writes the configuration to a TOML file.
///
/// This function serializes the given `Config` instance into the TOML format
/// and writes it to the specified file path.
///
/// # Arguments
///
/// * `config` - A reference to the `Config` instance to serialize.
/// * `path` - The path to the output TOML file.
///
/// # Returns
///
/// * `Ok(())` - If the writing is successful.
/// * `Err(ConfigError)` - If an error occurs during writing.
pub fn write_toml(config: &Config, path: &Path) -> Result<(), ConfigError> {
    let mut file = File::create(path).map_err(ConfigError::Io)?;

    writeln!(file, "#!config/toml").map_err(ConfigError::Io)?;

    let mut root_table = TomlTable::new();

    for (section, values) in &config.values {
        if section == "default" {
            for (key, value) in values {
                root_table.insert(key.clone(), config_value_to_toml_value(value));
            }
        } else {
            let mut section_table = TomlTable::new();
            for (key, value) in values {
                section_table.insert(key.clone(), config_value_to_toml_value(value));
            }

            if !section_table.is_empty() {
                root_table.insert(section.clone(), TomlValue::Table(section_table));
            }
        }
    }

    let toml_string = toml::to_string_pretty(&root_table)
        .map_err(|e| ConfigError::Generic(format!("Error in TOML serialization: {}", e)))?;

    writeln!(file, "{}", toml_string).map_err(ConfigError::Io)?;

    Ok(())
}