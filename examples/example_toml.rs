// Esempio 1: Creazione di una configurazione TOML
fn example_toml() -> Result<(), Box<dyn std::error::Error>> {
    use confucius::{Config, ConfigValue, ConfigFormat};
    use std::path::Path;

    // Creiamo una configurazione per un'app chiamata "myapp"
    let mut config = Config::new("myapp");

    // Impostiamo esplicitamente il formato TOML
    config.set_format(ConfigFormat::Toml);

    // Aggiungiamo dei valori
    config.set("server", "hostname", ConfigValue::String("localhost".to_string()));
    config.set("server", "port", ConfigValue::Integer(8080));
    config.set("auth", "enabled", ConfigValue::Boolean(true));

    // Array di stringhe
    let users = vec![
        ConfigValue::String("admin".to_string()),
        ConfigValue::String("user1".to_string()),
        ConfigValue::String("user2".to_string()),
    ];
    config.set("auth", "allowed_users", ConfigValue::Array(users));

    // Salviamo la configurazione
    config.save_to_file(Path::new("myapp.toml"))?;

    println!("Configurazione TOML salvata in myapp.toml");

    // Ora ricarichiamo la configurazione
    let mut loaded_config = Config::new("myapp");
    loaded_config.load_from_file(Path::new("myapp.toml"))?;

    // Verifichiamo i valori
    if let Some(hostname) = loaded_config.get("server", "hostname") {
        if let Some(hostname_str) = hostname.as_string() {
            println!("Server hostname: {}", hostname_str);
        }
    }

    if let Some(port) = loaded_config.get("server", "port") {
        if let Some(port_value) = port.as_integer() {
            println!("Server port: {}", port_value);
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Esempio TOML:");
    example_toml()?;

    Ok(())
}