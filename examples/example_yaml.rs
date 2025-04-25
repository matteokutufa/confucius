// examples/yaml_example.rs
//! Example of using YAML format with the confucius library

use confucius::{Config, ConfigValue, ConfigFormat};
use std::path::Path;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a configuration for an app called "webapp"
    let mut config = Config::new("webapp");

    // Explicitly set the format to YAML
    config.set_format(ConfigFormat::Yaml);

    // Add basic values
    config.set("app", "name", ConfigValue::String("My Web Application".to_string()));
    config.set("app", "version", ConfigValue::String("1.2.0".to_string()));
    config.set("app", "debug", ConfigValue::Boolean(false));

    // More complex configuration: nested table
    let mut db_config = HashMap::new();
    db_config.insert("host".to_string(), ConfigValue::String("localhost".to_string()));
    db_config.insert("port".to_string(), ConfigValue::Integer(5432));
    db_config.insert("user".to_string(), ConfigValue::String("webapp_user".to_string()));
    db_config.insert("password".to_string(), ConfigValue::String("secretpassword".to_string()));

    config.set("database", "main", ConfigValue::Table(db_config));

    // Array of objects for endpoint configuration
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

    // Caching configuration
    let mut cache_config = HashMap::new();
    cache_config.insert("enabled".to_string(), ConfigValue::Boolean(true));
    cache_config.insert("ttl".to_string(), ConfigValue::Integer(3600));

    let mut redis_config = HashMap::new();
    redis_config.insert("host".to_string(), ConfigValue::String("localhost".to_string()));
    redis_config.insert("port".to_string(), ConfigValue::Integer(6379));
    cache_config.insert("redis".to_string(), ConfigValue::Table(redis_config));

    config.set("caching", "config", ConfigValue::Table(cache_config));

    // Save the configuration
    let yaml_path = Path::new("webapp.yaml");
    config.save_to_file(yaml_path)?;
    println!("YAML configuration saved to webapp.yaml");

    // Now reload the configuration
    let mut loaded_config = Config::new("webapp");
    loaded_config.load_from_file(yaml_path)?;

    // Verify values
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

    // Working with nested configurations
    if let Some(db_table) = loaded_config.get_table("database", "main") {
        println!("\nDatabase Configuration:");
        if let Some(ConfigValue::String(host)) = db_table.get("host") {
            println!("  Host: {}", host);
        }
        if let Some(ConfigValue::Integer(port)) = db_table.get("port") {
            println!("  Port: {}", port);
        }
        if let Some(ConfigValue::String(user)) = db_table.get("user") {
            println!("  User: {}", user);
        }
    }

    // Working with arrays of complex objects
    if let Some(endpoints_array) = loaded_config.get_array("api", "endpoints") {
        println!("\nAPI Endpoints:");
        for (i, endpoint) in endpoints_array.iter().enumerate() {
            if let ConfigValue::Table(endpoint_table) = endpoint {
                println!("  Endpoint #{}:", i + 1);

                if let Some(ConfigValue::String(path)) = endpoint_table.get("path") {
                    println!("    Path: {}", path);
                }

                if let Some(ConfigValue::String(method)) = endpoint_table.get("method") {
                    println!("    Method: {}", method);
                }

                if let Some(ConfigValue::Boolean(auth)) = endpoint_table.get("auth_required") {
                    println!("    Auth Required: {}", auth);
                }
            }
        }
    }

    // Checking caching configuration
    if let Some(cache_table) = loaded_config.get_table("caching", "config") {
        println!("\nCaching Configuration:");
        if let Some(ConfigValue::Boolean(enabled)) = cache_table.get("enabled") {
            println!("  Enabled: {}", enabled);
        }

        if let Some(ConfigValue::Integer(ttl)) = cache_table.get("ttl") {
            println!("  TTL: {} seconds", ttl);
        }

        // Accessing nested table
        if let Some(ConfigValue::Table(redis)) = cache_table.get("redis") {
            println!("  Redis:");
            if let Some(ConfigValue::String(host)) = redis.get("host") {
                println!("    Host: {}", host);
            }
            if let Some(ConfigValue::Integer(port)) = redis.get("port") {
                println!("    Port: {}", port);
            }
        }
    }

    Ok(())
}