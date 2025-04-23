// Esempio di utilizzo con il formato JSON
fn example_json() -> Result<(), Box<dyn std::error::Error>> {
    use confucius::{Config, ConfigValue, ConfigFormat};
    use std::path::Path;
    use std::collections::HashMap;

    // Creiamo una configurazione per un'app chiamata "api_server"
    let mut config = Config::new("api_server");

    // Impostiamo esplicitamente il formato JSON
    config.set_format(ConfigFormat::Json);

    // Aggiungiamo la configurazione di base
    config.set("api", "version", ConfigValue::String("2.0.0".to_string()));
    config.set("api", "base_url", ConfigValue::String("/api/v2".to_string()));
    config.set("api", "enable_cors", ConfigValue::Boolean(true));

    // Configurazione più complessa: array di endpoint
    let mut endpoints = Vec::new();

    // Endpoint 1: /users
    let mut endpoint1 = HashMap::new();
    endpoint1.insert("path".to_string(), ConfigValue::String("/users".to_string()));
    endpoint1.insert("method".to_string(), ConfigValue::String("GET".to_string()));
    endpoint1.insert("auth_required".to_string(), ConfigValue::Boolean(true));

    // Parametri dell'endpoint
    let mut params1 = HashMap::new();
    params1.insert("limit".to_string(), ConfigValue::Integer(100));
    params1.insert("offset".to_string(), ConfigValue::Integer(0));
    endpoint1.insert("default_params".to_string(), ConfigValue::Table(params1));

    endpoints.push(ConfigValue::Table(endpoint1));

    // Endpoint 2: /auth
    let mut endpoint2 = HashMap::new();
    endpoint2.insert("path".to_string(), ConfigValue::String("/auth".to_string()));
    endpoint2.insert("method".to_string(), ConfigValue::String("POST".to_string()));
    endpoint2.insert("auth_required".to_string(), ConfigValue::Boolean(false));

    // Rate limiting
    let mut rate_limit = HashMap::new();
    rate_limit.insert("requests".to_string(), ConfigValue::Integer(10));
    rate_limit.insert("period".to_string(), ConfigValue::String("1m".to_string()));
    endpoint2.insert("rate_limit".to_string(), ConfigValue::Table(rate_limit));

    endpoints.push(ConfigValue::Table(endpoint2));

    // Aggiungiamo gli endpoint alla configurazione
    config.set("api", "endpoints", ConfigValue::Array(endpoints));

    // Configurazione del database
    config.set("database", "host", ConfigValue::String("localhost".to_string()));
    config.set("database", "port", ConfigValue::Integer(5432));
    config.set("database", "name", ConfigValue::String("api_db".to_string()));
    config.set("database", "user", ConfigValue::String("api_user".to_string()));
    config.set("database", "password", ConfigValue::String("secret123".to_string()));
    config.set("database", "max_connections", ConfigValue::Integer(100));

    // Configurazione di logging
    config.set("logging", "level", ConfigValue::String("info".to_string()));
    config.set("logging", "file", ConfigValue::String("/var/log/api_server.log".to_string()));
    config.set("logging", "stdout", ConfigValue::Boolean(true));

    // Array di livelli di logging
    let log_levels = vec![
        ConfigValue::String("info".to_string()),
        ConfigValue::String("warn".to_string()),
        ConfigValue::String("error".to_string()),
    ];
    config.set("logging", "enabled_levels", ConfigValue::Array(log_levels));

    // Salviamo la configurazione
    config.save_to_file(Path::new("api_server.json"))?;

    println!("Configurazione JSON salvata in api_server.json");

    // Ora ricarichiamo la configurazione
    let mut loaded_config = Config::new("api_server");
    loaded_config.load_from_file(Path::new("api_server.json"))?;

    // Verifichiamo qualche valore
    if let Some(base_url) = loaded_config.get("api", "base_url") {
        if let Some(url) = base_url.as_string() {
            println!("API Base URL: {}", url);
        }
    }

    if let Some(db_port) = loaded_config.get("database", "port") {
        if let Some(port) = db_port.as_integer() {
            println!("Database port: {}", port);
        }
    }

    // Verifichiamo se abbiamo potuto leggere l'array di endpoint
    if let Some(endpoints) = loaded_config.get("api", "endpoints") {
        if let ConfigValue::Array(endpoints_arr) = endpoints {
            println!("Trovati {} endpoint", endpoints_arr.len());

            // Estraiamo informazioni dal primo endpoint
            if let Some(ConfigValue::Table(first_endpoint)) = endpoints_arr.first() {
                if let Some(ConfigValue::String(path)) = first_endpoint.get("path") {
                    println!("Primo endpoint: {}", path);
                }
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Esempio di utilizzo del formato JSON:");
    example_json()?;
    Ok(())
}