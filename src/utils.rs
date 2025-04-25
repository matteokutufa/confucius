//! Utility functions for the library

use std::env;
use std::path::{Path, PathBuf};
use crate::ConfigError;
use path_clean::PathClean;

/// Retrieves the current username.
///
/// This function attempts to determine the current user's name by checking the
/// home directory or environment variables. It provides fallbacks for different
/// operating systems.
///
/// # Returns
///
/// * `Ok(String)` - The username as a string if successfully determined.
/// * `Err(ConfigError)` - If the username cannot be determined.
pub fn get_current_username() -> Result<String, ConfigError> {
    if let Some(home_dir) = home::home_dir() {
        if let Some(home_dir_str) = home_dir.to_str() {
            return Ok(home_dir_str.to_string());
        }
    }

    // Fallback: try to get it from the environment variable
    if let Ok(user) = env::var("USER") {
        return Ok(user);
    }

    // Fallback for Windows
    if let Ok(user) = env::var("USERNAME") {
        return Ok(user);
    }

    Err(ConfigError::Generic("Impossibile determinare il nome utente".to_string()))
}

/// Resolves a relative path against a base file.
///
/// This function computes the absolute path of a relative path by resolving it
/// against the directory of a base file. It also normalizes the resulting path
/// by removing redundant components (e.g., `../`, `./`).
///
/// # Arguments
///
/// * `base_file` - A reference to a `Path` representing the base file.
/// * `relative_path` - A string slice representing the relative path to resolve.
///
/// # Returns
///
/// A `PathBuf` containing the resolved and normalized path.
pub fn resolve_path(base_file: &Path, relative_path: &str) -> PathBuf {
    let base_dir = if let Some(parent) = base_file.parent() {
        parent
    } else {
        Path::new(".")
    };

    let path = if Path::new(relative_path).is_absolute() {
        PathBuf::from(relative_path)
    } else {
        base_dir.join(relative_path)
    };

    // Normalize the path (removes ../, ./, etc.)
    path.clean()
}

/// Checks if a string is enclosed in double quotes.
///
/// # Arguments
///
/// * `s` - A string slice to check.
///
/// # Returns
///
/// `true` if the string starts and ends with double quotes, otherwise `false`.
pub fn is_quoted(s: &str) -> bool {
    s.starts_with('"') && s.ends_with('"')
}

/// Removes double quotes from a string.
///
/// This function removes the leading and trailing double quotes from a string
/// if they exist. It also handles escaped quotes within the string.
///
/// # Arguments
///
/// * `s` - A string slice to unquote.
///
/// # Returns
///
/// A `String` with the double quotes removed and escape sequences processed.
pub fn unquote(s: &str) -> String {
    if is_quoted(s) {
        // Extract the string without the leading and trailing quotes
        let content = &s[1..s.len()-1];

        // Handle escape sequences by replacing \" with "
        let mut result = String::with_capacity(content.len());
        let mut chars = content.chars().peekable();
        let mut in_escape = false;

        while let Some(c) = chars.next() {
            if in_escape {
                // If in escape mode, include the character as is
                result.push(c);
                in_escape = false;
            } else if c == '\\' && chars.peek() == Some(&'"') {
                // If a backslash is followed by a quote, treat it as an escape
                in_escape = true;
            } else {
                result.push(c);
            }
        }

        result
    } else {
        s.to_string()
    }
}

/// Removes comments from a line.
///
/// Comments are defined as anything following a `#` character that is not
/// inside double quotes.
///
/// # Arguments
///
/// * `line` - A string slice representing the line to process.
///
/// # Returns
///
/// A `String` with the comments removed and trailing whitespace trimmed.
pub fn strip_comments(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                result.push(c);
            },
            '#' if !in_quotes => {
                break; // Comment found, stop processing
            },
            _ => {
                result.push(c);
            }
        }
    }

    result.trim_end().to_string()
}