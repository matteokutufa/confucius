// examples/toml_example.rs
//! Example of using TOML format with the confucius library

use confucius::{Config, ConfigValue, ConfigFormat};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a configuration for an app called "myapp"
    let mut config = Config::new("myapp");

    // Explicitly set the format to TOML
    config.set_format(ConfigFormat::Toml);

    // Add some values
    config.set("server", "hostname", ConfigValue::String("localhost".to_string()));
    config.set("server", "port", ConfigValue::Integer(8080));
    config.set("auth", "enabled", ConfigValue::Boolean(true));

    // Array of strings
    let users = vec![
        ConfigValue::String("admin".to_string()),
        ConfigValue::String("user1".to_string()),
        ConfigValue::String("user2".to_string()),
    ];
    config.set("auth", "allowed_users", ConfigValue::Array(users));

    // Add environment-specific settings
    config.set("environments", "list", ConfigValue::Array(vec![
        ConfigValue::String("development".to_string()),
        ConfigValue::String("staging".to_string()),
        ConfigValue::String("production".to_string()),
    ]));

    config.set("environments", "current", ConfigValue::String("development".to_string()));

    // Database configuration
    config.set("database", "host", ConfigValue::String("localhost".to_string()));
    config.set("database", "port", ConfigValue::Integer(5432));
    config.set("database", "name", ConfigValue::String("myapp_db".to_string()));
    config.set("database", "user", ConfigValue::String("dbuser".to_string()));
    config.set("database", "password", ConfigValue::String("dbpassword".to_string()));

    // Save the configuration
    let toml_path = Path::new("myapp.toml");
    config.save_to_file(toml_path)?;
    println!("TOML configuration saved to myapp.toml");

    // Now reload the configuration
    let mut loaded_config = Config::new("myapp");
    loaded_config.load_from_file(toml_path)?;

    // Verify values
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

    // Demonstrate the use of get_* convenience methods
    let current_env = loaded_config.get_string("environments", "current", Some("production"));
    println!("Current environment: {}", current_env.unwrap_or_else(|| "unknown".to_string()));

    let auth_enabled = loaded_config.get_boolean("auth", "enabled", Some(false));
    println!("Auth enabled: {}", auth_enabled.unwrap_or(false));

    // Check if we have users array
    if let Some(users_array) = loaded_config.get_array("auth", "allowed_users") {
        println!("Allowed users:");
        for (i, user) in users_array.iter().enumerate() {
            if let Some(user_str) = user.as_string() {
                println!("  {}. {}", i + 1, user_str);
            }
        }
    }

    // Print the database connection string
    let db_host = loaded_config.get_string("database", "host", None);
    let db_port = loaded_config.get_integer("database", "port", None);
    let db_name = loaded_config.get_string("database", "name", None);
    let db_user = loaded_config.get_string("database", "user", None);

    if let (Some(host), Some(port), Some(name), Some(user)) = (db_host, db_port, db_name, db_user) {
        println!("Database connection: postgresql://{}@{}:{}/{}",
                 user, host, port, name);
    }

    Ok(())
}