# confucius - Configuration Management Library in Rust

```
    ______            ____             _           
   / ____/___  ____  / __/_  _________(_)_  _______
  / /   / __ \/ __ \/ /_/ / / / ___/ / / / / / ___/
 / /___/ /_/ / / / / __/ /_/ / /__/ / / /_/ (__  ) 
 \____/\____/_/ /_/_/  \__,_/\___/_/_/\__,_/____/  
                                                   
 "Wisdom in configuration" 
```
# Confucius

> "Constancy is the virtue by which all other virtues bear fruit." - Confucius

Just as Confucius provided wisdom for life, Confucius provides wisdom for configuring your applications.

[![Crates.io](https://img.shields.io/crates/v/confucius.svg)](https://crates.io/crates/confucius)
[![Documentation](https://docs.rs/confucius/badge.svg)](https://docs.rs/confucius)
[![Build Status](https://github.com/yourusername/confucius/workflows/build/badge.svg)](https://github.com/yourusername/confucius/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/confucius.svg)](LICENSE)

## Features

`confucius` is a library for managing configuration files with support for:

- Automatic search for configuration files in standard paths
- Multiple formats support (INI, TOML, YAML, JSON)
- Include mechanism for modular configuration
- Format identification through shebang (`#!config/FORMAT`)
- Support for comments and typed values
- Robust validation system
- Hierarchical configuration with inheritance

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
confucius = "0.2.0"
```

## Basic Usage

```rust
use confucius::{Config, ConfigValue};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a configuration for an app called "myapp"
    let mut config = Config::new("myapp");
    
    // Load configuration from predefined paths
    match config.load() {
        Ok(_) => println!("Configuration loaded successfully!"),
        Err(e) => {
            println!("Error loading configuration: {}", e);
            // Fall back to a specific file
            config.load_from_file(Path::new("myapp.conf"))?;
        }
    }
    
    // Read values
    if let Some(server_name) = config.get("server", "hostname")
        .and_then(|v| v.as_string()) {
        println!("Server hostname: {}", server_name);
    }
    
    // Use convenience methods with default values
    let port = config.get_integer("server", "port", Some(8080)).unwrap_or(8080);
    println!("Server port: {}", port);
    
    // Modify values
    config.set("server", "timeout", ConfigValue::Integer(30));
    
    // Save the configuration
    config.save_to_file(Path::new("myapp_updated.conf"))?;
    
    Ok(())
}
```

## Supported Formats

Confucius supports multiple configuration formats:

### INI Format

```ini
#!config/ini
[section]
key1 = value1
key2 = "quoted value"
key3 = 123
key4 = true
```

### TOML Format

```toml
#!config/toml
[server]
hostname = "localhost"
port = 8080

[auth]
enabled = true
allowed_users = ["admin", "user1", "user2"]
```

### YAML Format

```yaml
#!config/yaml
app:
  name: My Web Application
  version: 1.2.0
  debug: false

database:
  main:
    host: localhost
    port: 5432
    user: webapp_user
```

### JSON Format

```json
#!config/json
{
  "api": {
    "version": "2.0.0",
    "base_url": "/api/v2",
    "enable_cors": true,
    "endpoints": [
      {
        "path": "/users",
        "method": "GET",
        "auth_required": true
      }
    ]
  }
}
```

## File Includes

Confucius supports including other configuration files:

```ini
#!config/ini
[main]
key = "main value"

# Include a single file
include=included.conf

# Or use glob patterns
include=conf.d/*.conf
```

## Configuration Validation

```rust
use confucius::{Config, ConfigValue, ValidationExt};
use confucius::{FieldConstraint, ValueType, ValidationSchema, FieldDefinition};

// Create a validation schema
let mut schema = ValidationSchema::new();

// Define required sections
schema.required_section("server")
      .required_section("database");

// Define fields with constraints
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

// Validate configuration
match config.validate(&schema) {
    Ok(_) => println!("Configuration is valid!"),
    Err(errors) => println!("Validation errors: {}", errors),
}

// Apply default values from schema
config.apply_defaults(&schema);
```

## Hierarchical Configuration

Confucius can be extended to support hierarchical configuration with inheritance:

```rust
// Define configuration levels
enum ConfigLevel {
  Default,    // Lowest priority
  Environment,
  Application,
  User,       // Highest priority
}

// Create a hierarchical config manager
let mut hierarchical_config = HierarchicalConfig::new("myapp")?;

// Get values (automatically uses highest priority level where the value exists)
let log_level = hierarchical_config.get_string("logging", "level", None);

// Override a value at a specific level
hierarchical_config.set_at_level(
ConfigLevel::User,
"logging",
"level",
ConfigValue::String("debug".to_string())
);

// Save a specific level
hierarchical_config.save_level(ConfigLevel::User)?;
```

## Examples

The repository includes several examples demonstrating various features:

- [Basic usage](examples/basic_usage.rs) - Basic configuration loading and usage
- [TOML format](examples/toml_example.rs) - Working with TOML configuration
- [YAML format](examples/yaml_example.rs) - Working with YAML configuration
- [JSON format](examples/json_example.rs) - Working with JSON configuration
- [Includes example](examples/includes_example.rs) - Using configuration file includes
- [Validation example](examples/validation_example.rs) - Configuration validation
- [Benchmark example](examples/benchmark_example.rs) - Performance benchmarking
- [Hierarchical config](examples/hierarchical_config.rs) - Multi-level configuration with inheritance

## License

Licensed under either of:

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.