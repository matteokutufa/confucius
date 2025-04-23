// src/lib.rs
//! # Confucius
//!
//! > "La costanza è la virtù per cui tutte le altre virtù danno frutto." - Confucio
//!
//! Così come Confucio ha fornito saggezza per la vita, Confucius fornisce
//! saggezza per la configurazione delle tue applicazioni.
//!
//! `confucius` è una libreria per la gestione di file di configurazione con supporto per:
//! - Ricerca automatica di file di configurazione in percorsi standard
//! - Supporto per diversi formati (.ini, .toml, .yaml, .json)
//! - Meccanismo di include per file multipli
//! - Identificazione del formato tramite shebang (#!config/FORMAT)
//! - Supporto per commenti e valori testuali

use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use thiserror::Error;

mod validation;
mod parser;
mod formats;
mod include;
mod utils;


/// Tipi di formato supportati per i file di configurazione
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Ini,
    Toml,
    Yaml,
    Json,
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

/// Errori che possono verificarsi durante la gestione della configurazione
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Errore di I/O: {0}")]
    Io(#[from] io::Error),

    #[error("Formato di configurazione non supportato: {0}")]
    UnsupportedFormat(String),

    #[error("Errore di parsing: {0}")]
    ParseError(String),

    #[error("File di configurazione non trovato per: {0}")]
    ConfigNotFound(String),

    #[error("Errore nell'include: {0}")]
    IncludeError(String),

    #[error("Errore generico: {0}")]
    Generic(String),
}

/// Tipo per rappresentare un valore di configurazione
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
    /// Converte il valore in una stringa
    pub fn as_string(&self) -> Option<&String> {
        if let ConfigValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Converte il valore in un intero
    pub fn as_integer(&self) -> Option<i64> {
        if let ConfigValue::Integer(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    /// Converte il valore in un float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(f) => Some(*f),
            ConfigValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Converte il valore in un booleano
    pub fn as_boolean(&self) -> Option<bool> {
        if let ConfigValue::Boolean(b) = self {
            Some(*b)
        } else {
            None
        }
    }
}

/// Struttura principale per la gestione della configurazione
#[derive(Debug)]
pub struct Config {
    /// Nome dell'applicazione (es. "galatea")
    app_name: String,

    /// Valori di configurazione organizzati per sezione e chiave
    values: HashMap<String, HashMap<String, ConfigValue>>,

    /// Formato del file di configurazione
    format: ConfigFormat,

    /// Percorso del file di configurazione caricato
    config_file_path: Option<PathBuf>,
}

impl Config {
    /// Crea una nuova istanza di Config
    pub fn new(app_name: &str) -> Self {
        Config {
            app_name: app_name.to_string(),
            values: HashMap::new(),
            format: ConfigFormat::Unknown,
            config_file_path: None,
        }
    }

    /// Imposta esplicitamente il formato della configurazione
    pub fn set_format(&mut self, format: ConfigFormat) -> &mut Self {
        self.format = format;
        self
    }

    /// Ottiene il formato corrente della configurazione
    pub fn get_format(&self) -> ConfigFormat {
        self.format
    }


    /// Carica la configurazione dai percorsi predefiniti
    pub fn load(&mut self) -> Result<(), ConfigError> {
        // Recuperiamo il percorso di esecuzione e il nome utente corrente
        let exec_path = env::current_exe().map_err(ConfigError::Io)?;
        let username = utils::get_current_username()?;

        // Costruiamo i percorsi di ricerca per il file di configurazione
        let search_paths = self.build_search_paths(&exec_path, &username);

        // Cerchiamo il primo file di configurazione disponibile
        for path in search_paths {
            if path.exists() {
                return self.load_from_file(&path);
            }
        }

        Err(ConfigError::ConfigNotFound(self.app_name.clone()))
    }

    /// Carica la configurazione da un file specifico
    pub fn load_from_file(&mut self, path: &Path) -> Result<(), ConfigError> {
        let content = fs::read_to_string(path).map_err(ConfigError::Io)?;
        self.config_file_path = Some(path.to_path_buf());

        // Determiniamo il formato dal contenuto
        self.detect_format_from_content(&content)?;

        // Parserizziamo il contenuto in base al formato
        match self.format {
            ConfigFormat::Ini => formats::ini::parse_ini(self, &content, path)?,
            ConfigFormat::Toml => formats::toml::parse_toml(self, &content, path)?,
            ConfigFormat::Yaml => formats::yaml::parse_yaml(self, &content, path)?,
            ConfigFormat::Json => formats::json::parse_json(self, &content, path)?,
            ConfigFormat::Unknown => return Err(ConfigError::UnsupportedFormat("Sconosciuto".to_string())),
        }

        Ok(())
    }

    /// Costruisce i percorsi di ricerca per il file di configurazione
    fn build_search_paths(&self, exec_path: &Path, username: &str) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let config_filename = format!("{}.conf", self.app_name);

        // /etc/appname/appname.conf
        paths.push(PathBuf::from(format!("/etc/{}/{}", self.app_name, config_filename)));

        // /etc/appname.conf
        paths.push(PathBuf::from(format!("/etc/{}", config_filename)));

        // /opt/etc/appname.conf
        paths.push(PathBuf::from(format!("/opt/etc/{}", config_filename)));

        // /home/username/.config/appname/appname.conf
        paths.push(PathBuf::from(format!("/home/{}/.config/{}/{}",
                                         username, self.app_name, config_filename)));

        // /home/username/.config/appname.conf
        paths.push(PathBuf::from(format!("/home/{}/.config/{}",
                                         username, config_filename)));

        // Percorso di esecuzione
        if let Some(exec_dir) = exec_path.parent() {
            paths.push(exec_dir.join(&config_filename));
        }

        paths
    }

    /// Rileva il formato dal contenuto del file
    fn detect_format_from_content(&mut self, content: &str) -> Result<(), ConfigError> {
        // Leggiamo la prima riga per il formato
        let first_line = content.lines().next().unwrap_or("");

        // Se la prima riga è nel formato #!config/FORMAT
        if first_line.starts_with("#!config/") {
            let format_str = first_line.trim_start_matches("#!config/").trim();
            self.format = ConfigFormat::from(format_str);

            if self.format == ConfigFormat::Unknown {
                return Err(ConfigError::UnsupportedFormat(format_str.to_string()));
            }
        } else {
            // Per ora, assumiamo INI se non specificato
            self.format = ConfigFormat::Ini;
        }

        Ok(())
    }

    /// Ottiene un valore dalla configurazione
    pub fn get(&self, section: &str, key: &str) -> Option<&ConfigValue> {
        self.values.get(section).and_then(|section_map| section_map.get(key))
    }

    /// Imposta un valore nella configurazione
    pub fn set(&mut self, section: &str, key: &str, value: ConfigValue) -> &mut Self {
        self.values
            .entry(section.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), value);
        self
    }

    /// Salva la configurazione nel file corrente
    pub fn save(&self) -> Result<(), ConfigError> {
        if let Some(path) = &self.config_file_path {
            self.save_to_file(path)
        } else {
            Err(ConfigError::Generic("Nessun file di configurazione caricato".to_string()))
        }
    }

    /// Salva la configurazione in un file specifico
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
}

// Esportiamo i moduli pubblici
pub use formats::ini;
pub use formats::toml;
pub use formats::yaml;
pub use formats::json;
pub use validation::*;