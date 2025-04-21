// examples/basic_usage.rs
//! Esempio di utilizzo base della libreria confucius

use std::env;
use std::fs;
use std::path::Path;
use confucius::{Config, ConfigValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Crea una configurazione per un'app chiamata "myapp"
    let mut config = Config::new("myapp");

    // Creiamo un file di configurazione di esempio
    create_example_config()?;

    // Carica la configurazione dai percorsi predefiniti
    match config.load() {
        Ok(_) => println!("Configurazione caricata con successo!"),
        Err(e) => {
            println!("Errore nel caricamento della configurazione: {}", e);
            println!("Utilizziamo un file specifico...");

            // Proviamo a caricare dal file di esempio che abbiamo creato
            config.load_from_file(Path::new("myapp.conf"))?;
        }
    }

    // Leggiamo alcuni valori
    if let Some(server) = config.get("server", "hostname") {
        if let Some(hostname) = server.as_string() {
            println!("Server hostname: {}", hostname);
        }
    }

    if let Some(port) = config.get("server", "port") {
        if let Some(port_num) = port.as_integer() {
            println!("Server port: {}", port_num);
        }
    }

    if let Some(debug) = config.get("app", "debug") {
        if let Some(debug_enabled) = debug.as_boolean() {
            println!("Debug mode: {}", if debug_enabled { "enabled" } else { "disabled" });
        }
    }

    // Modifichiamo alcuni valori
    config.set("app", "version", ConfigValue::String("1.0.1".to_string()));
    config.set("server", "timeout", ConfigValue::Integer(30));

    // Salviamo la configurazione
    config.save_to_file(Path::new("myapp_updated.conf"))?;
    println!("Configurazione salvata in myapp_updated.conf");

    Ok(())
}

/// Crea un file di configurazione di esempio
fn create_example_config() -> Result<(), Box<dyn std::error::Error>> {
    let config_content = r#"#!config/ini
# Questo è un file di configurazione di esempio per myapp

[app]
name = "My Application"
version = "1.0.0"
debug = true # Abilita il debug

[server]
hostname = "localhost"
port = 8080
max_connections = 100

[database]
url = "postgresql://user:password@localhost/mydb"
pool_size = 10

# Questo è un esempio di include
include=myapp_extra.conf
"#;

    let extra_config = r#"#!config/ini
# Configurazione extra per myapp

[logging]
level = "info"
file = "/var/log/myapp.log"

[security]
enable_ssl = true
cert_file = "/etc/ssl/certs/myapp.pem"
"#;

    // Scriviamo i file
    fs::write("myapp.conf", config_content)?;
    fs::write("myapp_extra.conf", extra_config)?;

    println!("File di configurazione di esempio creati.");

    Ok(())
}