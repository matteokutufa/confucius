// examples/basic_usage.rs
//! Basic usage example for the confucius library

use std::env;
use std::fs;
use std::path::Path;
use confucius::{Config, ConfigValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a configuration for an app called "myapp"
    let mut config = Config::new("myapp");

    // Create an example configuration file
    create_example_config()?;

    // Load configuration from default paths
    match config.load() {
        Ok(_) => println!("Configuration loaded successfully!"),
        Err(e) => {
            println!("Error loading configuration: {}", e);
            println!("Trying to load from a specific file...");

            // Try to load from the example file we created
            config.load_from_file(Path::new("myapp.conf"))?;
        }
    }

    // Read some values
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

    // Modify some values
    config.set("app", "version", ConfigValue::String("1.0.1".to_string()));
    config.set("server", "timeout", ConfigValue::Integer(30));

    // Save the configuration
    config.save_to_file(Path::new("myapp_updated.conf"))?;
    println!("Configuration saved to myapp_updated.conf");

    Ok(())
}

/// Creates example configuration files
fn create_example_config() -> Result<(), Box<dyn std::error::Error>> {
    let config_content = r#"#!config/ini
# This is an example configuration file for myapp

[app]
name = "My Application"
version = "1.0.0"
debug = true # Enable debugging

[server]
hostname = "localhost"
port = 8080
max_connections = 100

[database]
url = "postgresql://user:password@localhost/mydb"
pool_size = 10

# This is an include example
include=myapp_extra.conf
"#;

    let extra_config = r#"#!config/ini
# Extra configuration for myapp

[logging]
level = "info"
file = "/var/log/myapp.log"

[security]
enable_ssl = true
cert_file = "/etc/ssl/certs/myapp.pem"
"#;

    // Write the files
    fs::write("myapp.conf", config_content)?;
    fs::write("myapp_extra.conf", extra_config)?;

    println!("Example configuration files created.");

    Ok(())
}