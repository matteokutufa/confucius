// examples/json_example.rs
//! Example of using JSON format with the confucius library

use confucius::{Config, ConfigValue, ConfigFormat};
use std::path::Path;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a configuration for an app called "api_server"
    let mut config = Config::new("api_server");

    // Explicitly set the format to JSON
    config.set_format(ConfigFormat::Json);

    // Add basic configuration
    config.set("api", "version", ConfigValue::String("2.0.0".to_string()));
    config.set("api", "base_url", ConfigValue::String("/api/v2".to_string()));
    config.set("api", "enable_cors", ConfigValue::Boolean(true));

    // More complex configuration: array of endpoints
    let mut endpoints = Vec::new();

    // Endpoint 1: /users
    let mut endpoint1 = HashMap::new();
    endpoint1.insert("path".to_string(), ConfigValue::String("/users".to_string()));
    endpoint1.insert("method".to_string(), ConfigValue::String("GET".to_string()));
    endpoint1.insert("auth_required".to_string(), ConfigValue::Boolean(true));

    // Endpoint parameters
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

    // Add endpoints to configuration
    config.set("api", "endpoints", ConfigValue::Array(endpoints));

    // Database configuration
    config.set("database", "host", ConfigValue::String("localhost".to_string()));
    config.set("database", "port", ConfigValue::Integer(5432));
    config.set("database", "name", ConfigValue::String("api_db".to_string()));
    config.set("database", "user", ConfigValue::String("api_user".to_string()));
    config.set("database", "password", ConfigValue::String("secret123".to_string()));
    config.set("database", "max_connections", ConfigValue::Integer(100));

    // Logging configuration
    config.set("logging", "level", ConfigValue::String("info".to_string()));
    config.set("logging", "file", ConfigValue::String("/var/log/api_server.log".to_string()));
    config.set("logging", "stdout", ConfigValue::Boolean(true));

    // Array of log levels
    let log_levels = vec![
        ConfigValue::String("info".to_string()),
        ConfigValue::String("warn".to_string()),
        ConfigValue::String("error".to_string()),
    ];
    config.set("logging", "enabled_levels", ConfigValue::Array(log_levels));

    // Security settings
    let mut security = HashMap::new();
    security.insert("jwt_secret".to_string(), ConfigValue::String("your-secret-key".to_string()));
    security.insert("token_expiration".to_string(), ConfigValue::Integer(3600));

    let mut cors = HashMap::new();
    cors.insert("allowed_origins".to_string(), ConfigValue::Array(vec![
        ConfigValue::String("https://example.com".to_string()),
        ConfigValue::String("https://api.example.com".to_string()),
    ]));
    cors.insert("allowed_methods".to_string(), ConfigValue::Array(vec![
        ConfigValue::String("GET".to_string()),
        ConfigValue::String("POST".to_string()),
        ConfigValue::String("PUT".to_string()),
        ConfigValue::String("DELETE".to_string()),
    ]));
    security.insert("cors".to_string(), ConfigValue::Table(cors));

    config.set("security", "settings", ConfigValue::Table(security));

    // Save the configuration
    let json_path = Path::new("api_server.json");
    config.save_to_file(json_path)?;
    println!("JSON configuration saved to api_server.json");

    // Now reload the configuration
    let mut loaded_config = Config::new("api_server");
    loaded_config.load_from_file(json_path)?;
    println!("Configuration successfully loaded from JSON file");

    // Verify some values
    if let Some(base_url) = loaded_config.get("api", "base_url") {
        if let Some(url) = base_url.as_string() {
            println!("\nAPI Base URL: {}", url);
        }
    }

    if let Some(db_port) = loaded_config.get("database", "port") {
        if let Some(port) = db_port.as_integer() {
            println!("Database port: {}", port);
        }
    }

    // Check if we can read the array of endpoints
    if let Some(endpoints) = loaded_config.get("api", "endpoints") {
        if let ConfigValue::Array(endpoints_arr) = endpoints {
            println!("\nFound {} endpoints", endpoints_arr.len());

            // Extract information from the first endpoint
            if let Some(ConfigValue::Table(first_endpoint)) = endpoints_arr.first() {
                if let Some(ConfigValue::String(path)) = first_endpoint.get("path") {
                    println!("First endpoint path: {}", path);
                }

                if let Some(ConfigValue::Boolean(auth_required)) = first_endpoint.get("auth_required") {
                    println!("Authentication required: {}", auth_required);
                }

                // Check if there are default parameters
                if let Some(ConfigValue::Table(params)) = first_endpoint.get("default_params") {
                    println!("Default parameters:");
                    for (key, value) in params {
                        println!("  {} = {}", key, value);
                    }
                }
            }
        }
    }

    // Check security settings
    if let Some(ConfigValue::Table(security_settings)) = loaded_config.get("security", "settings") {
        println!("\nSecurity settings:");

        if let Some(ConfigValue::String(jwt_secret)) = security_settings.get("jwt_secret") {
            println!("  JWT Secret: {}", jwt_secret);
        }

        if let Some(ConfigValue::Integer(expiration)) = security_settings.get("token_expiration") {
            println!("  Token expiration: {} seconds", expiration);
        }

        // Check CORS settings
        if let Some(ConfigValue::Table(cors_settings)) = security_settings.get("cors") {
            println!("  CORS configuration:");

            if let Some(ConfigValue::Array(origins)) = cors_settings.get("allowed_origins") {
                println!("    Allowed origins:");
                for (i, origin) in origins.iter().enumerate() {
                    if let Some(origin_str) = origin.as_string() {
                        println!("      {}. {}", i + 1, origin_str);
                    }
                }
            }

            if let Some(ConfigValue::Array(methods)) = cors_settings.get("allowed_methods") {
                println!("    Allowed methods:");
                for (i, method) in methods.iter().enumerate() {
                    if let Some(method_str) = method.as_string() {
                        println!("      {}. {}", i + 1, method_str);
                    }
                }
            }
        }
    }

    Ok(())
}