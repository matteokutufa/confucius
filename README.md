# confucius - Configuration Management Library in Rust

```
    ______            ____             _           
   / ____/___  ____  / __/_  _________(_)_  _______
  / /   / __ \/ __ \/ /_/ / / / ___/ / / / / / ___/
 / /___/ /_/ / / / / __/ /_/ / /__/ / / /_/ (__  ) 
 \____/\____/_/ /_/_/  \__,_/\___/_/_/\__,_/____/  
                                                   
 "Wisdom in configuration" 
```

`confucius` is a flexible and powerful Rust library for configuration file management. Just as Confucius provided wisdom for life, `confucius` provides wisdom for configuring your applications.

## Key Features

- **Automatic configuration discovery** in standard system paths
- **Multi-format support**:
  - INI
  - TOML
  - YAML
  - JSON
- **Configuration includes** to import multiple files or glob patterns
- **Format auto-detection** via shebang (`#!config/FORMAT`)
- **Rich validation system** with constraints and default values
- **Inline comments support** (`# comment`) in configuration files
- **Quoted text values** support in all formats

## How Configuration Discovery Works

When creating a `Config` instance with your application name, `confucius` automatically searches for configuration files in these standard paths (in order of priority):

1. `/etc/appname/appname.conf`
2. `/etc/appname.conf`
3. `/opt/etc/appname.conf`
4. `/home/username/.config/appname/appname.conf`
5. `/home/username/.config/appname.conf`
6. `<executable_path>/appname.conf`

This provides a consistent behavior across different environments and follows common conventions for configuration files.

## Installation

Add this dependency to your `Cargo.toml`:

```toml
[dependencies]
confucius = "0.1.5-beta"
```

## Basic Usage

```rust
use confucius::{Config, ConfigValue, ConfigFormat};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new configuration for an app named "myapp"
    let mut config = Config::new("myapp");
    
    // Load configuration (automatically discovers in standard paths)
    match config.load() {
        Ok(_) => println!("Configuration loaded successfully!"),
        Err(e) => {
            println!("Error loading configuration: {}", e);
            // Or load from a specific path as fallback
            config.load_from_file(Path::new("/path/to/myapp.conf"))?;
        }
    }
    
    // Access configuration values
    if let Some(server) = config.get("server", "hostname") {
        if let Some(hostname) = server.as_string() {
            println!("Server hostname: {}", hostname);
        }
    }
    
    // Integer values
    if let Some(port) = config.get("server", "port") {
        if let Some(port_num) = port.as_integer() {
            println!("Server port: {}", port_num);
        }
    }
    
    // Modify configuration values
    config.set("app", "version", ConfigValue::String("1.1.0".to_string()));
    
    // Save the configuration back to file
    config.save()?;
    
    // Or save to a specific file
    config.save_to_file(Path::new("/path/to/new/config.conf"))?;
    
    Ok(())
}
```

## Configuration File Formats

### INI Format

```ini
#!config/ini
# This is a comment

[app]
name = "My Application"
version = "1.0.0"
debug = true # Inline comment

[server]
hostname = "localhost"
port = 8080

# Include other configuration files
include=/etc/myapp/extra.conf
include=/etc/myapp/conf.d/*.conf
```

### TOML Format

```toml
#!config/toml
# This is a comment

[app]
name = "My Application"
version = "1.0.0"
debug = true

[server]
hostname = "localhost"
port = 8080

# Include other configuration files
include = [
    "/etc/myapp/extra.conf", 
    "/etc/myapp/conf.d/*.conf"
]

# TOML format also supports arrays and nested tables
[database]
connections = [
    { name = "main", host = "localhost", port = 5432 },
    { name = "replica", host = "replica.example.com", port = 5432 }
]
```

### YAML Format

```yaml
#!config/yaml
# This is a comment

app:
  name: "My Application"
  version: "1.0.0"
  debug: true

server:
  hostname: "localhost"
  port: 8080

# Include other configuration files
include:
  - /etc/myapp/extra.conf
  - /etc/myapp/conf.d/*.conf

# YAML format supports complex data structures
database:
  connections:
    - name: main
      host: localhost
      port: 5432
    - name: replica
      host: replica.example.com
      port: 5432
```

### JSON Format

```json
#!config/json
{
  "app": {
    "name": "My Application",
    "version": "1.0.0",
    "debug": true
  },
  
  "server": {
    "hostname": "localhost",
    "port": 8080
  },
  
  "include": [
    "/etc/myapp/extra.conf",
    "/etc/myapp/conf.d/*.conf"
  ],
  
  "database": {
    "connections": [
      {
        "name": "main",
        "host": "localhost",
        "port": 5432
      },
      {
        "name": "replica",
        "host": "replica.example.com",
        "port": 5432
      }
    ]
  }
}
```

## The Include Mechanism

One of the most powerful features of `confucius` is the ability to include other configuration files. This allows you to:

1. Split large configurations into manageable pieces
2. Share common configuration across multiple applications
3. Override default settings with environment-specific ones

### How to use includes

```ini
# Include a single file
include=/path/to/extra.conf

# Include all .conf files in a directory
include=/path/to/conf.d/*.conf
```

The include mechanism works across all supported formats. When including files:

- Paths can be absolute or relative to the including file
- Glob patterns are supported for including multiple files
- Included files can have a different format than the parent file
- Include directives can be nested (files can include other files)

## Advanced Configuration Validation

`confucius` offers a powerful validation system that lets you define a schema for your configuration and enforce constraints on values.

### Creating a Validation Schema

```rust
use confucius::{Config, ConfigValue};
use confucius::validation::{ValidationSchema, ValidationExt, FieldDefinition, FieldConstraint, ValueType};

// Create a validation schema
let mut schema = ValidationSchema::new();

// Define required sections
schema.required_section("server");

// Define fields with constraints
schema.field(
    "server", 
    "hostname", 
    FieldDefinition::new(ValueType::String)
        .required()  // This field must exist
        .description("Server hostname")
        .constraint(FieldConstraint::string()
            .min_length(3)  // At least 3 characters
            .max_length(255)  // At most 255 characters
        )
);

schema.field(
    "server", 
    "port", 
    FieldDefinition::new(ValueType::Integer)
        .required()
        .description("Server port")
        .default(ConfigValue::Integer(8080))  // Default value if not specified
        .constraint(FieldConstraint::integer()
            .min_int(1)
            .max_int(65535)
        )
);

// Optional field with allowed values constraint
schema.field(
    "server",
    "mode",
    FieldDefinition::new(ValueType::String)
        .description("Server operation mode")
        .constraint(FieldConstraint::string()
            .allowed_string_values(vec!["development", "testing", "production"])
        )
);
```

### Using the Validation Schema

```rust
// Validate configuration against schema
match config.validate(&schema) {
    Ok(_) => println!("Configuration is valid!"),
    Err(errors) => {
        println!("Validation errors:");
        println!("{}", errors);
        return Err(errors.into());
    }
};

// Apply default values from schema
config.apply_defaults(&schema);

// Or do both in one operation
config.validate_and_apply_defaults(&schema)?;
```

### Available Constraint Types

#### String Constraints

```rust
FieldConstraint::string()
    .min_length(3)        // Minimum length
    .max_length(255)      // Maximum length
    .pattern("^[a-z]+$")  // Regex pattern
    .allowed_string_values(vec!["value1", "value2"])  // Allowed values
```

#### Integer Constraints

```rust
FieldConstraint::integer()
    .min_int(1)           // Minimum value
    .max_int(1000)        // Maximum value
    .allowed_int_values(vec![1, 2, 3])  // Allowed values
```

#### Float Constraints

```rust
FieldConstraint::float()
    .min_float(0.0)       // Minimum value
    .max_float(100.0)     // Maximum value
```

#### Array Constraints

```rust
FieldConstraint::array()
    .min_length(1)        // Minimum length
    .max_length(10)       // Maximum length
    .item_type(           // Element type
        FieldDefinition::new(ValueType::String)
            .constraint(FieldConstraint::string().min_length(3))
    )
```

#### Custom Constraints

You can define custom validation logic for more complex scenarios:

```rust
FieldConstraint::custom(
    |value| {
        // Custom validation function
        // For example, validating that a string is a valid hostname
        if let ConfigValue::String(s) = value {
            if s.contains(":") {
                return Err(format!("Hostname should not contain colon: {}", s));
            }
        }
        Ok(())
    },
    "Hostname validation"  // Description for error messages
)
```

## Type Conversion

`confucius` provides convenient methods to convert configuration values to their appropriate Rust types:

```rust
// Convert to string
if let Some(name) = config.get("app", "name") {
    if let Some(name_str) = name.as_string() {
        println!("App name: {}", name_str);
    }
}

// Convert to integer
if let Some(port) = config.get("server", "port") {
    if let Some(port_num) = port.as_integer() {
        println!("Port: {}", port_num);
    }
}

// Convert to float
if let Some(timeout) = config.get("server", "timeout") {
    if let Some(timeout_val) = timeout.as_float() {
        println!("Timeout: {} seconds", timeout_val);
    }
}

// Convert to boolean
if let Some(debug) = config.get("app", "debug") {
    if let Some(debug_val) = debug.as_boolean() {
        if debug_val {
            println!("Debug mode is enabled");
        }
    }
}
```

## Roadmap

Future development plans for `confucius` include:

- **Environment variable override** - Allow overriding configuration values through environment variables
- **Hot-reload functionality** - Automatically reload configuration when files change on disk
- **Performance optimizations** - Improved serialization/deserialization with serde
- **Custom formatters** - Add options for customizing the formatting of saved configuration files
- **Schema-based documentation** - Generate documentation for your configuration based on schema definitions
- **Configuration layering** - More sophisticated layering of configuration from multiple sources

## Contributing

Contributions to `confucius` are welcome! Areas that could use help include:

- Additional format support
- Performance improvements
- Documentation enhancements
- Example applications
- Testing on different platforms

## License

This project is released under the MIT license.