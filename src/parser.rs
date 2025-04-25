//! Generic parser for configuration files

use std::path::Path;
use crate::{Config, ConfigError, ConfigFormat};
use crate::formats;

/// Parses a configuration file based on its format.
///
/// This function reads the content of the specified file and determines its
/// format. Based on the detected format, it delegates the parsing to the
/// appropriate format-specific parser.
///
/// # Arguments
///
/// * `config` - A mutable reference to the `Config` instance where the parsed
///   data will be stored.
/// * `path` - A reference to a `Path` representing the file to be parsed.
///
/// # Returns
///
/// * `Ok(())` - If the file is successfully parsed and the data is loaded into
///   the `Config` instance.
/// * `Err(ConfigError)` - If an error occurs during file reading, format
///   detection, or parsing.
pub fn parse_file(config: &mut Config, path: &Path) -> Result<(), ConfigError> {
    let content = std::fs::read_to_string(path).map_err(ConfigError::Io)?;

    // Determine the format from the content
    let format = detect_format(&content);

    match format {
        ConfigFormat::Ini => formats::ini::parse_ini(config, &content, path),
        ConfigFormat::Toml => formats::toml::parse_toml(config, &content, path),
        ConfigFormat::Yaml => formats::yaml::parse_yaml(config, &content, path),
        ConfigFormat::Json => formats::json::parse_json(config, &content, path),
        ConfigFormat::Unknown => Err(ConfigError::UnsupportedFormat("Sconosciuto".to_string())),
    }
}

/// Detects the format of a configuration file from its content.
///
/// This function examines the first line of the file content to determine its
/// format. If the first line starts with `#!config/FORMAT`, the format is
/// extracted. If no format is specified, the default format is assumed to be INI.
///
/// # Arguments
///
/// * `content` - A string slice containing the content of the configuration file.
///
/// # Returns
///
/// The detected configuration format as a `ConfigFormat` enum.
fn detect_format(content: &str) -> ConfigFormat {
    // Read the first line
    if let Some(first_line) = content.lines().next() {
        if first_line.starts_with("#!config/") {
            let format_str = first_line.trim_start_matches("#!config/").trim();
            return ConfigFormat::from(format_str);
        }
    }

    // Default to INI format
    ConfigFormat::Ini
}