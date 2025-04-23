fn example_with_includes() -> Result<(), Box<dyn std::error::Error>> {
    use confucius::{Config, ConfigValue, ConfigFormat};
    use std::path::Path;
    use std::fs::{self, File};
    use std::io::Write;

    // Creiamo prima un file di configurazione principale
    let mut main_config = Config::new("app_with_includes");
    main_config.set_format(ConfigFormat::Toml);

    main_config.set("app", "name", ConfigValue::String("Application with includes".to_string()));
    main_config.set("app", "version", ConfigValue::String("1.0.0".to_string()));

    // Creiamo una directory per le configurazioni
    fs::create_dir_all("conf.d")?;

    // Creiamo un file INI per le impostazioni del server
    let mut server_file = File::create("conf.d/server.ini")?;
    writeln!(server_file, "#!config/ini")?;
    writeln!(server_file, "[server]")?;
    writeln!(server_file, "hostname = \"app.example.com\"")?;
    writeln!(server_file, "port = 8080")?;
    writeln!(server_file, "ssl = true")?;

    // Creiamo un file YAML per le impostazioni di logging
    let mut logging_file = File::create("conf.d/logging.yaml")?;
    writeln!(logging_file, "#!config/yaml")?;
    writeln!(logging_file, "logging:")?;
    writeln!(logging_file, "  level: info")?;
    writeln!(logging_file, "  file: /var/log/myapp.log")?;
    writeln!(logging_file, "  rotate: true")?;
    writeln!(logging_file, "  max_size: 10485760")?;

    // Aggiungiamo l'inclusione di questi file alla configurazione principale
    let mut file = File::create("app_config.toml")?;
    writeln!(file, "#!config/toml")?;
    writeln!(file, "app = {{ name = \"Application with includes\", version = \"1.0.0\" }}")?;
    writeln!(file, "include = [\"conf.d/server.ini\", \"conf.d/logging.yaml\"]")?;

    // Ora carichiamo la configurazione completa
    let mut config = Config::new("app_with_includes");
    config.load_from_file(Path::new("app_config.toml"))?;

    // Verifichiamo che i valori dai file inclusi siano stati caricati
    if let Some(hostname) = config.get("server", "hostname") {
        if let Some(hostname_str) = hostname.as_string() {
            println!("Server hostname: {}", hostname_str);
        }
    }

    if let Some(log_level) = config.get("logging", "level") {
        if let Some(level_str) = log_level.as_string() {
            println!("Log level: {}", level_str);
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("\nEsempio con inclusioni:");
    example_with_includes()?;

    Ok(())
}