//! Implementation of the parser and writer for the INI format.

use std::fs::{self, File};
use std::io::{BufRead, Write};
use std::path::Path;
use regex::Regex;

use crate::{Config, ConfigError, ConfigValue};
use crate::include;
use crate::utils;

/// Parses an INI file and updates the provided configuration.
///
/// This function reads the content of an INI file, processes its sections, key-value pairs,
/// and include directives, and updates the given `Config` instance accordingly.
///
/// # Arguments
///
/// * `config` - A mutable reference to the `Config` instance to update.
/// * `content` - The content of the INI file as a string.
/// * `path` - The path to the INI file being parsed.
///
/// # Returns
///
/// * `Ok(())` - If the parsing is successful.
/// * `Err(ConfigError)` - If an error occurs during parsing.
pub fn parse_ini(config: &mut Config, content: &str, path: &Path) -> Result<(), ConfigError> {
    let mut current_section = "default".to_string();
    let section_regex = Regex::new(r"^\s*\[(.*?)\]\s*$").unwrap();
    let kv_regex = Regex::new(r"^\s*(.*?)\s*=\s*(.*?)\s*$").unwrap();
    let include_regex = Regex::new(r"^\s*include\s*=\s*(.*?)\s*$").unwrap();

    // Skip the first line if it contains the format (#!config/...)
    let lines_to_process = if content.lines().next().unwrap_or("").starts_with("#!config/") {
        content.lines().skip(1).collect::<Vec<_>>()
    } else {
        content.lines().collect::<Vec<_>>()
    };

    for line in lines_to_process {
        // Remove comments from the line
        let line = utils::strip_comments(line);
        if line.is_empty() {
            continue;
        }

        // Check if it is an include directive
        if let Some(cap) = include_regex.captures(&line) {
            let include_path = cap.get(1).unwrap().as_str();
            process_include(config, include_path, path)?;
            continue;
        }

        // Check if it is a section
        if let Some(cap) = section_regex.captures(&line) {
            current_section = cap.get(1).unwrap().as_str().to_string();
            continue;
        }

        // Otherwise, it is a key-value pair
        if let Some(cap) = kv_regex.captures(&line) {
            let key = cap.get(1).unwrap().as_str();
            let value_str = cap.get(2).unwrap().as_str();

            // Convert the value to the appropriate type
            let value = parse_value(value_str);

            // Insert into the configuration
            config.set(&current_section, key, value);
        }
    }

    Ok(())
}

/// Processes an include directive in an INI file.
///
/// This function handles both single file includes and glob patterns, resolving
/// the paths and parsing the included files.
///
/// # Arguments
///
/// * `config` - A mutable reference to the `Config` instance to update.
/// * `include_path` - The path or glob pattern of the file(s) to include.
/// * `base_path` - The base path of the current INI file.
///
/// # Returns
///
/// * `Ok(())` - If the include is processed successfully.
/// * `Err(ConfigError)` - If an error occurs during processing.
fn process_include(config: &mut Config, include_path: &str, base_path: &Path) -> Result<(), ConfigError> {
    // If the include is a glob pattern, include all matching files
    if include_path.contains('*') {
        include::process_glob_include(config, include_path, base_path)?;
    } else {
        // Otherwise, include a single file
        let resolved_path = utils::resolve_path(base_path, include_path);
        if resolved_path.exists() {
            let content = fs::read_to_string(&resolved_path)
                .map_err(|e| ConfigError::IncludeError(format!("Error reading included file {}: {}",
                                                               resolved_path.display(), e)))?;

            parse_ini(config, &content, &resolved_path)?;
        } else {
            return Err(ConfigError::IncludeError(format!("Included file not found: {}",
                                                         resolved_path.display())));
        }
    }

    Ok(())
}

/// Converts a string into a `ConfigValue`.
///
/// This function attempts to parse the string into various types, such as boolean,
/// integer, float, or string, and returns the corresponding `ConfigValue`.
///
/// # Arguments
///
/// * `value_str` - The string to convert.
///
/// # Returns
///
/// A `ConfigValue` representing the parsed value.
fn parse_value(value_str: &str) -> ConfigValue {
    // If it is quoted, it is a string
    if utils::is_quoted(value_str) {
        return ConfigValue::String(utils::unquote(value_str));
    }

    // Try to convert to boolean
    match value_str.to_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => return ConfigValue::Boolean(true),
        "false" | "no" | "off" | "0" => return ConfigValue::Boolean(false),
        _ => {}
    }

    // Try to convert to integer
    if let Ok(i) = value_str.parse::<i64>() {
        return ConfigValue::Integer(i);
    }

    // Try to convert to float
    if let Ok(f) = value_str.parse::<f64>() {
        return ConfigValue::Float(f);
    }

    // Otherwise, it is a string
    ConfigValue::String(value_str.to_string())
}

/// Writes the configuration to an INI file.
///
/// This function serializes the given `Config` instance into the INI format
/// and writes it to the specified file path.
///
/// # Arguments
///
/// * `config` - A reference to the `Config` instance to serialize.
/// * `path` - The path to the output INI file.
///
/// # Returns
///
/// * `Ok(())` - If the writing is successful.
/// * `Err(ConfigError)` - If an error occurs during writing.
pub fn write_ini(config: &Config, path: &Path) -> Result<(), ConfigError> {
    let mut file = File::create(path).map_err(ConfigError::Io)?;

    // Write the format header
    writeln!(file, "#!config/ini").map_err(ConfigError::Io)?;

    // For each section
    for (section, values) in &config.values {
        // Skip the default section if it is empty
        if section == "default" && values.is_empty() {
            continue;
        }

        // Write the section header
        writeln!(file, "\n[{}]", section).map_err(ConfigError::Io)?;

        // Write each key-value pair
        for (key, value) in values {
            let value_str = format_value(value);
            writeln!(file, "{} = {}", key, value_str).map_err(ConfigError::Io)?;
        }
    }

    Ok(())
}

/// Formats a `ConfigValue` as a string.
///
/// This function converts a `ConfigValue` into its string representation
/// for serialization in the INI format.
///
/// # Arguments
///
/// * `value` - A reference to the `ConfigValue` to format.
///
/// # Returns
///
/// A string representing the formatted value.
fn format_value(value: &ConfigValue) -> String {
    match value {
        ConfigValue::String(s) => format!("\"{}\"", s),
        ConfigValue::Integer(i) => i.to_string(),
        ConfigValue::Float(f) => f.to_string(),
        ConfigValue::Boolean(b) => {
            if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }
        },
        ConfigValue::Array(a) => {
            // INI format does not support arrays, so join as a string
            let items: Vec<String> = a.iter().map(format_value).collect();
            format!("\"{}\"", items.join(", "))
        },
        ConfigValue::Table(t) => {
            // INI format does not support nested tables, so convert to a string
            format!("\"{}\"", format!("{:?}", t))
        },
    }
}