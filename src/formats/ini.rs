// src/formats/ini.rs
//! Implementazione del parser e writer per il formato INI

use std::fs::{self, File};
use std::io::{BufRead, Write};
use std::path::Path;
use regex::Regex;

use crate::{Config, ConfigError, ConfigValue};
use crate::include;
use crate::utils;

/// Parser per file INI
pub fn parse_ini(config: &mut Config, content: &str, path: &Path) -> Result<(), ConfigError> {
    let mut current_section = "default".to_string();
    let section_regex = Regex::new(r"^\s*\[(.*?)\]\s*$").unwrap();
    let kv_regex = Regex::new(r"^\s*(.*?)\s*=\s*(.*?)\s*$").unwrap();
    let include_regex = Regex::new(r"^\s*include\s*=\s*(.*?)\s*$").unwrap();

    // Saltare la prima riga se contiene il formato (#!config/...)
    let lines_to_process = if content.lines().next().unwrap_or("").starts_with("#!config/") {
        content.lines().skip(1).collect::<Vec<_>>()
    } else {
        content.lines().collect::<Vec<_>>()
    };

    for line in lines_to_process {
        // Rimuoviamo i commenti dalla riga
        let line = utils::strip_comments(line);
        if line.is_empty() {
            continue;
        }

        // Controlliamo se è una direttiva di inclusione
        if let Some(cap) = include_regex.captures(&line) {
            let include_path = cap.get(1).unwrap().as_str();
            process_include(config, include_path, path)?;
            continue;
        }

        // Controlliamo se è una sezione
        if let Some(cap) = section_regex.captures(&line) {
            current_section = cap.get(1).unwrap().as_str().to_string();
            continue;
        }

        // Altrimenti è una coppia chiave-valore
        if let Some(cap) = kv_regex.captures(&line) {
            let key = cap.get(1).unwrap().as_str();
            let value_str = cap.get(2).unwrap().as_str();

            // Convertiamo il valore nel tipo appropriato
            let value = parse_value(value_str);

            // Inseriamo nella configurazione
            config.set(&current_section, key, value);
        }
    }

    Ok(())
}

/// Processa una direttiva di inclusione
fn process_include(config: &mut Config, include_path: &str, base_path: &Path) -> Result<(), ConfigError> {
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

            parse_ini(config, &content, &resolved_path)?;
        } else {
            return Err(ConfigError::IncludeError(format!("File incluso non trovato: {}",
                                                         resolved_path.display())));
        }
    }

    Ok(())
}

/// Converte una stringa in un ConfigValue
fn parse_value(value_str: &str) -> ConfigValue {
    // Se è tra virgolette, è una stringa
    if utils::is_quoted(value_str) {
        return ConfigValue::String(utils::unquote(value_str));
    }

    // Proviamo a convertire in booleano
    match value_str.to_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => return ConfigValue::Boolean(true),
        "false" | "no" | "off" | "0" => return ConfigValue::Boolean(false),
        _ => {}
    }

    // Proviamo a convertire in intero
    if let Ok(i) = value_str.parse::<i64>() {
        return ConfigValue::Integer(i);
    }

    // Proviamo a convertire in float
    if let Ok(f) = value_str.parse::<f64>() {
        return ConfigValue::Float(f);
    }

    // Altrimenti è una stringa
    ConfigValue::String(value_str.to_string())
}

/// Scrive la configurazione in formato INI
pub fn write_ini(config: &Config, path: &Path) -> Result<(), ConfigError> {
    let mut file = File::create(path).map_err(ConfigError::Io)?;

    // Scriviamo l'intestazione del formato
    writeln!(file, "#!config/ini").map_err(ConfigError::Io)?;

    // Per ogni sezione
    for (section, values) in &config.values {
        // Non scriviamo la sezione default se è vuota
        if section == "default" && values.is_empty() {
            continue;
        }

        // Scriviamo l'intestazione della sezione
        writeln!(file, "\n[{}]", section).map_err(ConfigError::Io)?;

        // Scriviamo ogni coppia chiave-valore
        for (key, value) in values {
            let value_str = format_value(value);
            writeln!(file, "{} = {}", key, value_str).map_err(ConfigError::Io)?;
        }
    }

    Ok(())
}

/// Formatta un ConfigValue come stringa
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
            // Nel formato INI non supportiamo array, quindi facciamo il join come stringa
            let items: Vec<String> = a.iter().map(format_value).collect();
            format!("\"{}\"", items.join(", "))
        },
        ConfigValue::Table(t) => {
            // Nel formato INI non supportiamo tabelle nidificate, quindi convertiamo in stringa
            format!("\"{}\"", format!("{:?}", t))
        },
    }
}