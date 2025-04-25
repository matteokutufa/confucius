//! Module for supported configuration formats.
//!
//! This module provides submodules for handling different configuration file formats,
//! including INI, TOML, YAML, and JSON. Each submodule contains functionality specific
//! to parsing, validating, and working with the respective format.

pub mod ini;  // Submodule for INI format handling.
pub mod toml; // Submodule for TOML format handling.
pub mod yaml; // Submodule for YAML format handling.
pub mod json; // Submodule for JSON format handling.