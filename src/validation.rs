//! Module for configuration validation
//!
//! This module provides structures and functions to define validation schemas,
//! validate configuration files, and apply default values. It supports various
//! data types, constraints, and custom validation logic.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use regex::Regex;

use crate::{Config, ConfigError, ConfigValue};

/// Supported data types for validation
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    /// String type
    String,
    /// Integer type
    Integer,
    /// Float type
    Float,
    /// Boolean type
    Boolean,
    /// Array type
    Array,
    /// Table type
    Table,
    /// Accepts any type
    Any,
}

impl From<&ConfigValue> for ValueType {
    /// Converts a `ConfigValue` to its corresponding `ValueType`.
    fn from(value: &ConfigValue) -> Self {
        match value {
            ConfigValue::String(_) => ValueType::String,
            ConfigValue::Integer(_) => ValueType::Integer,
            ConfigValue::Float(_) => ValueType::Float,
            ConfigValue::Boolean(_) => ValueType::Boolean,
            ConfigValue::Array(_) => ValueType::Array,
            ConfigValue::Table(_) => ValueType::Table,
        }
    }
}

/// Definition of a field in the validation schema
#[derive(Debug, Clone)]
pub struct FieldDefinition {
    /// Expected data type
    pub value_type: ValueType,
    /// Indicates if the field is required
    pub required: bool,
    /// Default value (optional)
    pub default_value: Option<ConfigValue>,
    /// Constraints for the field
    pub constraints: Vec<FieldConstraint>,
    /// Field description (useful for documentation)
    pub description: Option<String>,
}

impl FieldDefinition {
    /// Creates a new field definition
    pub fn new(value_type: ValueType) -> Self {
        FieldDefinition {
            value_type,
            required: false,
            default_value: None,
            constraints: Vec::new(),
            description: None,
        }
    }

    /// Marks the field as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Sets a default value for the field
    pub fn default(mut self, value: ConfigValue) -> Self {
        self.default_value = Some(value);
        self
    }

    /// Adds a constraint to the field
    pub fn constraint(mut self, constraint: FieldConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Adds a description to the field
    pub fn description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    /// Validates a value against the field definition
    ///
    /// # Arguments
    ///
    /// * `value` - The value to validate.
    /// * `path` - The path of the field in the configuration.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the value is valid.
    /// * `Err(ValidationError)` - If the value is invalid.
    pub fn validate(&self, value: Option<&ConfigValue>, path: &str) -> Result<(), ValidationError> {
        if value.is_none() {
            if self.required {
                return Err(ValidationError::MissingField {
                    path: path.to_string(),
                });
            }
            return Ok(());
        }

        let value = value.unwrap();

        if self.value_type != ValueType::Any {
            let actual_type = ValueType::from(value);
            if actual_type != self.value_type {
                return Err(ValidationError::TypeMismatch {
                    path: path.to_string(),
                    expected: self.value_type.clone(),
                    actual: actual_type,
                });
            }
        }

        for constraint in &self.constraints {
            constraint.validate(value, path)?;
        }

        Ok(())
    }
}

/// Wrapper for custom validation functions
pub struct ValidateFn(Arc<dyn Fn(&ConfigValue) -> Result<(), String> + Send + Sync>);

impl ValidateFn {
    /// Creates a new custom validation function
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&ConfigValue) -> Result<(), String> + Send + Sync + 'static,
    {
        ValidateFn(Arc::new(f))
    }

    /// Executes the validation function on a value
    pub fn validate(&self, value: &ConfigValue) -> Result<(), String> {
        (self.0)(value)
    }
}

impl Clone for ValidateFn {
    fn clone(&self) -> Self {
        ValidateFn(Arc::clone(&self.0))
    }
}

impl std::fmt::Debug for ValidateFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ValidateFn")
    }
}

/// Custom constraints for fields
#[derive(Debug, Clone)]
pub enum FieldConstraint {
    /// Constraint for string values
    String {
        /// Minimum length (if specified)
        min_length: Option<usize>,
        /// Maximum length (if specified)
        max_length: Option<usize>,
        /// Regex pattern (if specified)
        pattern: Option<Regex>,
        /// Allowed values (if specified)
        allowed_values: Option<Vec<String>>,
    },
    /// Constraint for integer values
    Integer {
        /// Minimum value (if specified)
        min: Option<i64>,
        /// Maximum value (if specified)
        max: Option<i64>,
        /// Allowed values (if specified)
        allowed_values: Option<Vec<i64>>,
    },
    /// Constraint for float values
    Float {
        /// Minimum value (if specified)
        min: Option<f64>,
        /// Maximum value (if specified)
        max: Option<f64>,
    },
    /// Constraint for arrays
    Array {
        /// Minimum length (if specified)
        min_length: Option<usize>,
        /// Maximum length (if specified)
        max_length: Option<usize>,
        /// Type of elements (if specified)
        item_type: Option<Box<FieldDefinition>>,
    },
    /// Custom constraint with a validation function
    Custom {
        /// Validation function
        #[doc(hidden)]
        validate_fn: ValidateFn,
        /// Description of the constraint (for error messages)
        description: String,
    },
}

impl FieldConstraint {
    /// Creates a new string constraint
    pub fn string() -> Self {
        FieldConstraint::String {
            min_length: None,
            max_length: None,
            pattern: None,
            allowed_values: None,
        }
    }

    /// Sets the minimum length for a string constraint
    pub fn min_length(self, min: usize) -> Self {
        match self {
            FieldConstraint::String { max_length, pattern, allowed_values, .. } => {
                FieldConstraint::String {
                    min_length: Some(min),
                    max_length,
                    pattern,
                    allowed_values,
                }
            },
            FieldConstraint::Array { max_length, item_type, .. } => {
                FieldConstraint::Array {
                    min_length: Some(min),
                    max_length,
                    item_type,
                }
            },
            _ => self,
        }
    }

    /// Sets the maximum length for a string constraint
    pub fn max_length(self, max: usize) -> Self {
        match self {
            FieldConstraint::String { min_length, pattern, allowed_values, .. } => {
                FieldConstraint::String {
                    min_length,
                    max_length: Some(max),
                    pattern,
                    allowed_values,
                }
            },
            FieldConstraint::Array { min_length, item_type, .. } => {
                FieldConstraint::Array {
                    min_length,
                    max_length: Some(max),
                    item_type,
                }
            },
            _ => self,
        }
    }

    /// Sets the regex pattern for a string constraint
    pub fn pattern(self, pattern: &str) -> Self {
        match self {
            FieldConstraint::String { min_length, max_length, allowed_values, .. } => {
                FieldConstraint::String {
                    min_length,
                    max_length,
                    pattern: Some(Regex::new(pattern).unwrap()),
                    allowed_values,
                }
            },
            _ => self,
        }
    }

    /// Sets the allowed string values for a string constraint
    pub fn allowed_string_values(self, values: Vec<&str>) -> Self {
        match self {
            FieldConstraint::String { min_length, max_length, pattern, .. } => {
                FieldConstraint::String {
                    min_length,
                    max_length,
                    pattern,
                    allowed_values: Some(values.iter().map(|s| s.to_string()).collect()),
                }
            },
            _ => self,
        }
    }

    /// Creates a new integer constraint
    pub fn integer() -> Self {
        FieldConstraint::Integer {
            min: None,
            max: None,
            allowed_values: None,
        }
    }

    /// Sets the minimum value for an integer constraint
    pub fn min_int(self, min: i64) -> Self {
        match self {
            FieldConstraint::Integer { max, allowed_values, .. } => {
                FieldConstraint::Integer {
                    min: Some(min),
                    max,
                    allowed_values,
                }
            },
            _ => self,
        }
    }

    /// Sets the maximum value for an integer constraint
    pub fn max_int(self, max: i64) -> Self {
        match self {
            FieldConstraint::Integer { min, allowed_values, .. } => {
                FieldConstraint::Integer {
                    min,
                    max: Some(max),
                    allowed_values,
                }
            },
            _ => self,
        }
    }

    /// Sets the allowed integer values for an integer constraint
    pub fn allowed_int_values(self, values: Vec<i64>) -> Self {
        match self {
            FieldConstraint::Integer { min, max, .. } => {
                FieldConstraint::Integer {
                    min,
                    max,
                    allowed_values: Some(values),
                }
            },
            _ => self,
        }
    }

    /// Creates a new float constraint
    pub fn float() -> Self {
        FieldConstraint::Float {
            min: None,
            max: None,
        }
    }

    /// Sets the minimum value for a float constraint
    pub fn min_float(self, min: f64) -> Self {
        match self {
            FieldConstraint::Float { max, .. } => {
                FieldConstraint::Float {
                    min: Some(min),
                    max,
                }
            },
            _ => self,
        }
    }

    /// Sets the maximum value for a float constraint
    pub fn max_float(self, max: f64) -> Self {
        match self {
            FieldConstraint::Float { min, .. } => {
                FieldConstraint::Float {
                    min,
                    max: Some(max),
                }
            },
            _ => self,
        }
    }

    /// Creates a new array constraint
    pub fn array() -> Self {
        FieldConstraint::Array {
            min_length: None,
            max_length: None,
            item_type: None,
        }
    }

    /// Sets the type of elements for an array constraint
    pub fn item_type(self, item_def: FieldDefinition) -> Self {
        match self {
            FieldConstraint::Array { min_length, max_length, .. } => {
                FieldConstraint::Array {
                    min_length,
                    max_length,
                    item_type: Some(Box::new(item_def)),
                }
            },
            _ => self,
        }
    }

    /// Creates a new custom constraint
    pub fn custom<F>(validate_fn: F, description: &str) -> Self
    where
        F: Fn(&ConfigValue) -> Result<(), String> + Send + Sync + 'static,
    {
        FieldConstraint::Custom {
            validate_fn: ValidateFn::new(validate_fn),
            description: description.to_string(),
        }
    }

    /// Validates a value against the constraint.
    ///
    /// This method checks if a given `ConfigValue` satisfies the conditions defined
    /// by the `FieldConstraint`. It performs type-specific validation based on the
    /// constraint type (e.g., string, integer, float, array, or custom).
    ///
    /// # Arguments
    ///
    /// * `value` - A reference to the `ConfigValue` to validate.
    /// * `path` - A string slice representing the path of the field in the configuration.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the value satisfies the constraint.
    /// * `Err(ValidationError)` - If the value violates the constraint.
    pub fn validate(&self, value: &ConfigValue, path: &str) -> Result<(), ValidationError> {
        match self {
            // Validation for string constraints
            FieldConstraint::String { min_length, max_length, pattern, allowed_values } => {
                if let ConfigValue::String(s) = value {
                    // Check minimum length
                    if let Some(min) = min_length {
                        if s.len() < *min {
                            return Err(ValidationError::StringTooShort {
                                path: path.to_string(),
                                min: *min,
                                actual: s.len(),
                            });
                        }
                    }

                    // Check maximum length
                    if let Some(max) = max_length {
                        if s.len() > *max {
                            return Err(ValidationError::StringTooLong {
                                path: path.to_string(),
                                max: *max,
                                actual: s.len(),
                            });
                        }
                    }

                    // Check regex pattern
                    if let Some(regex) = pattern {
                        if !regex.is_match(s) {
                            return Err(ValidationError::PatternMismatch {
                                path: path.to_string(),
                                pattern: regex.to_string(),
                                value: s.clone(),
                            });
                        }
                    }

                    // Check allowed values
                    if let Some(allowed) = allowed_values {
                        if !allowed.contains(s) {
                            return Err(ValidationError::InvalidValue {
                                path: path.to_string(),
                                allowed: format!("{:?}", allowed),
                                actual: s.clone(),
                            });
                        }
                    }
                }
            },

            // Validation for integer constraints
            FieldConstraint::Integer { min, max, allowed_values } => {
                if let ConfigValue::Integer(i) = value {
                    // Check minimum value
                    if let Some(min_val) = min {
                        if *i < *min_val {
                            return Err(ValidationError::IntegerTooSmall {
                                path: path.to_string(),
                                min: *min_val,
                                actual: *i,
                            });
                        }
                    }

                    // Check maximum value
                    if let Some(max_val) = max {
                        if *i > *max_val {
                            return Err(ValidationError::IntegerTooLarge {
                                path: path.to_string(),
                                max: *max_val,
                                actual: *i,
                            });
                        }
                    }

                    // Check allowed values
                    if let Some(allowed) = allowed_values {
                        if !allowed.contains(i) {
                            return Err(ValidationError::InvalidInteger {
                                path: path.to_string(),
                                allowed: format!("{:?}", allowed),
                                actual: *i,
                            });
                        }
                    }
                }
            },

            // Validation for float constraints
            FieldConstraint::Float { min, max } => {
                if let ConfigValue::Float(f) = value {
                    // Check minimum value
                    if let Some(min_val) = min {
                        if *f < *min_val {
                            return Err(ValidationError::FloatTooSmall {
                                path: path.to_string(),
                                min: *min_val,
                                actual: *f,
                            });
                        }
                    }

                    // Check maximum value
                    if let Some(max_val) = max {
                        if *f > *max_val {
                            return Err(ValidationError::FloatTooLarge {
                                path: path.to_string(),
                                max: *max_val,
                                actual: *f,
                            });
                        }
                    }
                }
            },

            // Validation for array constraints
            FieldConstraint::Array { min_length, max_length, item_type } => {
                if let ConfigValue::Array(arr) = value {
                    // Check minimum length
                    if let Some(min) = min_length {
                        if arr.len() < *min {
                            return Err(ValidationError::ArrayTooShort {
                                path: path.to_string(),
                                min: *min,
                                actual: arr.len(),
                            });
                        }
                    }

                    // Check maximum length
                    if let Some(max) = max_length {
                        if arr.len() > *max {
                            return Err(ValidationError::ArrayTooLong {
                                path: path.to_string(),
                                max: *max,
                                actual: arr.len(),
                            });
                        }
                    }

                    // Validate each item in the array
                    if let Some(item_def) = item_type {
                        for (i, item) in arr.iter().enumerate() {
                            let item_path = format!("{}[{}]", path, i);
                            item_def.validate(Some(item), &item_path)?;
                        }
                    }
                }
            },

            // Validation for custom constraints
            FieldConstraint::Custom { validate_fn, description } => {
                if let Err(msg) = validate_fn.validate(value) {
                    return Err(ValidationError::CustomConstraintFailed {
                        path: path.to_string(),
                        description: description.clone(),
                        message: msg,
                    });
                }
            },
        }

        Ok(())
    }
}

/// Validation schema for a configuration.
///
/// This structure defines the schema for validating configuration files. It includes
/// definitions for sections, required sections, and rules for handling unknown sections
/// and keys.
#[derive(Debug, Clone)]
pub struct ValidationSchema {
    /// Field definitions for each section.
    sections: HashMap<String, HashMap<String, FieldDefinition>>,

    /// Required sections in the configuration file.
    required_sections: HashSet<String>,

    /// Indicates whether undefined sections are allowed in the schema.
    allow_unknown_sections: bool,

    /// Indicates whether undefined keys are allowed in sections.
    allow_unknown_keys: bool,
}

impl ValidationSchema {
    /// Creates a new validation schema.
    ///
    /// # Returns
    ///
    /// A new instance of `ValidationSchema` with default settings.
    pub fn new() -> Self {
        ValidationSchema {
            sections: HashMap::new(),
            required_sections: HashSet::new(),
            allow_unknown_sections: true,
            allow_unknown_keys: true,
        }
    }

    /// Defines a section in the schema.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the section to define.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `ValidationSchema` instance for method chaining.
    pub fn section(&mut self, name: &str) -> &mut Self {
        self.sections.insert(name.to_string(), HashMap::new());
        self
    }

    /// Defines a required section in the schema.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the required section.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `ValidationSchema` instance for method chaining.
    pub fn required_section(&mut self, name: &str) -> &mut Self {
        self.section(name);
        self.required_sections.insert(name.to_string());
        self
    }

    /// Defines a field in a section.
    ///
    /// # Arguments
    ///
    /// * `section` - The name of the section where the field is defined.
    /// * `key` - The name of the field.
    /// * `definition` - The definition of the field.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `ValidationSchema` instance for method chaining.
    pub fn field(&mut self, section: &str, key: &str, definition: FieldDefinition) -> &mut Self {
        // Ensure the section exists.
        if !self.sections.contains_key(section) {
            self.section(section);
        }

        // Add the field definition.
        if let Some(section_fields) = self.sections.get_mut(section) {
            section_fields.insert(key.to_string(), definition);
        }

        self
    }

    /// Configures whether undefined sections are allowed.
    ///
    /// # Arguments
    ///
    /// * `allow` - A boolean indicating whether to allow undefined sections.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `ValidationSchema` instance for method chaining.
    pub fn allow_unknown_sections(&mut self, allow: bool) -> &mut Self {
        self.allow_unknown_sections = allow;
        self
    }

    /// Configures whether undefined keys are allowed in sections.
    ///
    /// # Arguments
    ///
    /// * `allow` - A boolean indicating whether to allow undefined keys.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `ValidationSchema` instance for method chaining.
    pub fn allow_unknown_keys(&mut self, allow: bool) -> &mut Self {
        self.allow_unknown_keys = allow;
        self
    }

    /// Validates a configuration against the schema.
    ///
    /// # Arguments
    ///
    /// * `config` - A reference to the `Config` instance to validate.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the configuration is valid.
    /// * `Err(ValidationErrors)` - If validation errors are found.
    pub fn validate(&self, config: &Config) -> Result<(), ValidationErrors> {
        let mut errors = Vec::new();

        // Check required sections.
        for section in &self.required_sections {
            if !config.values.contains_key(section) {
                errors.push(ValidationError::MissingSection {
                    section: section.clone(),
                });
            }
        }

        // Validate each section in the configuration.
        for (section_name, section_values) in &config.values {
            // Handle undefined sections.
            if !self.sections.contains_key(section_name) {
                if !self.allow_unknown_sections {
                    errors.push(ValidationError::UnknownSection {
                        section: section_name.clone(),
                    });
                }
                continue;
            }

            // Validate fields in the section.
            if let Some(section_schema) = self.sections.get(section_name) {
                // Check for required fields.
                for (field_name, field_def) in section_schema {
                    let field_path = format!("{}.{}", section_name, field_name);
                    let field_value = section_values.get(field_name);

                    if let Err(err) = field_def.validate(field_value, &field_path) {
                        errors.push(err);
                    }
                }

                // Check for undefined keys if necessary.
                if !self.allow_unknown_keys {
                    for key in section_values.keys() {
                        if !section_schema.contains_key(key) {
                            errors.push(ValidationError::UnknownKey {
                                section: section_name.clone(),
                                key: key.clone(),
                            });
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationErrors(errors))
        }
    }


    /// Applies default values to missing fields in the configuration.
    ///
    /// This method iterates through the schema's sections and fields, checking if each field
    /// has a default value and is missing in the provided configuration. If so, it sets the
    /// default value in the configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - A mutable reference to the `Config` instance where default values will be applied.
    pub fn apply_defaults(&self, config: &mut Config) {
        for (section_name, section_fields) in &self.sections {
            for (field_name, field_def) in section_fields {
                // If the field has a default value and is not present in the configuration
                if let Some(default_value) = &field_def.default_value {
                    if !config.values.get(section_name).map_or(false, |s| s.contains_key(field_name)) {
                        // Add the default value
                        config.set(section_name, field_name, default_value.clone());
                    }
                }
            }
        }
    }
}

/// Validation errors.
///
/// This enum represents the various types of validation errors that can occur
/// when validating a configuration. Each variant provides specific details
/// about the error encountered.
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    /// Error for a missing section in the configuration.
    ///
    /// # Fields
    /// * `section` - The name of the missing section.
    #[error("Sezione mancante: {section}")]
    MissingSection {
        section: String,
    },

    /// Error for an unknown section in the configuration.
    ///
    /// # Fields
    /// * `section` - The name of the unknown section.
    #[error("Sezione sconosciuta: {section}")]
    UnknownSection {
        section: String,
    },

    /// Error for a missing field in the configuration.
    ///
    /// # Fields
    /// * `path` - The path of the missing field.
    #[error("Campo mancante: {path}")]
    MissingField {
        path: String,
    },

    /// Error for an unknown key in a section.
    ///
    /// # Fields
    /// * `section` - The name of the section containing the unknown key.
    /// * `key` - The name of the unknown key.
    #[error("Chiave sconosciuta: {section}.{key}")]
    UnknownKey {
        section: String,
        key: String,
    },

    /// Error for a type mismatch in a field.
    ///
    /// # Fields
    /// * `path` - The path of the field.
    /// * `expected` - The expected value type.
    /// * `actual` - The actual value type.
    #[error("Tipo non corrispondente per {path}: atteso {expected:?}, trovato {actual:?}")]
    TypeMismatch {
        path: String,
        expected: ValueType,
        actual: ValueType,
    },

    /// Error for a string that is too short.
    ///
    /// # Fields
    /// * `path` - The path of the field.
    /// * `min` - The minimum allowed length.
    /// * `actual` - The actual length of the string.
    #[error("Stringa troppo corta per {path}: lunghezza minima {min}, attuale {actual}")]
    StringTooShort {
        path: String,
        min: usize,
        actual: usize,
    },

    /// Error for a string that is too long.
    ///
    /// # Fields
    /// * `path` - The path of the field.
    /// * `max` - The maximum allowed length.
    /// * `actual` - The actual length of the string.
    #[error("Stringa troppo lunga per {path}: lunghezza massima {max}, attuale {actual}")]
    StringTooLong {
        path: String,
        max: usize,
        actual: usize,
    },

    /// Error for a string that does not match a regex pattern.
    ///
    /// # Fields
    /// * `path` - The path of the field.
    /// * `pattern` - The regex pattern.
    /// * `value` - The actual string value.
    #[error("Pattern non corrispondente per {path}: pattern {pattern}, valore {value}")]
    PatternMismatch {
        path: String,
        pattern: String,
        value: String,
    },

    /// Error for an invalid value in a field.
    ///
    /// # Fields
    /// * `path` - The path of the field.
    /// * `allowed` - The allowed values.
    /// * `actual` - The actual value.
    #[error("Valore non valido per {path}: consentiti {allowed}, attuale {actual}")]
    InvalidValue {
        path: String,
        allowed: String,
        actual: String,
    },

    /// Error for an integer that is too small.
    ///
    /// # Fields
    /// * `path` - The path of the field.
    /// * `min` - The minimum allowed value.
    /// * `actual` - The actual value.
    #[error("Intero troppo piccolo per {path}: minimo {min}, attuale {actual}")]
    IntegerTooSmall {
        path: String,
        min: i64,
        actual: i64,
    },

    /// Error for an integer that is too large.
    ///
    /// # Fields
    /// * `path` - The path of the field.
    /// * `max` - The maximum allowed value.
    /// * `actual` - The actual value.
    #[error("Intero troppo grande per {path}: massimo {max}, attuale {actual}")]
    IntegerTooLarge {
        path: String,
        max: i64,
        actual: i64,
    },

    /// Error for an invalid integer value.
    ///
    /// # Fields
    /// * `path` - The path of the field.
    /// * `allowed` - The allowed values.
    /// * `actual` - The actual value.
    #[error("Valore intero non valido per {path}: consentiti {allowed}, attuale {actual}")]
    InvalidInteger {
        path: String,
        allowed: String,
        actual: i64,
    },

    /// Error for a float that is too small.
    ///
    /// # Fields
    /// * `path` - The path of the field.
    /// * `min` - The minimum allowed value.
    /// * `actual` - The actual value.
    #[error("Float troppo piccolo per {path}: minimo {min}, attuale {actual}")]
    FloatTooSmall {
        path: String,
        min: f64,
        actual: f64,
    },

    /// Error for a float that is too large.
    ///
    /// # Fields
    /// * `path` - The path of the field.
    /// * `max` - The maximum allowed value.
    /// * `actual` - The actual value.
    #[error("Float troppo grande per {path}: massimo {max}, attuale {actual}")]
    FloatTooLarge {
        path: String,
        max: f64,
        actual: f64,
    },

    /// Error for an array that is too short.
    ///
    /// # Fields
    /// * `path` - The path of the field.
    /// * `min` - The minimum allowed length.
    /// * `actual` - The actual length of the array.
    #[error("Array troppo corto per {path}: lunghezza minima {min}, attuale {actual}")]
    ArrayTooShort {
        path: String,
        min: usize,
        actual: usize,
    },

    /// Error for an array that is too long.
    ///
    /// # Fields
    /// * `path` - The path of the field.
    /// * `max` - The maximum allowed length.
    /// * `actual` - The actual length of the array.
    #[error("Array troppo lungo per {path}: lunghezza massima {max}, attuale {actual}")]
    ArrayTooLong {
        path: String,
        max: usize,
        actual: usize,
    },

    /// Error for a custom constraint that failed.
    ///
    /// # Fields
    /// * `path` - The path of the field.
    /// * `description` - A description of the constraint.
    /// * `message` - The error message from the custom validation function.
    #[error("Vincolo personalizzato fallito per {path}: {description} - {message}")]
    CustomConstraintFailed {
        path: String,
        description: String,
        message: String,
    },
}

/// Collection of validation errors.
///
/// This structure wraps a vector of `ValidationError` instances and provides
/// functionality to format them into a single string for easier debugging and
/// error reporting.
#[derive(Debug, thiserror::Error)]
#[error("Configuration validation errors:\n{}", self.format_errors())]
pub struct ValidationErrors(pub Vec<ValidationError>);

impl ValidationErrors {
    /// Formats all validation errors into a single string.
    ///
    /// This method iterates over the list of validation errors, enumerates them,
    /// and concatenates their string representations into a single formatted string.
    ///
    /// # Returns
    ///
    /// A string containing all validation errors, each on a new line.
    fn format_errors(&self) -> String {
        self.0.iter()
            .enumerate()
            .map(|(i, err)| format!("{}. {}", i + 1, err))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Extension trait for `Config` to support validation.
///
/// This trait provides methods to validate a configuration against a schema,
/// apply default values from the schema, and perform both operations in a single step.
pub trait ValidationExt {
    /// Validates the configuration against a schema.
    ///
    /// # Arguments
    ///
    /// * `schema` - A reference to the `ValidationSchema` to validate against.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the configuration is valid.
    /// * `Err(ValidationErrors)` - If validation errors are found.
    fn validate(&self, schema: &ValidationSchema) -> Result<(), ValidationErrors>;

    /// Applies default values from the schema to the configuration.
    ///
    /// # Arguments
    ///
    /// * `schema` - A reference to the `ValidationSchema` containing default values.
    fn apply_defaults(&mut self, schema: &ValidationSchema);

    /// Validates the configuration and applies default values in one operation.
    ///
    /// # Arguments
    ///
    /// * `schema` - A reference to the `ValidationSchema` to validate against and apply defaults from.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the configuration is valid after applying defaults.
    /// * `Err(ValidationErrors)` - If validation errors are found.
    fn validate_and_apply_defaults(&mut self, schema: &ValidationSchema) -> Result<(), ValidationErrors>;
}

impl ValidationExt for Config {
    /// Validates the configuration against a schema.
    fn validate(&self, schema: &ValidationSchema) -> Result<(), ValidationErrors> {
        schema.validate(self)
    }

    /// Applies default values from the schema to the configuration.
    fn apply_defaults(&mut self, schema: &ValidationSchema) {
        schema.apply_defaults(self)
    }

    /// Validates the configuration and applies default values in one operation.
    fn validate_and_apply_defaults(&mut self, schema: &ValidationSchema) -> Result<(), ValidationErrors> {
        // First, apply default values.
        self.apply_defaults(schema);

        // Then, validate the configuration.
        self.validate(schema)
    }
}

/// Extends the `ConfigError` enum to include validation errors.
///
/// This implementation allows `ValidationErrors` to be converted into a `ConfigError`
/// for unified error handling.
impl From<ValidationErrors> for ConfigError {
    /// Converts `ValidationErrors` into a `ConfigError`.
    ///
    /// # Arguments
    ///
    /// * `errors` - The `ValidationErrors` to convert.
    ///
    /// # Returns
    ///
    /// A `ConfigError` containing the formatted validation errors.
    fn from(errors: ValidationErrors) -> Self {
        ConfigError::Generic(format!("Validation errors: {}", errors))
    }
}