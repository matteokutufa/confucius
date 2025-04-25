// examples/hierarchical_config.rs
//! Example of implementing hierarchical configuration with inheritance
//!
//! This example shows how to create a multi-level configuration system
//! where settings can be overridden at different levels (default, environment,
//! application, user) with proper inheritance.

use std::fs;
use std::path::Path;
use std::env;
use std::io::Write;

use confucius::{Config, ConfigValue, ConfigFormat};

// Define our configuration levels
enum ConfigLevel {
    Default,
    Environment,
    Application,
    User,
}

impl ConfigLevel {
    fn to_string(&self) -> String {
        match self {
            ConfigLevel::Default => "default".to_string(),
            ConfigLevel::Environment => "environment".to_string(),
            ConfigLevel::Application => "application".to_string(),
            ConfigLevel::User => "user".to_string(),
        }
    }

    fn priority(&self) -> i32 {
        match self {
            ConfigLevel::Default => 0,
            ConfigLevel::Environment => 10,
            ConfigLevel::Application => 20,
            ConfigLevel::User => 30,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Hierarchical Configuration Example ===\n");

    // Create example configuration files for each level
    create_config_files()?;

    // Create our layered configuration manager
    let mut hierarchical_config = HierarchicalConfig::new("myapp")?;

    // Test the hierarchical settings
    println!("\n=== Testing Hierarchical Configuration ===");

    // 1. Log level - this setting exists at all levels
    let log_level = hierarchical_config.get_string("logging", "level", None);
    println!("Log level: {} (should be 'debug' from user level)",
             log_level.unwrap_or_else(|| "not found".to_string()));

    // 2. Log file - overridden at environment and application level, but not user level
    let log_file = hierarchical_config.get_string("logging", "file", None);
    println!("Log file: {} (should be '/var/log/app/myapp.log' from application level)",
             log_file.unwrap_or_else(|| "not found".to_string()));

    // 3. Database host - only defined at default and environment levels
    let db_host = hierarchical_config.get_string("database", "host", None);
    println!("Database host: {} (should be 'db.staging.example.com' from environment level)",
             db_host.unwrap_or_else(|| "not found".to_string()));

    // 4. Database user - only defined at application level
    let db_user = hierarchical_config.get_string("database", "user", None);
    println!("Database user: {} (should be 'app_user' from application level)",
             db_user.unwrap_or_else(|| "not found".to_string()));

    // 5. Theme - only defined at user level
    let theme = hierarchical_config.get_string("ui", "theme", None);
    println!("UI theme: {} (should be 'dark' from user level)",
             theme.unwrap_or_else(|| "not found".to_string()));

    // 6. Value only in default settings
    let timeout = hierarchical_config.get_integer("server", "timeout", None);
    println!("Server timeout: {} (should be 30 from default level)",
             timeout.unwrap_or(0));

    // Demonstrate overriding a value at runtime
    println!("\n=== Overriding Values at Runtime ===");

    // Override the log level
    hierarchical_config.set_at_level(
        ConfigLevel::User, "logging", "level", ConfigValue::String("trace".to_string()));

    let log_level = hierarchical_config.get_string("logging", "level", None);
    println!("Log level after override: {} (should be 'trace')",
             log_level.unwrap_or_else(|| "not found".to_string()));

    // Save user settings (with our override)
    hierarchical_config.save_level(ConfigLevel::User)?;
    println!("User settings saved with override");

    // Examine a specific level
    println!("\n=== Values at Environment Level Only ===");
    let env_values = hierarchical_config.get_level_values(ConfigLevel::Environment);

    println!("Environment-level configuration contents:");
    for (section, keys) in &env_values.values {
        println!("  [{}]", section);
        for (key, value) in keys {
            println!("  {} = {}", key, value);
        }
    }

    println!("\nExample completed!");

    Ok(())
}

/// A hierarchical configuration manager that combines multiple configuration levels
struct HierarchicalConfig {
    levels: Vec<(ConfigLevel, Config)>,
    app_name: String,
}

impl HierarchicalConfig {
    /// Create a new hierarchical configuration
    fn new(app_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut levels = Vec::new();

        // Load each configuration level
        let default_config = load_config_level(app_name, ConfigLevel::Default)?;
        let env_config = load_config_level(app_name, ConfigLevel::Environment)?;
        let app_config = load_config_level(app_name, ConfigLevel::Application)?;
        let user_config = load_config_level(app_name, ConfigLevel::User)?;

        // Add levels in order of priority (lowest to highest)
        levels.push((ConfigLevel::Default, default_config));
        levels.push((ConfigLevel::Environment, env_config));
        levels.push((ConfigLevel::Application, app_config));
        levels.push((ConfigLevel::User, user_config));

        Ok(HierarchicalConfig {
            levels,
            app_name: app_name.to_string(),
        })
    }

    /// Get a configuration value, respecting priority levels
    fn get(&self, section: &str, key: &str) -> Option<&ConfigValue> {
        // Start from highest priority and work down
        for (_, config) in self.levels.iter().rev() {
            if let Some(value) = config.get(section, key) {
                return Some(value);
            }
        }
        None
    }

    /// Get a string value with optional default
    fn get_string(&self, section: &str, key: &str, default: Option<&str>) -> Option<String> {
        self.get(section, key)
            .and_then(|v| v.as_string().cloned())
            .or_else(|| default.map(|s| s.to_string()))
    }

    /// Get an integer value with optional default
    fn get_integer(&self, section: &str, key: &str, default: Option<i64>) -> Option<i64> {
        self.get(section, key)
            .and_then(|v| v.as_integer())
            .or(default)
    }

    /// Get a float value with optional default
    fn get_float(&self, section: &str, key: &str, default: Option<f64>) -> Option<f64> {
        self.get(section, key)
            .and_then(|v| v.as_float())
            .or(default)
    }

    /// Get a boolean value with optional default
    fn get_boolean(&self, section: &str, key: &str, default: Option<bool>) -> Option<bool> {
        self.get(section, key)
            .and_then(|v| v.as_boolean())
            .or(default)
    }

    /// Set a value at a specific configuration level
    fn set_at_level(&mut self, level: ConfigLevel, section: &str, key: &str, value: ConfigValue) {
        for (lvl, config) in &mut self.levels {
            if lvl.priority() == level.priority() {
                config.set(section, key, value);
                break;
            }
        }
    }

    /// Save a specific configuration level
    fn save_level(&self, level: ConfigLevel) -> Result<(), Box<dyn std::error::Error>> {
        for (lvl, config) in &self.levels {
            if lvl.priority() == level.priority() {
                // Define the file path based on the level
                let file_path = match level {
                    ConfigLevel::Default => format!("{}_default.toml", self.app_name),
                    ConfigLevel::Environment => format!("{}_environment.toml", self.app_name),
                    ConfigLevel::Application => format!("{}_application.toml", self.app_name),
                    ConfigLevel::User => format!("{}_user.toml", self.app_name),
                };

                config.save_to_file(Path::new(&file_path))?;
                break;
            }
        }
        Ok(())
    }

    /// Get a reference to the Config object for a specific level
    fn get_level_values(&self, level: ConfigLevel) -> &Config {
        for (lvl, config) in &self.levels {
            if lvl.priority() == level.priority() {
                return config;
            }
        }
        // Default to the lowest level if not found (shouldn't happen)
        &self.levels[0].1
    }
}

/// Load a specific configuration level
fn load_config_level(app_name: &str, level: ConfigLevel) -> Result<Config, Box<dyn std::error::Error>> {
    let level_name = level.to_string();
    let file_path = format!("{}_{}.toml", app_name, level_name);

    let mut config = Config::new(&format!("{}_{}", app_name, level_name));
    config.set_format(ConfigFormat::Toml);

    if Path::new(&file_path).exists() {
        config.load_from_file(Path::new(&file_path))?;
    }

    Ok(config)
}

/// Create example configuration files for each level
fn create_config_files() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating example configuration files...");

    // Default configuration (lowest priority)
    let default_config = r#"#!config/toml
[logging]
level = "info"
file = "/var/log/default.log"
rotation = "daily"

[database]
host = "db.example.com"
port = 5432
name = "myapp_db"

[server]
host = "0.0.0.0"
port = 8080
timeout = 30
"#;

    // Environment configuration (staging environment)
    let environment_config = r#"#!config/toml
[logging]
level = "info"
file = "/var/log/staging/myapp.log"

[database]
host = "db.staging.example.com"
port = 5432

[server]
host = "0.0.0.0"
port = 8000
"#;

    // Application configuration
    let application_config = r#"#!config/toml
[logging]
level = "warn"
file = "/var/log/app/myapp.log"

[database]
user = "app_user"
password = "securepassword"

[server]
workers = 4
"#;

    // User configuration (highest priority)
    let user_config = r#"#!config/toml
[logging]
level = "debug"

[ui]
theme = "dark"
font_size = 14
show_tooltips = true
"#;

    // Write the configuration files
    fs::write("myapp_default.toml", default_config)?;
    fs::write("myapp_environment.toml", environment_config)?;
    fs::write("myapp_application.toml", application_config)?;
    fs::write("myapp_user.toml", user_config)?;

    println!("Example configuration files created:");
    println!("  - myapp_default.toml");
    println!("  - myapp_environment.toml");
    println!("  - myapp_application.toml");
    println!("  - myapp_user.toml");

    Ok(())
}