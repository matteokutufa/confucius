// src/lib.rs
//! # Confucius
//!
//! > "Constancy is the virtue by which all other virtues bear fruit." - Confucius
//!
//! Just as Confucius provided wisdom for life, Confucius provides
//! wisdom for configuring your applications.
//!
//! `confucius` is a library for managing configuration files with support for:
//! - Automatic search for configuration files in standard paths
//! - Support for different formats (.ini, .toml, .yaml, .json)
//! - Include mechanism for multiple files
//! - Format identification through shebang (#!config/FORMAT)
//! - Support for comments and text values

use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use thiserror::Error;
use serde::{Serialize, Deserialize};

pub mod validation;
mod parser;
mod formats;
mod include;
mod utils;


/// Supported configuration file formats.
///
/// This enum represents the different formats that can be used for
/// configuration files. It includes common formats such as INI, TOML,
/// YAML, and JSON, as well as an `Unknown` variant for unsupported or
/// unrecognized formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// INI format.
    Ini,
    /// TOML format.
    Toml,
    /// YAML format.
    Yaml,
    /// JSON format.
    Json,
    /// Unknown or unsupported format.
    Unknown,
}

impl fmt::Display for ConfigFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigFormat::Ini => write!(f, "ini"),
            ConfigFormat::Toml => write!(f, "toml"),
            ConfigFormat::Yaml => write!(f, "yaml"),
            ConfigFormat::Json => write!(f, "json"),
            ConfigFormat::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<&str> for ConfigFormat {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "ini" => ConfigFormat::Ini,
            "toml" => ConfigFormat::Toml,
            "yaml" | "yml" => ConfigFormat::Yaml,
            "json" => ConfigFormat::Json,
            _ => ConfigFormat::Unknown,
        }
    }
}

/// Errors that can occur during configuration management.
///
/// This enum defines the possible errors that might be encountered
/// while working with configuration files, such as I/O issues,
/// unsupported formats, parsing errors, or missing configuration files.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// I/O error occurred while accessing a file or resource.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// The configuration format is unknown or unsupported.
    #[error("Configuration file format unknown or unsupported: {0}")]
    UnsupportedFormat(String),

    /// An error occurred while parsing the configuration file.
    #[error("Configuration file parsing error: {0}")]
    ParseError(String),

    /// The configuration file could not be found for the specified application.
    #[error("Configuration file not found for: {0}")]
    ConfigNotFound(String),

    /// An error occurred while including one or more files.
    #[error("File or files include error: {0}")]
    IncludeError(String),

    /// A generic or unknown error occurred.
    #[error("Unknown error: {0}")]
    Generic(String),
}

/// Represents a configuration value.
///
/// This enum is used to store different types of values that can be
/// found in a configuration file. It supports primitive types like
/// strings, integers, floats, and booleans, as well as complex types
/// like arrays and tables.
///
/// # Variants
///
/// * `String` - A string value.
/// * `Integer` - An integer value.
/// * `Float` - A floating-point value.
/// * `Boolean` - A boolean value.
/// * `Array` - A list of configuration values.
/// * `Table` - A map of string keys to configuration values.
#[derive(Debug, Clone)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
    Table(HashMap<String, ConfigValue>),
}

impl ConfigValue {
    /// Converts the configuration value to a string, if possible.
    ///
    /// This method attempts to extract the inner string value from the
    /// `ConfigValue` enum. If the value is of type `String`, it returns
    /// a reference to the string. Otherwise, it returns `None`.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the string if the value is
    /// of type `String`, or `None` otherwise.
    pub fn as_string(&self) -> Option<&String> {
        if let ConfigValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Converts the configuration value to an integer, if possible.
    ///
    /// This method attempts to extract the inner integer value from the
    /// `ConfigValue` enum. If the value is of type `Integer`, it returns
    /// the integer. Otherwise, it returns `None`.
    ///
    /// # Returns
    ///
    /// An `Option` containing the integer value if the `ConfigValue` is
    /// of type `Integer`, or `None` otherwise.
    pub fn as_integer(&self) -> Option<i64> {
        if let ConfigValue::Integer(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    /// Converts the configuration value to a floating-point number, if possible.
    ///
    /// This method attempts to extract the inner float value from the
    /// `ConfigValue` enum. If the value is of type `Float`, it returns
    /// the float. If the value is of type `Integer`, it converts the
    /// integer to a float and returns it. Otherwise, it returns `None`.
    ///
    /// # Returns
    ///
    /// An `Option` containing the float value if the `ConfigValue` is
    /// of type `Float` or `Integer`, or `None` otherwise.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(f) => Some(*f),
            ConfigValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Converts the configuration value to a boolean, if possible.
    ///
    /// This method attempts to extract the inner boolean value from the
    /// `ConfigValue` enum. If the value is of type `Boolean`, it returns
    /// the boolean. Otherwise, it returns `None`.
    ///
    /// # Returns
    ///
    /// An `Option` containing the boolean value if the `ConfigValue` is
    /// of type `Boolean`, or `None` otherwise.
    pub fn as_boolean(&self) -> Option<bool> {
        if let ConfigValue::Boolean(b) = self {
            Some(*b)
        } else {
            None
        }
    }
}

/// Implements the `Serialize` trait for the `ConfigValue` enum.
///
/// This implementation allows `ConfigValue` instances to be serialized
/// into formats supported by Serde, such as JSON, YAML, or TOML. Each
/// variant of the `ConfigValue` enum is serialized according to its type.
///
/// # Type Parameters
/// * `S` - The serializer type, which must implement the `serde::Serializer` trait.
///
/// # Arguments
/// * `serializer` - The serializer instance used to serialize the `ConfigValue`.
///
/// # Returns
/// * `Result<S::Ok, S::Error>` - Returns the result of the serialization process,
///   which is either a success (`S::Ok`) or an error (`S::Error`).
impl Serialize for ConfigValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            // Serializes a string value.
            ConfigValue::String(s) => serializer.serialize_str(s),

            // Serializes an integer value.
            ConfigValue::Integer(i) => serializer.serialize_i64(*i),

            // Serializes a floating-point value.
            ConfigValue::Float(f) => serializer.serialize_f64(*f),

            // Serializes a boolean value.
            ConfigValue::Boolean(b) => serializer.serialize_bool(*b),

            // Serializes an array of `ConfigValue` instances.
            ConfigValue::Array(arr) => {
                use serde::ser::SerializeSeq;
                let mut seq = serializer.serialize_seq(Some(arr.len()))?;
                for item in arr {
                    seq.serialize_element(item)?;
                }
                seq.end()
            },

            // Serializes a table (map) of string keys to `ConfigValue` instances.
            ConfigValue::Table(table) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(table.len()))?;
                for (k, v) in table {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            },
        }
    }
}

/// Implements the `Deserialize` trait for the `ConfigValue` enum.
///
/// This implementation allows `ConfigValue` instances to be deserialized
/// from formats supported by Serde, such as JSON, YAML, or TOML. Each
/// variant of the `ConfigValue` enum is deserialized according to its type.
///
/// # Type Parameters
/// * `'de` - The lifetime of the data being deserialized.
///
/// # Arguments
/// * `deserializer` - The deserializer instance used to deserialize the `ConfigValue`.
///
/// # Returns
/// * `Result<Self, D::Error>` - Returns the deserialized `ConfigValue` or an error.
impl<'de> Deserialize<'de> for ConfigValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor, SeqAccess, MapAccess};
        use std::fmt;

        /// Visitor for deserializing `ConfigValue`.
        ///
        /// This struct implements the `Visitor` trait to handle the deserialization
        /// of various types into the corresponding `ConfigValue` variants.
        struct ConfigValueVisitor;

        impl<'de> Visitor<'de> for ConfigValueVisitor {
            type Value = ConfigValue;

            /// Describes the expected input for this visitor.
            ///
            /// # Arguments
            /// * `formatter` - A mutable reference to the formatter.
            ///
            /// # Returns
            /// * `fmt::Result` - The result of the formatting operation.
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string, number, boolean, array, or map")
            }

            /// Visits a boolean value and converts it to `ConfigValue::Boolean`.
            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ConfigValue::Boolean(value))
            }

            /// Visits a signed integer and converts it to `ConfigValue::Integer`.
            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ConfigValue::Integer(value))
            }

            /// Visits an unsigned integer and converts it to `ConfigValue::Integer` or `ConfigValue::Float`.
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value <= i64::MAX as u64 {
                    Ok(ConfigValue::Integer(value as i64))
                } else {
                    Ok(ConfigValue::Float(value as f64))
                }
            }

            /// Visits a floating-point number and converts it to `ConfigValue::Float`.
            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ConfigValue::Float(value))
            }

            /// Visits a string slice and converts it to `ConfigValue::String`.
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ConfigValue::String(value.to_string()))
            }

            /// Visits a string and converts it to `ConfigValue::String`.
            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ConfigValue::String(value))
            }

            /// Visits a sequence and converts it to `ConfigValue::Array`.
            ///
            /// # Arguments
            /// * `seq` - The sequence access object.
            ///
            /// # Returns
            /// * `Result<Self::Value, A::Error>` - The deserialized array or an error.
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::new();

                while let Some(value) = seq.next_element()? {
                    values.push(value);
                }

                Ok(ConfigValue::Array(values))
            }

            /// Visits a map and converts it to `ConfigValue::Table`.
            ///
            /// # Arguments
            /// * `map` - The map access object.
            ///
            /// # Returns
            /// * `Result<Self::Value, M::Error>` - The deserialized table or an error.
            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut values = HashMap::new();

                while let Some((key, value)) = map.next_entry()? {
                    values.insert(key, value);
                }

                Ok(ConfigValue::Table(values))
            }
        }

        // Use the visitor to deserialize any type into a `ConfigValue`.
        deserializer.deserialize_any(ConfigValueVisitor)
    }
}

/// Implements the `Display` trait for the `ConfigValue` enum.
///
/// This implementation provides a string representation for each variant
/// of the `ConfigValue` enum. It formats the value as a string, integer,
/// float, boolean, array, or table, depending on the variant.
///
/// # Arguments
/// * `f` - A mutable reference to the formatter used to write the output.
///
/// # Returns
/// * `fmt::Result` - The result of the formatting operation, which is either
///   `Ok` if successful or an error if formatting fails.
impl fmt::Display for ConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Formats a string value.
            ConfigValue::String(s) => write!(f, "{}", s),

            // Formats an integer value.
            ConfigValue::Integer(i) => write!(f, "{}", i),

            // Formats a floating-point value.
            ConfigValue::Float(fl) => write!(f, "{}", fl),

            // Formats a boolean value.
            ConfigValue::Boolean(b) => write!(f, "{}", b),

            // Formats an array of `ConfigValue` instances.
            ConfigValue::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            },

            // Formats a table (map) of string keys to `ConfigValue` instances.
            ConfigValue::Table(map) => {
                write!(f, "{{")?;
                for (i, (key, val)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, val)?;
                }
                write!(f, "}}")
            }
        }
    }
}

/// Represents the main structure for configuration management.
///
/// This struct is used to manage configuration values for an application,
/// including the application's name, configuration values organized by
/// sections and keys, the format of the configuration file, and the path
/// to the loaded configuration file.
///
/// # Fields
///
/// * `app_name` - The name of the application (e.g., "galatea").
/// * `values` - A map of configuration values organized by section and key.
/// * `format` - The format of the configuration file (e.g., INI, TOML, YAML, JSON).
/// * `config_file_path` - The path to the loaded configuration file, if any.
#[derive(Debug)]
pub struct Config {
    /// The name of the application (e.g., "galatea").
    app_name: String,

    /// Configuration values organized by section and key.
    values: HashMap<String, HashMap<String, ConfigValue>>,

    /// The format of the configuration file.
    format: ConfigFormat,

    /// The path to the loaded configuration file, if any.
    config_file_path: Option<PathBuf>,
}

impl Config {
    /// Creates a new instance of `Config`.
    ///
    /// This constructor initializes a `Config` instance with the specified
    /// application name, an empty set of configuration values, an unknown
    /// configuration format, and no associated configuration file path.
    ///
    /// # Arguments
    ///
    /// * `app_name` - A string slice representing the name of the application.
    ///
    /// # Returns
    ///
    /// A new `Config` instance with default values.
    pub fn new(app_name: &str) -> Self {
        Config {
            app_name: app_name.to_string(),
            values: HashMap::new(),
            format: ConfigFormat::Unknown,
            config_file_path: None,
        }
    }

    /// Explicitly sets the configuration format.
    ///
    /// This method allows you to set the format of the configuration file
    /// (e.g., INI, TOML, YAML, JSON) explicitly. The format is stored in
    /// the `format` field of the `Config` struct.
    ///
    /// # Arguments
    ///
    /// * `format` - The desired configuration format as a `ConfigFormat` enum.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `Config` instance, allowing method chaining.
    pub fn set_format(&mut self, format: ConfigFormat) -> &mut Self {
        self.format = format;
        self
    }

    /// Retrieves the current configuration format.
    ///
    /// This method returns the format of the configuration file currently
    /// associated with the `Config` instance. The format is represented
    /// as a `ConfigFormat` enum.
    ///
    /// # Returns
    ///
    /// The current configuration format as a `ConfigFormat` enum.
    pub fn get_format(&self) -> ConfigFormat {
        self.format
    }


    /// Loads the configuration from predefined paths.
    ///
    /// This method attempts to locate and load a configuration file from a set
    /// of predefined search paths. It retrieves the current executable's path
    /// and the username of the current user to construct these paths. If a
    /// configuration file is found, it is loaded into the `Config` instance.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If a configuration file is successfully loaded.
    /// * `Err(ConfigError)` - If no configuration file is found or an error occurs.
    ///
    /// # Errors
    ///
    /// Returns a `ConfigError` in the following cases:
    /// - I/O error while retrieving the executable path.
    /// - Failure to retrieve the current username.
    /// - No configuration file is found in the predefined paths.
    pub fn load(&mut self) -> Result<(), ConfigError> {
        // Retrieve the current executable's path and the current username.
        let exec_path = env::current_exe().map_err(ConfigError::Io)?;
        let username = utils::get_current_username()?;

        // Build the search paths for the configuration file.
        let search_paths = self.build_search_paths(&exec_path, &username);

        // Search for the first available configuration file.
        for path in search_paths {
            if path.exists() {
                return self.load_from_file(&path);
            }
        }

        // Return an error if no configuration file is found.
        Err(ConfigError::ConfigNotFound(self.app_name.clone()))
    }

    /// Loads the configuration from a specific file.
    ///
    /// This method reads the content of the specified configuration file,
    /// determines its format, and parses it into the `Config` structure.
    /// The file path is stored in the `config_file_path` field of the `Config` instance.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to a `Path` representing the file to load the configuration from.
    ///
    /// # Errors
    ///
    /// Returns a `ConfigError` if:
    /// - The file cannot be read (I/O error).
    /// - The format of the configuration file is unsupported.
    /// - Parsing the file content fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut config = Config::new("my_app");
    /// config.load_from_file(Path::new("/path/to/config.toml")).unwrap();
    /// ```
    pub fn load_from_file(&mut self, path: &Path) -> Result<(), ConfigError> {
        let content = fs::read_to_string(path).map_err(ConfigError::Io)?;
        self.config_file_path = Some(path.to_path_buf());

        // Determiniamo il formato dal contenuto
        self.detect_format_from_content(&content)?;

        // Parserizziamo il contenuto in base al formato
        match self.format {
            ConfigFormat::Ini => ini::parse_ini(self, &content, path)?,
            ConfigFormat::Toml => toml::parse_toml(self, &content, path)?,
            ConfigFormat::Yaml => yaml::parse_yaml(self, &content, path)?,
            ConfigFormat::Json => json::parse_json(self, &content, path)?,
            ConfigFormat::Unknown => return Err(ConfigError::UnsupportedFormat("Unknown".to_string())),
        }

        Ok(())
    }

    /// Builds a list of potential search paths for the configuration file.
    ///
    /// This function generates a vector of `PathBuf` objects representing
    /// the possible locations where the configuration file might be found.
    /// The paths are constructed based on the application's name, the current
    /// execution path, and the username of the user.
    ///
    /// # Arguments
    ///
    /// * `exec_path` - A reference to a `Path` representing the current executable's path.
    /// * `username` - A string slice representing the current user's username.
    ///
    /// # Returns
    ///
    /// A `Vec<PathBuf>` containing the potential search paths for the configuration file.
    fn build_search_paths(&self, exec_path: &Path, username: &str) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let config_filename = format!("{}.conf", self.app_name);

        // /etc/myapp/myapp.conf
        paths.push(PathBuf::from(format!("/etc/{}/{}", self.app_name, config_filename)));

        // /etc/myapp.conf
        paths.push(PathBuf::from(format!("/etc/{}", config_filename)));

        // /opt/etc/myapp.conf
        paths.push(PathBuf::from(format!("/opt/etc/{}", config_filename)));

        // ~/.config/myapp/myapp.conf
        paths.push(PathBuf::from(format!("/home/{}/.config/{}/{}", username, self.app_name, config_filename)));

        // ~/.config/myapp.conf
        paths.push(PathBuf::from(format!("/home/{}/.config/{}", username, config_filename)));

        // Path of executable file
        if let Some(exec_dir) = exec_path.parent() {
            paths.push(exec_dir.join(&config_filename));
        }

        paths
    }

    /// Detects the configuration format from the file content.
    ///
    /// This function reads the first line of the provided content to determine
    /// the configuration format. If the first line starts with `#!config/FORMAT`,
    /// the format is extracted and set in the `format` field of the `Config` struct.
    /// If the format is unknown or unsupported, an error is returned. If no format
    /// is specified, the default format is assumed to be INI.
    ///
    /// # Arguments
    ///
    /// * `content` - A string slice containing the content of the configuration file.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the format is successfully detected and set.
    /// * `Err(ConfigError)` - If the format is unknown or unsupported.
    ///
    /// # Errors
    ///
    /// Returns a `ConfigError::UnsupportedFormat` if the format specified in the
    /// content is not recognized.
    fn detect_format_from_content(&mut self, content: &str) -> Result<(), ConfigError> {
        // Read the first line to determine the format.
        let first_line = content.lines().next().unwrap_or("");

        // If the first line is in the format #!config/FORMAT
        if first_line.starts_with("#!config/") {
            let format_str = first_line.trim_start_matches("#!config/").trim();
            self.format = ConfigFormat::from(format_str);

            if self.format == ConfigFormat::Unknown {
                return Err(ConfigError::UnsupportedFormat(format_str.to_string()));
            }
        } else {
            // For now, assume INI if no format is specified.
            self.format = ConfigFormat::Ini;
        }

        Ok(())
    }

    /// Retrieves a value from the configuration.
    ///
    /// This method looks up a value in the configuration by its section and key.
    ///
    /// # Arguments
    ///
    /// * `section` - A string slice representing the section name.
    /// * `key` - A string slice representing the key name.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the `ConfigValue` if the value exists,
    /// or `None` if the section or key is not found.
    pub fn get(&self, section: &str, key: &str) -> Option<&ConfigValue> {
        self.values.get(section).and_then(|section_map| section_map.get(key))
    }

    /// Sets a value in the configuration.
    ///
    /// This method inserts or updates a value in the configuration under the specified
    /// section and key.
    ///
    /// # Arguments
    ///
    /// * `section` - A string slice representing the section name.
    /// * `key` - A string slice representing the key name.
    /// * `value` - The `ConfigValue` to be set.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `Config` instance, allowing method chaining.
    pub fn set(&mut self, section: &str, key: &str, value: ConfigValue) -> &mut Self {
        self.values
            .entry(section.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), value);
        self
    }

    /// Saves the configuration to the current file.
    ///
    /// This method writes the configuration to the file specified in the `config_file_path`
    /// field. If no file path is set, an error is returned.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the configuration is successfully saved.
    /// * `Err(ConfigError)` - If no file path is set or an error occurs during saving.
    pub fn save(&self) -> Result<(), ConfigError> {
        if let Some(path) = &self.config_file_path {
            self.save_to_file(path)
        } else {
            Err(ConfigError::Generic("Nessun file di configurazione caricato".to_string()))
        }
    }

    /// Saves the configuration to a specific file.
    ///
    /// This method writes the configuration to the specified file path in the format
    /// defined by the `format` field.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to a `Path` representing the file to save the configuration to.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the configuration is successfully saved.
    /// * `Err(ConfigError)` - If an error occurs during saving or the format is unsupported.
    pub fn save_to_file(&self, path: &Path) -> Result<(), ConfigError> {
        match self.format {
            ConfigFormat::Ini => formats::ini::write_ini(self, path)?,
            ConfigFormat::Toml => formats::toml::write_toml(self, path)?,
            ConfigFormat::Yaml => formats::yaml::write_yaml(self, path)?,
            ConfigFormat::Json => formats::json::write_json(self, path)?,
            ConfigFormat::Unknown => return Err(ConfigError::UnsupportedFormat("Sconosciuto".to_string())),
        }

        Ok(())
    }

    /// Retrieves a string value from the configuration.
    ///
    /// This method is a convenience wrapper that looks up a configuration value
    /// by section and key, and returns it as a string if found.
    ///
    /// # Arguments
    ///
    /// * `section` - A string slice representing the section name.
    /// * `key` - A string slice representing the key name.
    /// * `default` - An optional default value to return if the key is not found.
    ///
    /// # Returns
    ///
    /// An `Option<String>` containing the string value if found, or the default value
    /// if provided, or `None` if the key is not found and no default was provided.
    pub fn get_string(&self, section: &str, key: &str, default: Option<&str>) -> Option<String> {
        match self.get(section, key) {
            Some(value) => value.as_string().map(|s| s.clone()),
            None => default.map(|s| s.to_string()),
        }
    }

    /// Retrieves an integer value from the configuration.
    ///
    /// This method is a convenience wrapper that looks up a configuration value
    /// by section and key, and returns it as an integer if found.
    ///
    /// # Arguments
    ///
    /// * `section` - A string slice representing the section name.
    /// * `key` - A string slice representing the key name.
    /// * `default` - An optional default value to return if the key is not found.
    ///
    /// # Returns
    ///
    /// An `Option<i64>` containing the integer value if found, or the default value
    /// if provided, or `None` if the key is not found and no default was provided.
    pub fn get_integer(&self, section: &str, key: &str, default: Option<i64>) -> Option<i64> {
        match self.get(section, key) {
            Some(value) => value.as_integer().or(default),
            None => default,
        }
    }

    /// Retrieves a float value from the configuration.
    ///
    /// This method is a convenience wrapper that looks up a configuration value
    /// by section and key, and returns it as a float if found.
    ///
    /// # Arguments
    ///
    /// * `section` - A string slice representing the section name.
    /// * `key` - A string slice representing the key name.
    /// * `default` - An optional default value to return if the key is not found.
    ///
    /// # Returns
    ///
    /// An `Option<f64>` containing the float value if found, or the default value
    /// if provided, or `None` if the key is not found and no default was provided.
    pub fn get_float(&self, section: &str, key: &str, default: Option<f64>) -> Option<f64> {
        match self.get(section, key) {
            Some(value) => value.as_float().or(default),
            None => default,
        }
    }

    /// Retrieves a boolean value from the configuration.
    ///
    /// This method is a convenience wrapper that looks up a configuration value
    /// by section and key, and returns it as a boolean if found.
    ///
    /// # Arguments
    ///
    /// * `section` - A string slice representing the section name.
    /// * `key` - A string slice representing the key name.
    /// * `default` - An optional default value to return if the key is not found.
    ///
    /// # Returns
    ///
    /// An `Option<bool>` containing the boolean value if found, or the default value
    /// if provided, or `None` if the key is not found and no default was provided.
    pub fn get_boolean(&self, section: &str, key: &str, default: Option<bool>) -> Option<bool> {
        match self.get(section, key) {
            Some(value) => value.as_boolean().or(default),
            None => default,
        }
    }

    /// Retrieves an array value from the configuration.
    ///
    /// This method is a convenience wrapper that looks up a configuration value
    /// by section and key, and returns it as an array if found.
    ///
    /// # Arguments
    ///
    /// * `section` - A string slice representing the section name.
    /// * `key` - A string slice representing the key name.
    ///
    /// # Returns
    ///
    /// An `Option<&Vec<ConfigValue>>` containing the array value if found, or `None`
    /// if the key is not found or the value is not an array.
    pub fn get_array<'a>(&'a self, section: &str, key: &str) -> Option<&'a Vec<ConfigValue>> {
        self.get(section, key).and_then(|value| {
            if let ConfigValue::Array(arr) = value {
                Some(arr)
            } else {
                None
            }
        })
    }

    /// Retrieves a table value from the configuration.
    ///
    /// This method is a convenience wrapper that looks up a configuration value
    /// by section and key, and returns it as a table if found.
    ///
    /// # Arguments
    ///
    /// * `section` - A string slice representing the section name.
    /// * `key` - A string slice representing the key name.
    ///
    /// # Returns
    ///
    /// An `Option<&HashMap<String, ConfigValue>>` containing the table value if found,
    /// or `None` if the key is not found or the value is not a table.
    pub fn get_table<'a>(&'a self, section: &str, key: &str) -> Option<&'a HashMap<String, ConfigValue>> {
        self.get(section, key).and_then(|value| {
            if let ConfigValue::Table(table) = value {
                Some(table)
            } else {
                None
            }
        })
    }
}

// Add Default implementation for Config
impl Default for Config {
    /// Creates a new `Config` instance with default values.
    ///
    /// The default instance has an empty application name, no values, an unknown format,
    /// and no configuration file path.
    fn default() -> Self {
        Config {
            app_name: String::new(),
            values: HashMap::new(),
            format: ConfigFormat::Unknown,
            config_file_path: None,
        }
    }
}


// Esportiamo i moduli pubblici
pub use formats::ini;
pub use formats::toml;
pub use formats::yaml;
pub use formats::json;
pub use validation::*;