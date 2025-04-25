// examples/includes_example.rs
//! Example of using configuration file includes with the confucius library

use confucius::{Config, ConfigValue, ConfigFormat};
use std::path::Path;
use std::fs::{self, File};
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Configuration with Includes Example ===\n");

    // Create directory structure for our example
    setup_directories()?;

    // Create all necessary configuration files
    create_config_files()?;

    // Load the main configuration which includes other files
    let mut config = Config::new("app_with_includes");
    config.load_from_file(Path::new("app_config.toml"))?;

    println!("Configuration loaded successfully!");

    // Verify that values from included files were loaded
    if let Some(hostname) = config.get("server", "hostname") {
        if let Some(hostname_str) = hostname.as_string() {
            println!("\nServer hostname: {}", hostname_str);
        }
    }

    if let Some(log_level) = config.get("logging", "level") {
        if let Some(level_str) = log_level.as_string() {
            println!("Log level: {}", level_str);
        }
    }

    // Display database configuration loaded from the includes
    println!("\nDatabase configuration:");
    if let Some(db_host) = config.get("database", "host") {
        if let Some(host_str) = db_host.as_string() {
            println!("  Host: {}", host_str);
        }
    }

    if let Some(db_port) = config.get("database", "port") {
        if let Some(port_num) = db_port.as_integer() {
            println!("  Port: {}", port_num);
        }
    }

    if let Some(db_name) = config.get("database", "name") {
        if let Some(name_str) = db_name.as_string() {
            println!("  Name: {}", name_str);
        }
    }

    // Display security settings loaded from the includes
    println!("\nSecurity configuration:");
    if let Some(ssl) = config.get("security", "ssl_enabled") {
        if let Some(ssl_bool) = ssl.as_boolean() {
            println!("  SSL Enabled: {}", ssl_bool);
        }
    }

    if let Some(cert_path) = config.get("security", "cert_file") {
        if let Some(path_str) = cert_path.as_string() {
            println!("  Certificate: {}", path_str);
        }
    }

    // Show all secrets from the secrets file
    println!("\nSecrets configuration:");
    if let Some(api_key) = config.get("secrets", "api_key") {
        if let Some(key_str) = api_key.as_string() {
            println!("  API Key: {}", key_str);
        }
    }

    if let Some(jwt_secret) = config.get("secrets", "jwt_secret") {
        if let Some(jwt_str) = jwt_secret.as_string() {
            println!("  JWT Secret: {}", jwt_str);
        }
    }

    // Show how to add another config to the existing one
    println!("\nAdding new configuration file to existing config...");

    // Create an additional config file
    let monitoring_content = r#"#!config/toml
[monitoring]
enabled = true
interval = 60
url = "https://monitoring.example.com/report"
"#;

    fs::write("conf.d/monitoring.toml", monitoring_content)?;

    // Reload configuration to include the new file
    let mut updated_config = Config::new("app_with_includes");
    updated_config.load_from_file(Path::new("app_config.toml"))?;

    // Verify the new monitoring configuration
    if let Some(monitoring) = updated_config.get("monitoring", "enabled") {
        if let Some(enabled) = monitoring.as_boolean() {
            println!("  Monitoring Enabled: {}", enabled);
        }
    }

    if let Some(interval) = updated_config.get("monitoring", "interval") {
        if let Some(interval_val) = interval.as_integer() {
            println!("  Monitoring Interval: {} seconds", interval_val);
        }
    }

    println!("\nExample completed!");

    Ok(())
}

/// Setup directory structure for the example
fn setup_directories() -> Result<(), Box<dyn std::error::Error>> {
    // Create a directory for the configuration files
    fs::create_dir_all("conf.d")?;

    println!("Directory structure created.");

    Ok(())
}

/// Create all the necessary configuration files for the example
fn create_config_files() -> Result<(), Box<dyn std::error::Error>> {
    // Create a main TOML configuration file
    let main_config = r#"#!config/toml
app = { name = "Application with includes", version = "1.0.0" }
include = ["conf.d/server.ini", "conf.d/logging.yaml", "conf.d/database.json", "conf.d/security.toml", "conf.d/secrets.ini"]
"#;

    // Create an INI file for server settings
    let server_content = r#"#!config/ini
[server]
hostname = "app.example.com"
port = 8080
workers = 4
keepalive = true
"#;

    // Create a YAML file for logging settings
    let logging_content = r#"#!config/yaml
logging:
  level: info
  file: /var/log/myapp.log
  rotate: true
  max_size: 10485760
  handlers:
    - console
    - file
    - syslog
"#;

    // Create a JSON file for database settings
    let database_content = r#"#!config/json
{
  "database": {
    "host": "db.example.com",
    "port": 5432,
    "name": "app_database",
    "user": "app_user",
    "pool_size": 10,
    "timeout": 30,
    "ssl_mode": "require"
  }
}
"#;

    // Create a TOML file for security settings
    let security_content = r#"#!config/toml
[security]
ssl_enabled = true
cert_file = "/etc/ssl/certs/app.crt"
key_file = "/etc/ssl/private/app.key"
min_tls_version = "1.2"
ciphers = ["TLS_AES_128_GCM_SHA256", "TLS_AES_256_GCM_SHA384"]

[security.cors]
allowed_origins = ["https://example.com", "https://api.example.com"]
allowed_methods = ["GET", "POST", "PUT", "DELETE"]
allow_credentials = true
"#;

    // Create an INI file for secrets
    let secrets_content = r#"#!config/ini
[secrets]
api_key = "0123456789abcdef0123456789abcdef"
jwt_secret = "very-secret-jwt-signing-key"
encryption_key = "AES256-encryption-key-must-be-kept-secret"
"#;

    // Write all files
    fs::write("app_config.toml", main_config)?;
    fs::write("conf.d/server.ini", server_content)?;
    fs::write("conf.d/logging.yaml", logging_content)?;
    fs::write("conf.d/database.json", database_content)?;
    fs::write("conf.d/security.toml", security_content)?;
    fs::write("conf.d/secrets.ini", secrets_content)?;

    println!("Configuration files created:");
    println!("  - app_config.toml (main config)");
    println!("  - conf.d/server.ini");
    println!("  - conf.d/logging.yaml");
    println!("  - conf.d/database.json");
    println!("  - conf.d/security.toml");
    println!("  - conf.d/secrets.ini");

    Ok(())
}