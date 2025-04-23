// examples/validation_example.rs
//! Esempio di utilizzo del sistema di validazione

use confucius::{Config, ConfigValue, ConfigFormat};
use confucius::validation::{ValidationSchema, ValidationExt, FieldDefinition, FieldConstraint, ValueType};
use std::path::Path;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Creiamo un file di configurazione TOML per il test
    let config_content = r#"#!config/toml
[server]
hostname = "example.com"
port = 8080
max_connections = 500
ssl = true

[database]
host = "localhost"
port = 5432
name = "myapp_db"
# Manca user e password qui

[logging]
level = "debug"
file = "/var/log/myapp.log"
"#;

    // Scriviamo il file di configurazione
    let config_path = Path::new("test_config.toml");
    let mut file = File::create(&config_path)?;
    file.write_all(config_content.as_bytes())?;

    // Carica la configurazione
    let mut config = Config::new("myapp");
    config.load_from_file(&config_path)?;

    println!("Configurazione caricata con successo!");

    // Creiamo uno schema di validazione
    let mut schema = ValidationSchema::new();

    // Definiamo le sezioni richieste
    schema.required_section("server")
        .required_section("database")
        .required_section("logging")
        .allow_unknown_sections(false);  // Non permettiamo sezioni non definite

    // Definiamo i campi per la sezione "server"
    schema.field(
        "server",
        "hostname",
        FieldDefinition::new(ValueType::String)
            .required()
            .description("Nome host del server")
            .constraint(FieldConstraint::string()
                .min_length(3)
                .max_length(255))
    );

    schema.field(
        "server",
        "port",
        FieldDefinition::new(ValueType::Integer)
            .required()
            .description("Porta del server")
            .constraint(FieldConstraint::integer()
                .min_int(1)
                .max_int(65535))
    );

    schema.field(
        "server",
        "max_connections",
        FieldDefinition::new(ValueType::Integer)
            .description("Numero massimo di connessioni")
            .default(ConfigValue::Integer(100))
            .constraint(FieldConstraint::integer()
                .min_int(1)
                .max_int(10000))
    );

    schema.field(
        "server",
        "ssl",
        FieldDefinition::new(ValueType::Boolean)
            .description("Abilita SSL")
            .default(ConfigValue::Boolean(false))
    );

    // Definiamo i campi per la sezione "database"
    schema.field(
        "database",
        "host",
        FieldDefinition::new(ValueType::String)
            .required()
            .description("Host del database")
    );

    schema.field(
        "database",
        "port",
        FieldDefinition::new(ValueType::Integer)
            .required()
            .description("Porta del database")
            .constraint(FieldConstraint::integer()
                .min_int(1)
                .max_int(65535))
    );

    schema.field(
        "database",
        "name",
        FieldDefinition::new(ValueType::String)
            .required()
            .description("Nome del database")
    );

    schema.field(
        "database",
        "user",
        FieldDefinition::new(ValueType::String)
            .required()
            .description("Utente del database")
    );

    schema.field(
        "database",
        "password",
        FieldDefinition::new(ValueType::String)
            .required()
            .description("Password del database")
    );

    // Definiamo i campi per la sezione "logging"
    schema.field(
        "logging",
        "level",
        FieldDefinition::new(ValueType::String)
            .required()
            .description("Livello di logging")
            .constraint(FieldConstraint::string()
                .allowed_string_values(vec!["debug", "info", "warn", "error"]))
    );

    schema.field(
        "logging",
        "file",
        FieldDefinition::new(ValueType::String)
            .required()
            .description("File di log")
    );

    schema.field(
        "logging",
        "max_size",
        FieldDefinition::new(ValueType::Integer)
            .description("Dimensione massima del file di log in MB")
            .default(ConfigValue::Integer(10))
            .constraint(FieldConstraint::integer()
                .min_int(1)
                .max_int(1000))
    );

    // Validiamo la configurazione (ci aspettiamo degli errori)
    println!("\nValidazione iniziale (ci aspettiamo errori):");
    match config.validate(&schema) {
        Ok(_) => println!("La configurazione è valida (inaspettato)!"),
        Err(errors) => println!("Errori di validazione:\n{}", errors),
    }

    // Applichiamo i valori di default
    println!("\nApplichiamo i valori di default...");
    config.apply_defaults(&schema);

    // Mostriamo i valori modificati
    println!("\nValori dopo l'applicazione dei default:");
    if let Some(max_size) = config.get("logging", "max_size") {
        println!("logging.max_size = {:?}", max_size);
    }

    // Aggiungiamo manualmente i campi mancanti
    println!("\nAggiungiamo i campi mancanti...");
    config.set("database", "user", ConfigValue::String("db_user".to_string()));
    config.set("database", "password", ConfigValue::String("secure_password".to_string()));

    // Validiamo di nuovo
    println!("\nValidazione finale:");
    match config.validate(&schema) {
        Ok(_) => println!("La configurazione è valida!"),
        Err(errors) => println!("Errori di validazione:\n{}", errors),
    }

    // Salviamo la configurazione validata
    let validated_path = Path::new("validated_config.toml");
    config.save_to_file(&validated_path)?;
    println!("\nConfigurazione validata salvata in {}", validated_path.display());

    Ok(())
}