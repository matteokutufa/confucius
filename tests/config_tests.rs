//! Test di base per la libreria Conf-ucius
//! Questi test verificano le funzionalità principali

use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

use confucius::{Config, ConfigValue, ConfigError};

// Utility per creare file di configurazione temporanei
fn create_temp_config(content: &str, filename: &str) -> PathBuf {
    let dir = tempdir().expect("Impossibile creare directory temporanea");
    let file_path = dir.path().join(filename);
    fs::write(&file_path, content).expect("Impossibile scrivere file di configurazione temporaneo");
    file_path
}

#[test]
fn test_load_basic_ini() {
    let content = r#"#!config/ini
[section1]
key1 = value1
key2 = "quoted value"
key3 = 123
key4 = true

[section2]
key1 = 3.14
"#;

    let file_path = create_temp_config(content, "test_basic.conf");

    let mut config = Config::new("test");
    let result = config.load_from_file(&file_path);
    assert!(result.is_ok(), "Caricamento del file fallito: {:?}", result.err());

    // Verifichiamo i valori
    if let Some(value) = config.get("section1", "key1") {
        assert_eq!(value.as_string(), Some(&"value1".to_string()));
    } else {
        panic!("key1 non trovata");
    }

    if let Some(value) = config.get("section1", "key2") {
        assert_eq!(value.as_string(), Some(&"quoted value".to_string()));
    } else {
        panic!("key2 non trovata");
    }

    if let Some(value) = config.get("section1", "key3") {
        assert_eq!(value.as_integer(), Some(123));
    } else {
        panic!("key3 non trovata");
    }

    if let Some(value) = config.get("section1", "key4") {
        assert_eq!(value.as_boolean(), Some(true));
    } else {
        panic!("key4 non trovata");
    }

    if let Some(value) = config.get("section2", "key1") {
        assert_eq!(value.as_float(), Some(3.14));
    } else {
        panic!("key1 in section2 non trovata");
    }
}

#[test]
fn test_comments() {
    let content = r#"#!config/ini
# Commento all'inizio del file
[section]
key1 = value1 # Commento in linea
key2 = "value with # inside quotes" # Commento dopo un valore con # all'interno
key3 = 123 # Commento dopo un numero
"#;

    let file_path = create_temp_config(content, "test_comments.conf");

    let mut config = Config::new("test");
    let result = config.load_from_file(&file_path);
    assert!(result.is_ok(), "Caricamento del file fallito: {:?}", result.err());

    // Verifichiamo i valori (i commenti dovrebbero essere rimossi)
    if let Some(value) = config.get("section", "key1") {
        assert_eq!(value.as_string(), Some(&"value1".to_string()));
    } else {
        panic!("key1 non trovata");
    }

    if let Some(value) = config.get("section", "key2") {
        assert_eq!(value.as_string(), Some(&"value with # inside quotes".to_string()));
    } else {
        panic!("key2 non trovata");
    }

    if let Some(value) = config.get("section", "key3") {
        assert_eq!(value.as_integer(), Some(123));
    } else {
        panic!("key3 non trovata");
    }
}

#[test]
fn test_include_single_file() {
    // File principale
    let main_content = r#"#!config/ini
[main]
key1 = "main value"
include=included.conf
"#;

    // File incluso
    let included_content = r#"#!config/ini
[included]
key2 = "included value"
"#;

    // Creiamo i file temporanei
    let temp_dir = tempdir().expect("Impossibile creare directory temporanea");
    let main_path = temp_dir.path().join("main.conf");
    let included_path = temp_dir.path().join("included.conf");

    fs::write(&main_path, main_content).expect("Impossibile scrivere file main");
    fs::write(&included_path, included_content).expect("Impossibile scrivere file included");

    // Carichiamo la configurazione
    let mut config = Config::new("test");
    let result = config.load_from_file(&main_path);
    assert!(result.is_ok(), "Caricamento del file fallito: {:?}", result.err());

    // Verifichiamo che entrambi i valori siano stati caricati
    if let Some(value) = config.get("main", "key1") {
        assert_eq!(value.as_string(), Some(&"main value".to_string()));
    } else {
        panic!("key1 non trovata");
    }

    if let Some(value) = config.get("included", "key2") {
        assert_eq!(value.as_string(), Some(&"included value".to_string()));
    } else {
        panic!("key2 non trovata dal file incluso");
    }
}

#[test]
fn test_include_glob_pattern() {
    // File principale
    let main_content = r#"#!config/ini
[main]
key1 = "main value"
include=conf.d/*.conf
"#;

    // File inclusi
    let included1_content = r#"#!config/ini
[included1]
key2 = "included1 value"
"#;

    let included2_content = r#"#!config/ini
[included2]
key3 = "included2 value"
"#;

    // Creiamo i file temporanei
    let temp_dir = tempdir().expect("Impossibile creare directory temporanea");
    let main_path = temp_dir.path().join("main.conf");

    // Creiamo la directory conf.d
    let conf_d_path = temp_dir.path().join("conf.d");
    fs::create_dir(&conf_d_path).expect("Impossibile creare directory conf.d");

    let included1_path = conf_d_path.join("file1.conf");
    let included2_path = conf_d_path.join("file2.conf");

    fs::write(&main_path, main_content).expect("Impossibile scrivere file main");
    fs::write(&included1_path, included1_content).expect("Impossibile scrivere file included1");
    fs::write(&included2_path, included2_content).expect("Impossibile scrivere file included2");

    // Carichiamo la configurazione
    let mut config = Config::new("test");
    let result = config.load_from_file(&main_path);

    // Nota: questa potrebbe fallire se la libreria glob non è configurata correttamente
    // In tal caso, potrebbe essere necessario modificare il test o la libreria
    assert!(result.is_ok(), "Caricamento del file fallito: {:?}", result.err());

    // Verifichiamo che tutti i valori siano stati caricati
    if let Some(value) = config.get("main", "key1") {
        assert_eq!(value.as_string(), Some(&"main value".to_string()));
    } else {
        panic!("key1 non trovata");
    }

    if let Some(value) = config.get("included1", "key2") {
        assert_eq!(value.as_string(), Some(&"included1 value".to_string()));
    } else {
        panic!("key2 non trovata dal file incluso1");
    }

    if let Some(value) = config.get("included2", "key3") {
        assert_eq!(value.as_string(), Some(&"included2 value".to_string()));
    } else {
        panic!("key3 non trovata dal file incluso2");
    }
}

#[test]
fn test_save_config() {
    // Creiamo una configurazione da zero
    let mut config = Config::new("test");

    // Aggiungiamo alcuni valori
    config.set("section1", "key1", ConfigValue::String("value1".to_string()));
    config.set("section1", "key2", ConfigValue::Integer(123));
    config.set("section2", "key3", ConfigValue::Boolean(true));
    config.set("section2", "key4", ConfigValue::Float(3.14));

    // File temporaneo per il salvataggio
    let temp_dir = tempdir().expect("Impossibile creare directory temporanea");
    let save_path = temp_dir.path().join("saved.conf");

    // Salviamo la configurazione
    let result = config.save_to_file(&save_path);
    assert!(result.is_ok(), "Salvataggio del file fallito: {:?}", result.err());

    // Verifichiamo che il file esista
    assert!(save_path.exists(), "Il file salvato non esiste");

    // Leggiamo il contenuto del file
    let content = fs::read_to_string(&save_path).expect("Impossibile leggere il file salvato");

    // Verifichiamo che il contenuto sia corretto
    assert!(content.contains("#!config/ini"), "Manca l'intestazione del formato");
    assert!(content.contains("[section1]"), "Manca la sezione1");
    assert!(content.contains("[section2]"), "Manca la sezione2");
    assert!(content.contains("key1 = \"value1\""), "Manca key1");
    assert!(content.contains("key2 = 123"), "Manca key2");
    assert!(content.contains("key3 = true"), "Manca key3");
    assert!(content.contains("key4 = 3.14"), "Manca key4");

    // Carichiamo la configurazione dal file salvato
    let mut loaded_config = Config::new("test");
    let load_result = loaded_config.load_from_file(&save_path);
    assert!(load_result.is_ok(), "Caricamento del file salvato fallito: {:?}", load_result.err());

    // Verifichiamo che i valori caricati siano corretti
    if let Some(value) = loaded_config.get("section1", "key1") {
        assert_eq!(value.as_string(), Some(&"value1".to_string()));
    } else {
        panic!("key1 non trovata nel file salvato");
    }

    if let Some(value) = loaded_config.get("section1", "key2") {
        assert_eq!(value.as_integer(), Some(123));
    } else {
        panic!("key2 non trovata nel file salvato");
    }

    if let Some(value) = loaded_config.get("section2", "key3") {
        assert_eq!(value.as_boolean(), Some(true));
    } else {
        panic!("key3 non trovata nel file salvato");
    }

    if let Some(value) = loaded_config.get("section2", "key4") {
        assert_eq!(value.as_float(), Some(3.14));
    } else {
        panic!("key4 non trovata nel file salvato");
    }
}

#[test]
fn test_detect_format() {
    let ini_content = r#"#!config/ini
[section]
key = value
"#;

    let toml_content = r#"#!config/toml
# This is a TOML document
key = "value"
"#;

    let yaml_content = r#"#!config/yaml
# This is a YAML document
key: value
"#;

    let json_content = r#"#!config/json
{
  "key": "value"
}
"#;

    // Creiamo i file temporanei
    let ini_path = create_temp_config(ini_content, "test_ini.conf");
    let toml_path = create_temp_config(toml_content, "test_toml.conf");
    let yaml_path = create_temp_config(yaml_content, "test_yaml.conf");
    let json_path = create_temp_config(json_content, "test_json.conf");

    // Testiamo il rilevamento del formato INI
    let mut config = Config::new("test");
    let result = config.load_from_file(&ini_path);
    assert!(result.is_ok(), "Caricamento del file INI fallito");

    // I formati non supportati dovrebbero dare errore UnsupportedFormat
    let mut config = Config::new("test");
    let result = config.load_from_file(&toml_path);
    assert!(matches!(result, Err(ConfigError::UnsupportedFormat(_))),
            "Dovrebbe dare errore UnsupportedFormat per TOML");

    let mut config = Config::new("test");
    let result = config.load_from_file(&yaml_path);
    assert!(matches!(result, Err(ConfigError::UnsupportedFormat(_))),
            "Dovrebbe dare errore UnsupportedFormat per YAML");

    let mut config = Config::new("test");
    let result = config.load_from_file(&json_path);
    assert!(matches!(result, Err(ConfigError::UnsupportedFormat(_))),
            "Dovrebbe dare errore UnsupportedFormat per JSON");
}

#[test]
fn test_quoted_values() {
    let content = r#"#!config/ini
[section]
key1 = "value with spaces"
key2 = "value with # symbol inside quotes"
key3 = "value with \"escaped quotes\" inside"
key4 = "123" # Questo è una stringa, non un numero
"#;

    let file_path = create_temp_config(content, "test_quoted.conf");

    let mut config = Config::new("test");
    let result = config.load_from_file(&file_path);
    assert!(result.is_ok(), "Caricamento del file fallito: {:?}", result.err());

    // Verifichiamo i valori
    if let Some(value) = config.get("section", "key1") {
        assert_eq!(value.as_string(), Some(&"value with spaces".to_string()));
    } else {
        panic!("key1 non trovata");
    }

    if let Some(value) = config.get("section", "key2") {
        assert_eq!(value.as_string(), Some(&"value with # symbol inside quotes".to_string()));
    } else {
        panic!("key2 non trovata");
    }

    if let Some(value) = config.get("section", "key3") {
        assert_eq!(value.as_string(), Some(&"value with \"escaped quotes\" inside".to_string()));
    } else {
        panic!("key3 non trovata");
    }

    if let Some(value) = config.get("section", "key4") {
        // Questo dovrebbe essere una stringa, non un numero
        assert_eq!(value.as_string(), Some(&"123".to_string()));
        assert_eq!(value.as_integer(), None);
    } else {
        panic!("key4 non trovata");
    }
}

#[test]
fn test_config_value_conversions() {
    // Testiamo le conversioni tra tipi in ConfigValue

    // Stringa
    let string_value = ConfigValue::String("test".to_string());
    assert_eq!(string_value.as_string(), Some(&"test".to_string()));
    assert_eq!(string_value.as_integer(), None);
    assert_eq!(string_value.as_float(), None);
    assert_eq!(string_value.as_boolean(), None);

    // Intero
    let int_value = ConfigValue::Integer(42);
    assert_eq!(int_value.as_string(), None);
    assert_eq!(int_value.as_integer(), Some(42));
    assert_eq!(int_value.as_float(), Some(42.0));
    assert_eq!(int_value.as_boolean(), None);

    // Float
    let float_value = ConfigValue::Float(3.14);
    assert_eq!(float_value.as_string(), None);
    assert_eq!(float_value.as_integer(), None);
    assert_eq!(float_value.as_float(), Some(3.14));
    assert_eq!(float_value.as_boolean(), None);

    // Booleano
    let bool_value = ConfigValue::Boolean(true);
    assert_eq!(bool_value.as_string(), None);
    assert_eq!(bool_value.as_integer(), None);
    assert_eq!(bool_value.as_float(), None);
    assert_eq!(bool_value.as_boolean(), Some(true));
}