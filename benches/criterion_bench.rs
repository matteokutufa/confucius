// examples/benchmark_example.rs
//! Example of benchmarking performance for the confucius library
//!
//! This example demonstrates how to set up simple benchmarks to measure
//! the performance of the library's core operations.

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use std::collections::HashMap;

use confucius::{Config, ConfigValue, ConfigFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Confucius Performance Benchmark Example ===\n");

    // Create test configuration files
    create_test_files()?;

    // Run benchmarks
    benchmark_parse_formats()?;
    benchmark_get_values()?;
    benchmark_set_values()?;
    benchmark_save_formats()?;

    // Clean up test files
    cleanup_files()?;

    println!("\nBenchmarks completed!");

    Ok(())
}

/// Run a benchmark function multiple times and report the average
fn run_benchmark<F>(name: &str, iterations: u32, mut f: F) -> Result<Duration, Box<dyn std::error::Error>>
where
    F: FnMut() -> Result<(), Box<dyn std::error::Error>>
{
    println!("Running benchmark: {}", name);

    // Warm up
    for _ in 0..5 {
        f()?;
    }

    // Timed runs
    let mut total_duration = Duration::new(0, 0);

    for i in 0..iterations {
        let start = Instant::now();
        f()?;
        let duration = start.elapsed();
        total_duration += duration;

        // Print progress dots
        if i % 10 == 0 {
            print!(".");
        }
    }
    println!();

    let avg_duration = total_duration / iterations;
    println!("  Average duration ({} iterations): {:?}", iterations, avg_duration);

    Ok(avg_duration)
}

/// Benchmark configuration parsing for different formats
fn benchmark_parse_formats() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Benchmarking Configuration Parsing ===");

    // INI format
    run_benchmark("Parse INI", 100, || {
        let mut config = Config::new("bench");
        config.load_from_file(Path::new("bench_config.ini"))?;
        Ok(())
    })?;

    // TOML format
    run_benchmark("Parse TOML", 100, || {
        let mut config = Config::new("bench");
        config.load_from_file(Path::new("bench_config.toml"))?;
        Ok(())
    })?;

    // YAML format
    run_benchmark("Parse YAML", 100, || {
        let mut config = Config::new("bench");
        config.load_from_file(Path::new("bench_config.yaml"))?;
        Ok(())
    })?;

    // JSON format
    run_benchmark("Parse JSON", 100, || {
        let mut config = Config::new("bench");
        config.load_from_file(Path::new("bench_config.json"))?;
        Ok(())
    })?;

    Ok(())
}

/// Benchmark getting values from configuration
fn benchmark_get_values() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Benchmarking Configuration Value Retrieval ===");

    // Load a configuration for testing
    let mut config = Config::new("bench");
    config.load_from_file(Path::new("bench_config.ini"))?;

    // Simple get
    run_benchmark("Get Single Value", 10000, || {
        for _ in 0..100 {
            let _ = config.get("section5", "string_key2");
            let _ = config.get("section1", "int_key3");
            let _ = config.get("section8", "float_key4");
            let _ = config.get("section3", "bool_key1");
        }
        Ok(())
    })?;

    // Typed get methods
    run_benchmark("Get Typed Values", 10000, || {
        for _ in 0..100 {
            let _ = config.get_string("section5", "string_key2", None);
            let _ = config.get_integer("section1", "int_key3", None);
            let _ = config.get_float("section8", "float_key4", None);
            let _ = config.get_boolean("section3", "bool_key1", None);
        }
        Ok(())
    })?;

    // Access to nested sections
    run_benchmark("Read Many Values", 1000, || {
        for section_idx in [1, 3, 5, 8].iter() {
            for key_idx in 0..10 {
                let _ = config.get(&format!("section{}", section_idx), &format!("string_key{}", key_idx));
                let _ = config.get(&format!("section{}", section_idx), &format!("int_key{}", key_idx));
                let _ = config.get(&format!("section{}", section_idx), &format!("float_key{}", key_idx));
                let _ = config.get(&format!("section{}", section_idx), &format!("bool_key{}", key_idx));
            }
        }
        Ok(())
    })?;

    Ok(())
}

/// Benchmark setting values in configuration
fn benchmark_set_values() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Benchmarking Configuration Value Setting ===");

    // Create a new configuration for testing
    let mut config = Config::new("bench");

    // Simple set
    run_benchmark("Set Simple Values", 10000, || {
        config.set("benchmark", "string", ConfigValue::String("benchmark value".to_string()));
        config.set("benchmark", "int", ConfigValue::Integer(12345));
        config.set("benchmark", "float", ConfigValue::Float(3.14159));
        config.set("benchmark", "bool", ConfigValue::Boolean(true));
        Ok(())
    })?;

    // Set with complex values
    run_benchmark("Set Complex Values", 1000, || {
        // Array
        let array = vec![
            ConfigValue::String("value1".to_string()),
            ConfigValue::Integer(123),
            ConfigValue::Boolean(true)
        ];
        config.set("complex", "array", ConfigValue::Array(array));

        // Table
        let mut table = HashMap::new();
        table.insert("key1".to_string(), ConfigValue::String("value1".to_string()));
        table.insert("key2".to_string(), ConfigValue::Integer(456));
        table.insert("key3".to_string(), ConfigValue::Boolean(false));
        config.set("complex", "table", ConfigValue::Table(table));

        Ok(())
    })?;

    // Set multiple values
    run_benchmark("Set Multiple Values", 1000, || {
        for i in 0..10 {
            config.set("test_section", &format!("string{}", i),
                       ConfigValue::String(format!("value{}", i)));
            config.set("test_section", &format!("int{}", i),
                       ConfigValue::Integer(i as i64 * 100));
            config.set("test_section", &format!("float{}", i),
                       ConfigValue::Float(i as f64 / 10.0));
            config.set("test_section", &format!("bool{}", i),
                       ConfigValue::Boolean(i % 2 == 0));
        }
        Ok(())
    })?;

    Ok(())
}

/// Benchmark saving configurations in different formats
fn benchmark_save_formats() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Benchmarking Configuration Saving ===");

    // Create a test configuration with some data
    let mut config = create_benchmark_config();

    // INI format
    config.set_format(ConfigFormat::Ini);
    run_benchmark("Save INI", 100, || {
        config.save_to_file(Path::new("bench_save.ini"))?;
        Ok(())
    })?;

    // TOML format
    config.set_format(ConfigFormat::Toml);
    run_benchmark("Save TOML", 100, || {
        config.save_to_file(Path::new("bench_save.toml"))?;
        Ok(())
    })?;

    // YAML format
    config.set_format(ConfigFormat::Yaml);
    run_benchmark("Save YAML", 100, || {
        config.save_to_file(Path::new("bench_save.yaml"))?;
        Ok(())
    })?;

    // JSON format
    config.set_format(ConfigFormat::Json);
    run_benchmark("Save JSON", 100, || {
        config.save_to_file(Path::new("bench_save.json"))?;
        Ok(())
    })?;

    Ok(())
}

/// Creates a configuration with test data for benchmarking
fn create_benchmark_config() -> Config {
    let mut config = Config::new("bench");

    // Add sections with various types of data
    for section_idx in 0..10 {
        for key_idx in 0..10 {
            let section = format!("section{}", section_idx);

            // Add different types of values
            config.set(&section, &format!("string_key{}", key_idx),
                       ConfigValue::String(format!("string value {}{}", section_idx, key_idx)));

            config.set(&section, &format!("int_key{}", key_idx),
                       ConfigValue::Integer((section_idx * 1000 + key_idx) as i64));

            config.set(&section, &format!("float_key{}", key_idx),
                       ConfigValue::Float(section_idx as f64 + key_idx as f64 / 10.0));

            config.set(&section, &format!("bool_key{}", key_idx),
                       ConfigValue::Boolean((section_idx + key_idx) % 2 == 0));
        }
    }

    // Add some complex data

    // Arrays
    let string_array = vec![
        ConfigValue::String("value1".to_string()),
        ConfigValue::String("value2".to_string()),
        ConfigValue::String("value3".to_string()),
    ];
    config.set("arrays", "strings", ConfigValue::Array(string_array));

    let mixed_array = vec![
        ConfigValue::String("text".to_string()),
        ConfigValue::Integer(123),
        ConfigValue::Float(45.67),
        ConfigValue::Boolean(true),
    ];
    config.set("arrays", "mixed", ConfigValue::Array(mixed_array));

    // Tables
    let mut simple_table = HashMap::new();
    simple_table.insert("name".to_string(), ConfigValue::String("Test User".to_string()));
    simple_table.insert("age".to_string(), ConfigValue::Integer(30));
    simple_table.insert("active".to_string(), ConfigValue::Boolean(true));

    config.set("tables", "user", ConfigValue::Table(simple_table));

    // Nested table
    let mut address = HashMap::new();
    address.insert("street".to_string(), ConfigValue::String("123 Main St".to_string()));
    address.insert("city".to_string(), ConfigValue::String("Anytown".to_string()));
    address.insert("zip".to_string(), ConfigValue::String("12345".to_string()));

    let mut nested_table = HashMap::new();
    nested_table.insert("name".to_string(), ConfigValue::String("Test User".to_string()));
    nested_table.insert("address".to_string(), ConfigValue::Table(address));

    config.set("tables", "nested", ConfigValue::Table(nested_table));

    config
}

/// Create test configuration files in different formats
fn create_test_files() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating test configuration files...");

    let config = create_benchmark_config();

    // Save in different formats
    let mut ini_config = config.clone();
    ini_config.set_format(ConfigFormat::Ini);
    ini_config.save_to_file(Path::new("bench_config.ini"))?;

    let mut toml_config = config.clone();
    toml_config.set_format(ConfigFormat::Toml);
    toml_config.save_to_file(Path::new("bench_config.toml"))?;

    let mut yaml_config = config.clone();
    yaml_config.set_format(ConfigFormat::Yaml);
    yaml_config.save_to_file(Path::new("bench_config.yaml"))?;

    let mut json_config = config.clone();
    json_config.set_format(ConfigFormat::Json);
    json_config.save_to_file(Path::new("bench_config.json"))?;

    println!("Test files created.");

    Ok(())
}

/// Clean up benchmark test files
fn cleanup_files() -> Result<(), Box<dyn std::error::Error>> {
    println!("\nCleaning up test files...");

    let files = [
        "bench_config.ini",
        "bench_config.toml",
        "bench_config.yaml",
        "bench_config.json",
        "bench_save.ini",
        "bench_save.toml",
        "bench_save.yaml",
        "bench_save.json",
    ];

    for file in &files {
        if Path::new(file).exists() {
            fs::remove_file(file)?;
        }
    }

    println!("Test files cleaned up.");

    Ok(())
}