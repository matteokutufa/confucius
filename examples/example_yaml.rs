// Esempio 2: Creazione di una configurazione YAML
fn example_yaml() -> Result<(), Box<dyn std::error::Error>> {
    use confucius::{Config, ConfigValue, ConfigFormat};
    use std::path::Path;
    use std::collections::HashMap;

    // Creiamo una configurazione per un'app chiamata "webapp"
    let mut config = Config::new("webapp");

    // Impostiamo esplicitamente il formato YAML
    config.set_format(ConfigFormat::Yaml);

    // Aggiungiamo dei valori base
    config.set("app", "name", ConfigValue::String("My Web Application".to_string()));
    config.set("app", "version", ConfigValue::String("1.2.0".to_string()));
    config.set("app", "debug", ConfigValue::Boolean(false));

    // Configurazione più complessa: tabella annidata
    let mut db_config = HashMap::new();
    db_config.insert("host".to_string(), ConfigValue::String("localhost".to_string()));
    db_config.insert("port".to_string(), ConfigValue::Integer(5432));
    db_config.insert("user".to_string(), ConfigValue::String("webapp_user".to_string()));
    db_config.insert("password".to_string(), ConfigValue::String("secretpassword".to_string()));

    config.set("database", "main", ConfigValue::Table(db_config));

    // Array di oggetti per la configurazione degli endpoint
    let endpoints = vec![
        {
            let mut endpoint = HashMap::new();
            endpoint.insert("path".to_string(), ConfigValue::String("/api/users".to_string()));
            endpoint.insert("method".to_string(), ConfigValue::String("GET".to_string()));
            endpoint.insert("auth_required".to_string(), ConfigValue::Boolean(true));
            ConfigValue::Table(endpoint)
        },
        {
            let mut endpoint = HashMap::new();
            endpoint.insert("path".to_string(), ConfigValue::String("/api/login".to_string()));
            endpoint.insert("method".to_string(), ConfigValue::String("POST".to_string()));
            endpoint.insert("auth_required".to_string(), ConfigValue::Boolean(false));
            ConfigValue::Table(endpoint)
        }
    ];

    config.set("api", "endpoints", ConfigValue::Array(endpoints));

    // Salviamo la configurazione
    config.save_to_file(Path::new("webapp.yaml"))?;

    println!("Configurazione YAML salvata in webapp.yaml");

    // Ora ricarichiamo la configurazione
    let mut loaded_config = Config::new("webapp");
    loaded_config.load_from_file(Path::new("webapp.yaml"))?;

    // Verifichiamo i valori
    if let Some(app_name) = loaded_config.get("app", "name") {
        if let Some(name_str) = app_name.as_string() {
            println!("App name: {}", name_str);
        }
    }

    if let Some(debug) = loaded_config.get("app", "debug") {
        if let Some(debug_value) = debug.as_boolean() {
            println!("Debug mode: {}", debug_value);
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("\nEsempio YAML:");
    example_yaml()?;

    Ok(())
}