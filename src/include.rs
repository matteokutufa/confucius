// src/include.rs
//! Management of inclusion directives in configuration files

use std::fs;
use std::path::Path;
use glob::glob;

use crate::{Config, ConfigError, ConfigFormat};
use crate::utils;
use crate::formats;

/// Processes a glob pattern inclusion.
///
/// This function resolves a glob pattern relative to a base path and includes
/// the content of all matching files into the configuration.
///
/// # Arguments
///
/// * `config` - A mutable reference to the `Config` instance where the content will be included.
/// * `glob_pattern` - A string slice representing the glob pattern to match files.
/// * `base_path` - A reference to a `Path` representing the base path for resolving the glob pattern.
///
/// # Returns
///
/// * `Ok(())` - If all matching files are successfully included.
/// * `Err(ConfigError)` - If an error occurs during glob resolution, file reading, or content inclusion.
pub fn process_glob_include(config: &mut Config, glob_pattern: &str, base_path: &Path) -> Result<(), ConfigError> {
    // Resolve the pattern relative to the base path
    let resolved_pattern = utils::resolve_path(base_path, glob_pattern);
    let pattern_str = resolved_pattern.to_string_lossy();

    // Use the glob library to find all matching files
    let entries = glob(&pattern_str)
        .map_err(|e| ConfigError::IncludeError(format!("Error in glob pattern: {}", e)))?;

    let mut found_any = false;

    // For each matching file
    for entry in entries {
        match entry {
            Ok(path) => {
                found_any = true;

                // Read the content of the file
                let content = fs::read_to_string(&path)
                    .map_err(|e| ConfigError::IncludeError(format!("Error reading included file {}: {}",
                                                                   path.display(), e)))?;

                // Determine the format and include the content
                include_content(config, &content, &path)?;
            },
            Err(e) => {
                return Err(ConfigError::IncludeError(format!("Error expanding glob: {}", e)));
            }
        }
    }

    if !found_any {
        return Err(ConfigError::IncludeError(format!("No files found for pattern: {}", glob_pattern)));
    }

    Ok(())
}

/// Includes the content of a file into the configuration based on its format.
///
/// This function determines the format of the file content and parses it
/// accordingly to include it into the configuration.
///
/// # Arguments
///
/// * `config` - A mutable reference to the `Config` instance where the content will be included.
/// * `content` - A string slice containing the content of the file.
/// * `path` - A reference to a `Path` representing the file path.
///
/// # Returns
///
/// * `Ok(())` - If the content is successfully included.
/// * `Err(ConfigError)` - If an error occurs during format detection or parsing.
fn include_content(config: &mut Config, content: &str, path: &Path) -> Result<(), ConfigError> {
    // Determine the format from the content
    let format = detect_format_from_content(content);

    // Parse the content based on the format
    match format {
        ConfigFormat::Ini => formats::ini::parse_ini(config, content, path)?,
        ConfigFormat::Toml => return Err(ConfigError::UnsupportedFormat("TOML".to_string())),
        ConfigFormat::Yaml => return Err(ConfigError::UnsupportedFormat("YAML".to_string())),
        ConfigFormat::Json => return Err(ConfigError::UnsupportedFormat("JSON".to_string())),
        ConfigFormat::Unknown => {
            // If the format is unknown, assume INI
            formats::ini::parse_ini(config, content, path)?
        }
    }

    Ok(())
}

/// Detects the format from the content of a file.
///
/// This function reads the first line of the content to determine the format.
/// If the first line starts with `#!config/FORMAT`, the format is extracted.
/// If no format is specified, the default format is assumed to be INI.
///
/// # Arguments
///
/// * `content` - A string slice containing the content of the file.
///
/// # Returns
///
/// The detected configuration format as a `ConfigFormat` enum.
fn detect_format_from_content(content: &str) -> ConfigFormat {
    // Read the first line to determine the format
    let first_line = content.lines().next().unwrap_or("");

    // If the first line is in the format #!config/FORMAT
    if first_line.starts_with("#!config/") {
        let format_str = first_line.trim_start_matches("#!config/").trim();
        ConfigFormat::from(format_str)
    } else {
        // If not specified, assume INI
        ConfigFormat::Ini
    }
}