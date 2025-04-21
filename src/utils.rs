// src/utils.rs
//! Funzioni di utilità per la libreria

use std::env;
use std::path::{Path, PathBuf};
use crate::ConfigError;
use path_clean::PathClean;

/// Ottiene il nome dell'utente corrente
pub fn get_current_username() -> Result<String, ConfigError> {
    if let Some(home_dir) = home::home_dir() {
        if let Some(home_dir_str) = home_dir.to_str() {
            return Ok(home_dir_str.to_string());
        }
    }

    // Fallback: proviamo a ottenerlo dalla variabile d'ambiente
    if let Ok(user) = env::var("USER") {
        return Ok(user);
    }

    // Fallback per Windows
    if let Ok(user) = env::var("USERNAME") {
        return Ok(user);
    }

    Err(ConfigError::Generic("Impossibile determinare il nome utente".to_string()))
}

/// Risolve un percorso relativo rispetto a un file base
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

    // Normalizza il percorso (rimuove ../, ./, etc.)
    path.clean()
}

/// Controlla se una stringa è racchiusa tra virgolette doppie
pub fn is_quoted(s: &str) -> bool {
    s.starts_with('"') && s.ends_with('"')
}

/// Rimuove le virgolette doppie da una stringa
pub fn unquote(s: &str) -> String {
    if is_quoted(s) {
        // Prendiamo la stringa senza le virgolette iniziali e finali
        let content = &s[1..s.len()-1];

        // Gestiamo gli escape sostituendo \" con "
        let mut result = String::with_capacity(content.len());
        let mut chars = content.chars().peekable();
        let mut in_escape = false;

        while let Some(c) = chars.next() {
            if in_escape {
                // Se siamo in un escape, includiamo il carattere così com'è
                result.push(c);
                in_escape = false;
            } else if c == '\\' && chars.peek() == Some(&'"') {
                // Se troviamo un backslash seguito da virgolette, lo trattiamo come escape
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

/// Elimina i commenti da una riga
///
/// I commenti sono tutto ciò che segue un # non all'interno di virgolette doppie
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
                break; // Commento trovato, interrompiamo
            },
            _ => {
                result.push(c);
            }
        }
    }

    result.trim_end().to_string()
}