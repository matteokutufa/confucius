# Conf-ucius - Configuration Management Library in Rust

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
    - INI (implemented)
    - TOML (planned)
    - YAML (planned)
    - JSON (planned)
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
confucius = "0.1.0"
```

## Usage Example

```rust
use confucius::{Config, ConfigValue};
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

## Configuration File Format

### INI Format Example

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

## Include Mechanism

The library supports the inclusion of other configuration files through the `include` directive:

```ini
# Include a single file
include=/path/to/extra.conf

# Include all files with .conf extension in a directory
include=/path/to/conf.d/*.conf
```

## Future Development

- Full implementation of TOML, YAML, and JSON formats
- Addition of schema validation features
- Support for overriding through environment variables
- Hot-reload functionality to reload configuration when it changes

## License

This project is released under the MIT license.