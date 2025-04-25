// examples/validation_example.rs
//! Example of using the validation system in the confucius library

use confucius::{Config, ConfigValue};
use confucius::ValidationExt;
use confucius::{
    FieldConstraint,
    ValueType,
    ValidationSchema,
    FieldDefinition,
};

use std::path::Path;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Configuration Validation Example ===\n");

    // Create a test TOML config file with some issues to validate
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
# Missing user and password fields here

[logging]
level = "debug"
file = "/var/log/myapp.log"

[api]
timeout = 0  # Invalid value (too small)
max_requests = 1500  # Invalid value (too large)
"#;

    // Write the configuration file
    let config_path = Path::new("test_config.toml");
    let mut file = File::create(&config_path)?;
    file.write_all(config_content.as_bytes())?;

    println!("Test configuration file created at: {}", config_path.display());

    // Load the configuration
    let mut config = Config::new("myapp");
    config.load_from_file(&config_path)?;

    println!("Configuration loaded successfully!");

    // Create a validation schema
    println!("\nCreating validation schema...");
    let schema = create_validation_schema();

    // First validation attempt (expect errors)
    println!("\n=== INITIAL VALIDATION (expect errors) ===");
    match config.validate(&schema) {
        Ok(_) => println!("Configuration is valid! (unexpected)"),
        Err(errors) => println!("Validation errors found:\n{}", errors),
    }

    // Apply default values
    println!("\n=== APPLYING DEFAULT VALUES ===");
    config.apply_defaults(&schema);

    println!("Default values applied");

    // Show the modified values
    println!("\nValues after applying defaults:");

    if let Some(user) = config.get("database", "user") {
        println!("  database.user = {}", user);
    }

    if let Some(password) = config.get("database", "password") {
        println!("  database.password = {}", password);
    }

    if let Some(max_size) = config.get("logging", "max_size") {
        println!("  logging.max_size = {}", max_size);
    }

    // Manually fix the remaining issues
    println!("\n=== FIXING REMAINING ISSUES ===");

    config.set("api", "timeout", ConfigValue::Integer(30));
    config.set("api", "max_requests", ConfigValue::Integer(1000));

    println!("Invalid values fixed manually");

    // Final validation
    println!("\n=== FINAL VALIDATION ===");
    match config.validate(&schema) {
        Ok(_) => println!("Success! Configuration is now valid."),
        Err(errors) => println!("Validation errors still found:\n{}", errors),
    }

    // Save the validated configuration
    let validated_path = Path::new("validated_config.toml");
    config.save_to_file(&validated_path)?;
    println!("\nValidated configuration saved to: {}", validated_path.display());

    // Example of validating and applying defaults in a single step
    println!("\n=== USING VALIDATE_AND_APPLY_DEFAULTS ===");

    // Load the original config again
    let mut config2 = Config::new("myapp");
    config2.load_from_file(&config_path)?;

    // Fix API values first (can't be fixed by defaults)
    config2.set("api", "timeout", ConfigValue::Integer(30));
    config2.set("api", "max_requests", ConfigValue::Integer(1000));

    // Validate and apply defaults in one step
    match config2.validate_and_apply_defaults(&schema) {
        Ok(_) => println!("Success! Configuration is valid after applying defaults."),
        Err(errors) => println!("Validation errors after applying defaults:\n{}", errors),
    }

    println!("\nExample completed!");

    Ok(())
}

/// Creates a validation schema for the configuration
fn create_validation_schema() -> ValidationSchema {
    let mut schema = ValidationSchema::new();

    // Define required sections
    schema.required_section("server")
        .required_section("database")
        .required_section("logging")
        .allow_unknown_sections(true);  // Allow undefined sections (like 'api')

    // Define fields for the "server" section
    schema.field(
        "server",
        "hostname",
        FieldDefinition::new(ValueType::String)
            .required()
            .description("Server hostname")
            .constraint(FieldConstraint::string()
                .min_length(3)
                .max_length(255))
    );

    schema.field(
        "server",
        "port",
        FieldDefinition::new(ValueType::Integer)
            .required()
            .description("Server port")
            .constraint(FieldConstraint::integer()
                .min_int(1)
                .max_int(65535))
    );

    schema.field(
        "server",
        "max_connections",
        FieldDefinition::new(ValueType::Integer)
            .description("Maximum number of connections")
            .default(ConfigValue::Integer(100))
            .constraint(FieldConstraint::integer()
                .min_int(1)
                .max_int(10000))
    );

    schema.field(
        "server",
        "ssl",
        FieldDefinition::new(ValueType::Boolean)
            .description("Enable SSL")
            .default(ConfigValue::Boolean(false))
    );

    // Define fields for the "database" section
    schema.field(
        "database",
        "host",
        FieldDefinition::new(ValueType::String)
            .required()
            .description("Database host")
    );

    schema.field(
        "database",
        "port",
        FieldDefinition::new(ValueType::Integer)
            .required()
            .description("Database port")
            .constraint(FieldConstraint::integer()
                .min_int(1)
                .max_int(65535))
    );

    schema.field(
        "database",
        "name",
        FieldDefinition::new(ValueType::String)
            .required()
            .description("Database name")
    );

    schema.field(
        "database",
        "user",
        FieldDefinition::new(ValueType::String)
            .required()
            .description("Database user")
            .default(ConfigValue::String("default_user".to_string()))
    );

    schema.field(
        "database",
        "password",
        FieldDefinition::new(ValueType::String)
            .required()
            .description("Database password")
            .default(ConfigValue::String("default_password".to_string()))
    );

    // Define fields for the "logging" section
    schema.field(
        "logging",
        "level",
        FieldDefinition::new(ValueType::String)
            .required()
            .description("Logging level")
            .constraint(FieldConstraint::string()
                .allowed_string_values(vec!["debug", "info", "warn", "error"]))
    );

    schema.field(
        "logging",
        "file",
        FieldDefinition::new(ValueType::String)
            .required()
            .description("Log file path")
    );

    schema.field(
        "logging",
        "max_size",
        FieldDefinition::new(ValueType::Integer)
            .description("Maximum log file size in MB")
            .default(ConfigValue::Integer(10))
            .constraint(FieldConstraint::integer()
                .min_int(1)
                .max_int(1000))
    );

    // Define fields for the "api" section (optional section)
    schema.field(
        "api",
        "timeout",
        FieldDefinition::new(ValueType::Integer)
            .description("API timeout in seconds")
            .constraint(FieldConstraint::integer()
                .min_int(1)
                .max_int(300))
    );

    schema.field(
        "api",
        "max_requests",
        FieldDefinition::new(ValueType::Integer)
            .description("Maximum API requests per minute")
            .constraint(FieldConstraint::integer()
                .min_int(1)
                .max_int(1000))
    );

    // Add a field with a custom constraint
    schema.field(
        "api",
        "rate_limit",
        FieldDefinition::new(ValueType::Table)
            .description("Rate limiting configuration")
            .constraint(FieldConstraint::custom(
                |value| {
                    if let ConfigValue::Table(table) = value {
                        // Check for required keys
                        if !table.contains_key("requests") || !table.contains_key("period") {
                            return Err("Must contain both 'requests' and 'period' keys".to_string());
                        }
                        // Validate 'requests' is a positive integer
                        if let Some(ConfigValue::Integer(req)) = table.get("requests") {
                            if *req <= 0 {
                                return Err("'requests' must be positive".to_string());
                            }
                        } else {
                            return Err("'requests' must be an integer".to_string());
                        }
                        // Validate 'period' is a string in the correct format
                        if let Some(ConfigValue::String(period)) = table.get("period") {
                            if !period.ends_with('s') && !period.ends_with('m') && !period.ends_with('h') {
                                return Err("'period' must end with 's', 'm', or 'h'".to_string());
                            }
                        } else {
                            return Err("'period' must be a string".to_string());
                        }
                        Ok(())
                    } else {
                        Err("Must be a table with 'requests' and 'period' keys".to_string())
                    }
                },
                "Rate limit configuration validation"
            ))
    );

    schema
}