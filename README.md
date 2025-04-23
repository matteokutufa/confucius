# confucius - Configuration Management Library in Rust

```
    ______            ____             _           
   / ____/___  ____  / __/_  _________(_)_  _______
  / /   / __ \/ __ \/ /_/ / / / ___/ / / / / / ___/
 / /___/ /_/ / / / / __/ /_/ / /__/ / / /_/ (__  ) 
 \____/\____/_/ /_/_/  \__,_/\___/_/_/\__,_/____/  
                                                   
 "Wisdom in configuration" 
```

`confucius` is a Rust library that simplifies configuration file management for applications. Designed to be flexible and intuitive, it offers support for various formats and includes advanced features such as file inclusion and automatic configuration file discovery.

## Features

- **Automatic search** for configuration files in standard paths
- Support for various **configuration formats**:
  - INI
  - TOML
  - YAML
  - JSON
- **Inclusion mechanism** for importing multiple files or with glob patterns
- **Format identification** via shebang (`#!config/FORMAT`)
- Support for **inline comments** (`# comment`)
- Support for **text values** in double quotes

## Search Paths

The library automatically searches for configuration files in the following paths (in order of priority), where `appname` is the name of the application:

1. `/etc/appname/appname.conf`
2. `/etc/appname.conf`
3. `/opt/etc/appname.conf`
4. `/home/username/.config/appname/appname.conf`
5. `/home/username/.config/appname.conf`
6. `<executable_path>/appname.conf`

## Installation

Add this dependency to your `Cargo.toml`:

```toml
[dependencies]
confucius = "0.1.4-beta"
```

## Usage Example

```rust
use confucius::{Config, ConfigValue, ConfigFormat};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a configuration for an app called "myapp"
    let mut config = Config::new("myapp");
    
    // Load the configuration from default paths
    match config.load() {
        Ok(_) => println!("Configuration loaded successfully!"),
        Err(e) => {
            println!("Error loading configuration: {}", e);
            // Load from a specific path
            config.load_from_file(Path::new("/path/to/myapp.conf"))?;
        }
    }
    
    // Read values
    if let Some(server) = config.get("server", "hostname") {
        if let Some(hostname) = server.as_string() {
            println!("Server: {}", hostname);
        }
    }
    
    // Modify values
    config.set("app", "version", ConfigValue::String("1.1.0".to_string()));
    
    // Save the configuration
    config.save()?;
    
    Ok(())
}
```

## Configuration Validation

`confucius` offers a powerful validation system that allows you to verify that the values in your configuration meet certain constraints:

```rust
use confucius::validation::{ValidationSchema, ValidationExt, FieldDefinition, FieldConstraint, ValueType};

// Create a validation schema
let mut schema = ValidationSchema::new();

// Define sections and fields
schema
    .required_section("server")
    .field(
        "server", 
        "port", 
        FieldDefinition::new(ValueType::Integer)
            .required()
            .constraint(FieldConstraint::integer().min_int(1).max_int(65535))
    );

// Validate the configuration
match config.validate(&schema) {
    Ok(_) => println!("Valid configuration!"),
    Err(errors) => println!("Errors: {}", errors),
}
```

For more details on validation, see the [dedicated section](#configuration-validation-1).

## Examples of Supported Configuration Formats

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

## Include Mechanism

The library supports the inclusion of other configuration files through the `include` directive:

```ini
# Include a single file
include=/path/to/extra.conf

# Include all files with .conf extension in a directory
include=/path/to/conf.d/*.conf
```

## Configuration Validation

`confucius` offers a powerful validation system that allows you to define a schema for configuration files and verify that values meet the defined constraints.

### Features of the Validation System

- Definition of required sections and fields
- Specification of expected data types
- Custom constraints for each data type
- Default values that are automatically applied
- Detailed error messages

### Validation Usage Example

```rust
use confucius::{Config, ConfigValue};
use confucius::validation::{ValidationSchema, ValidationExt, FieldDefinition, FieldConstraint, ValueType};

// Create a validation schema
let mut schema = ValidationSchema::new();

// Define required sections
schema.required_section("server");

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

// Load the configuration
let mut config = Config::new("myapp");
config.load()?;

// Apply default values defined in the schema
config.apply_defaults(&schema);

// Validate the configuration
match config.validate(&schema) {
    Ok(_) => println!("The configuration is valid!"),
    Err(errors) => println!("Validation errors: {}", errors),
}

// Alternatively, we can do both operations in a single call
match config.validate_and_apply_defaults(&schema) {
    Ok(_) => println!("The configuration is valid!"),
    Err(errors) => println!("Validation errors: {}", errors),
}
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

```rust
FieldConstraint::custom(
    |value| {
        // Custom validation function
        // Returns Ok(()) if the value is valid, Err(String) otherwise
        Ok(())
    },
    "Constraint description"
)
```

## Future Development

- Support for overriding through environment variables
- Hot-reload functionality to reload configuration when it changes
- Performance improvements with serialization support using serde
- Addition of custom formatting options for file writing

## License

This project is released under the MIT license.