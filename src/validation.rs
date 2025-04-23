// src/validation.rs
//! Modulo per la validazione delle configurazioni

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use regex::Regex;

use crate::{Config, ConfigError, ConfigValue};

/// Tipi di dato supportati per la validazione
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Table,
    Any, // Accetta qualsiasi tipo
}

impl From<&ConfigValue> for ValueType {
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

/// Definizione di un campo nello schema di validazione
#[derive(Debug, Clone)]
pub struct FieldDefinition {
    /// Tipo di dato atteso
    pub value_type: ValueType,

    /// Indica se il campo è obbligatorio
    pub required: bool,

    /// Valore di default (opzionale)
    pub default_value: Option<ConfigValue>,

    /// Vincoli per il campo
    pub constraints: Vec<FieldConstraint>,

    /// Descrizione del campo (utile per la documentazione)
    pub description: Option<String>,
}

impl FieldDefinition {
    /// Crea una nuova definizione di campo
    pub fn new(value_type: ValueType) -> Self {
        FieldDefinition {
            value_type,
            required: false,
            default_value: None,
            constraints: Vec::new(),
            description: None,
        }
    }

    /// Imposta il campo come obbligatorio
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Imposta un valore di default
    pub fn default(mut self, value: ConfigValue) -> Self {
        self.default_value = Some(value);
        self
    }

    /// Aggiunge un vincolo al campo
    pub fn constraint(mut self, constraint: FieldConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Aggiunge una descrizione
    pub fn description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    /// Verifica se un valore rispetta la definizione del campo
    pub fn validate(&self, value: Option<&ConfigValue>, path: &str) -> Result<(), ValidationError> {
        // Se il valore è None ma è richiesto, errore
        if value.is_none() {
            if self.required {
                return Err(ValidationError::MissingField {
                    path: path.to_string(),
                });
            }
            // Se non è richiesto e manca, è ok
            return Ok(());
        }

        let value = value.unwrap();

        // Verifichiamo il tipo
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

        // Verifichiamo i vincoli
        for constraint in &self.constraints {
            constraint.validate(value, path)?;
        }

        Ok(())
    }
}

/// Wrapper per funzioni di validazione personalizzate

pub struct ValidateFn(Arc<dyn Fn(&ConfigValue) -> Result<(), String> + Send + Sync>);

impl ValidateFn {
    /// Crea una nuova funzione di validazione
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&ConfigValue) -> Result<(), String> + Send + Sync + 'static
    {
        ValidateFn(Arc::new(f))
    }

    /// Esegue la validazione su un valore
    pub fn validate(&self, value: &ConfigValue) -> Result<(), String> {
        (self.0)(value)
    }
}

// Ora l'implementazione di Clone è semplice
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

/// Vincoli personalizzati per i campi
#[derive(Debug, Clone)]
pub enum FieldConstraint {
    /// Vincolo per valori stringa
    String {
        /// Lunghezza minima (se specificata)
        min_length: Option<usize>,
        /// Lunghezza massima (se specificata)
        max_length: Option<usize>,
        /// Pattern regex (se specificato)
        pattern: Option<Regex>,
        /// Valori consentiti (se specificati)
        allowed_values: Option<Vec<String>>,
    },

    /// Vincolo per valori interi
    Integer {
        /// Valore minimo (se specificato)
        min: Option<i64>,
        /// Valore massimo (se specificato)
        max: Option<i64>,
        /// Valori consentiti (se specificati)
        allowed_values: Option<Vec<i64>>,
    },

    /// Vincolo per valori float
    Float {
        /// Valore minimo (se specificato)
        min: Option<f64>,
        /// Valore massimo (se specificato)
        max: Option<f64>,
    },

    /// Vincolo per array
    Array {
        /// Lunghezza minima (se specificata)
        min_length: Option<usize>,
        /// Lunghezza massima (se specificata)
        max_length: Option<usize>,
        /// Tipo degli elementi (se specificato)
        item_type: Option<Box<FieldDefinition>>,
    },

    /// Vincolo custom con funzione di validazione
    Custom {
        /// Funzione di validazione
        #[doc(hidden)]  // Nascondiamo questo campo nella documentazione generata
        validate_fn: ValidateFn,
        /// Descrizione del vincolo (per messaggi di errore)
        description: String,
    },
}

impl FieldConstraint {
    /// Crea un nuovo vincolo per stringhe
    pub fn string() -> Self {
        FieldConstraint::String {
            min_length: None,
            max_length: None,
            pattern: None,
            allowed_values: None,
        }
    }

    /// Imposta la lunghezza minima per un vincolo stringa
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

    /// Imposta la lunghezza massima per un vincolo stringa
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

    /// Imposta il pattern regex per un vincolo stringa
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

    /// Imposta i valori consentiti per un vincolo stringa
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

    /// Crea un nuovo vincolo per interi
    pub fn integer() -> Self {
        FieldConstraint::Integer {
            min: None,
            max: None,
            allowed_values: None,
        }
    }

    /// Imposta il valore minimo per un vincolo intero
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

    /// Imposta il valore massimo per un vincolo intero
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

    /// Imposta i valori consentiti per un vincolo intero
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

    /// Crea un nuovo vincolo per float
    pub fn float() -> Self {
        FieldConstraint::Float {
            min: None,
            max: None,
        }
    }

    /// Imposta il valore minimo per un vincolo float
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

    /// Imposta il valore massimo per un vincolo float
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

    /// Crea un nuovo vincolo per array
    pub fn array() -> Self {
        FieldConstraint::Array {
            min_length: None,
            max_length: None,
            item_type: None,
        }
    }

    /// Imposta il tipo degli elementi per un vincolo array
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

    /// Crea un nuovo vincolo personalizzato
    pub fn custom<F>(validate_fn: F, description: &str) -> Self
    where
        F: Fn(&ConfigValue) -> Result<(), String> + Send + Sync + 'static,
    {
        FieldConstraint::Custom {
            validate_fn: ValidateFn::new(validate_fn),
            description: description.to_string(),
        }
    }

    /// Valida un valore rispetto al vincolo
    pub fn validate(&self, value: &ConfigValue, path: &str) -> Result<(), ValidationError> {
        match self {
            FieldConstraint::String { min_length, max_length, pattern, allowed_values } => {
                if let ConfigValue::String(s) = value {
                    // Controllo lunghezza minima
                    if let Some(min) = min_length {
                        if s.len() < *min {
                            return Err(ValidationError::StringTooShort {
                                path: path.to_string(),
                                min: *min,
                                actual: s.len(),
                            });
                        }
                    }

                    // Controllo lunghezza massima
                    if let Some(max) = max_length {
                        if s.len() > *max {
                            return Err(ValidationError::StringTooLong {
                                path: path.to_string(),
                                max: *max,
                                actual: s.len(),
                            });
                        }
                    }

                    // Controllo pattern regex
                    if let Some(regex) = pattern {
                        if !regex.is_match(s) {
                            return Err(ValidationError::PatternMismatch {
                                path: path.to_string(),
                                pattern: regex.to_string(),
                                value: s.clone(),
                            });
                        }
                    }

                    // Controllo valori consentiti
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

            FieldConstraint::Integer { min, max, allowed_values } => {
                if let ConfigValue::Integer(i) = value {
                    // Controllo valore minimo
                    if let Some(min_val) = min {
                        if *i < *min_val {
                            return Err(ValidationError::IntegerTooSmall {
                                path: path.to_string(),
                                min: *min_val,
                                actual: *i,
                            });
                        }
                    }

                    // Controllo valore massimo
                    if let Some(max_val) = max {
                        if *i > *max_val {
                            return Err(ValidationError::IntegerTooLarge {
                                path: path.to_string(),
                                max: *max_val,
                                actual: *i,
                            });
                        }
                    }

                    // Controllo valori consentiti
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

            FieldConstraint::Float { min, max } => {
                if let ConfigValue::Float(f) = value {
                    // Controllo valore minimo
                    if let Some(min_val) = min {
                        if *f < *min_val {
                            return Err(ValidationError::FloatTooSmall {
                                path: path.to_string(),
                                min: *min_val,
                                actual: *f,
                            });
                        }
                    }

                    // Controllo valore massimo
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

            FieldConstraint::Array { min_length, max_length, item_type } => {
                if let ConfigValue::Array(arr) = value {
                    // Controllo lunghezza minima
                    if let Some(min) = min_length {
                        if arr.len() < *min {
                            return Err(ValidationError::ArrayTooShort {
                                path: path.to_string(),
                                min: *min,
                                actual: arr.len(),
                            });
                        }
                    }

                    // Controllo lunghezza massima
                    if let Some(max) = max_length {
                        if arr.len() > *max {
                            return Err(ValidationError::ArrayTooLong {
                                path: path.to_string(),
                                max: *max,
                                actual: arr.len(),
                            });
                        }
                    }

                    // Controllo tipo degli elementi
                    if let Some(item_def) = item_type {
                        for (i, item) in arr.iter().enumerate() {
                            let item_path = format!("{}[{}]", path, i);
                            item_def.validate(Some(item), &item_path)?;
                        }
                    }
                }
            },

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

/// Schema di validazione per una configurazione
#[derive(Debug, Clone)]
pub struct ValidationSchema {
    /// Definizioni dei campi per ogni sezione
    sections: HashMap<String, HashMap<String, FieldDefinition>>,

    /// Sezioni richieste nel file di configurazione
    required_sections: HashSet<String>,

    /// Indica se sono ammesse sezioni non definite nello schema
    allow_unknown_sections: bool,

    /// Indica se sono ammesse chiavi non definite nelle sezioni
    allow_unknown_keys: bool,
}

impl ValidationSchema {
    /// Crea un nuovo schema di validazione
    pub fn new() -> Self {
        ValidationSchema {
            sections: HashMap::new(),
            required_sections: HashSet::new(),
            allow_unknown_sections: true,
            allow_unknown_keys: true,
        }
    }

    /// Definisce una sezione nello schema
    pub fn section(&mut self, name: &str) -> &mut Self {
        self.sections.insert(name.to_string(), HashMap::new());
        self
    }

    /// Definisce una sezione obbligatoria nello schema
    pub fn required_section(&mut self, name: &str) -> &mut Self {
        self.section(name);
        self.required_sections.insert(name.to_string());
        self
    }

    /// Definisce un campo in una sezione
    pub fn field(&mut self, section: &str, key: &str, definition: FieldDefinition) -> &mut Self {
        // Assicuriamoci che la sezione esista
        if !self.sections.contains_key(section) {
            self.section(section);
        }

        // Aggiungiamo la definizione del campo
        if let Some(section_fields) = self.sections.get_mut(section) {
            section_fields.insert(key.to_string(), definition);
        }

        self
    }

    /// Configura se accettare sezioni non definite
    pub fn allow_unknown_sections(&mut self, allow: bool) -> &mut Self {
        self.allow_unknown_sections = allow;
        self
    }

    /// Configura se accettare chiavi non definite nelle sezioni
    pub fn allow_unknown_keys(&mut self, allow: bool) -> &mut Self {
        self.allow_unknown_keys = allow;
        self
    }

    /// Valida una configurazione rispetto allo schema
    pub fn validate(&self, config: &Config) -> Result<(), ValidationErrors> {
        let mut errors = Vec::new();

        // Verifichiamo le sezioni richieste
        for section in &self.required_sections {
            if !config.values.contains_key(section) {
                errors.push(ValidationError::MissingSection {
                    section: section.clone(),
                });
            }
        }

        // Verifichiamo ogni sezione presente nella configurazione
        for (section_name, section_values) in &config.values {
            // Se la sezione non è definita nello schema
            if !self.sections.contains_key(section_name) {
                if !self.allow_unknown_sections {
                    errors.push(ValidationError::UnknownSection {
                        section: section_name.clone(),
                    });
                }
                continue;
            }

            // Verifichiamo i campi della sezione
            if let Some(section_schema) = self.sections.get(section_name) {
                // Verifichiamo che tutti i campi obbligatori siano presenti
                for (field_name, field_def) in section_schema {
                    let field_path = format!("{}.{}", section_name, field_name);
                    let field_value = section_values.get(field_name);

                    if let Err(err) = field_def.validate(field_value, &field_path) {
                        errors.push(err);
                    }
                }

                // Verifichiamo che non ci siano campi non definiti (se necessario)
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

    /// Applica i valori di default ai campi mancanti
    pub fn apply_defaults(&self, config: &mut Config) {
        for (section_name, section_fields) in &self.sections {
            for (field_name, field_def) in section_fields {
                // Se il campo ha un valore di default e non è presente nella configurazione
                if let Some(default_value) = &field_def.default_value {
                    if !config.values.get(section_name).map_or(false, |s| s.contains_key(field_name)) {
                        // Aggiungiamo il valore di default
                        config.set(section_name, field_name, default_value.clone());
                    }
                }
            }
        }
    }
}

/// Errori di validazione
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Sezione mancante: {section}")]
    MissingSection {
        section: String,
    },

    #[error("Sezione sconosciuta: {section}")]
    UnknownSection {
        section: String,
    },

    #[error("Campo mancante: {path}")]
    MissingField {
        path: String,
    },

    #[error("Chiave sconosciuta: {section}.{key}")]
    UnknownKey {
        section: String,
        key: String,
    },

    #[error("Tipo non corrispondente per {path}: atteso {expected:?}, trovato {actual:?}")]
    TypeMismatch {
        path: String,
        expected: ValueType,
        actual: ValueType,
    },

    #[error("Stringa troppo corta per {path}: lunghezza minima {min}, attuale {actual}")]
    StringTooShort {
        path: String,
        min: usize,
        actual: usize,
    },

    #[error("Stringa troppo lunga per {path}: lunghezza massima {max}, attuale {actual}")]
    StringTooLong {
        path: String,
        max: usize,
        actual: usize,
    },

    #[error("Pattern non corrispondente per {path}: pattern {pattern}, valore {value}")]
    PatternMismatch {
        path: String,
        pattern: String,
        value: String,
    },

    #[error("Valore non valido per {path}: consentiti {allowed}, attuale {actual}")]
    InvalidValue {
        path: String,
        allowed: String,
        actual: String,
    },

    #[error("Intero troppo piccolo per {path}: minimo {min}, attuale {actual}")]
    IntegerTooSmall {
        path: String,
        min: i64,
        actual: i64,
    },

    #[error("Intero troppo grande per {path}: massimo {max}, attuale {actual}")]
    IntegerTooLarge {
        path: String,
        max: i64,
        actual: i64,
    },

    #[error("Valore intero non valido per {path}: consentiti {allowed}, attuale {actual}")]
    InvalidInteger {
        path: String,
        allowed: String,
        actual: i64,
    },

    #[error("Float troppo piccolo per {path}: minimo {min}, attuale {actual}")]
    FloatTooSmall {
        path: String,
        min: f64,
        actual: f64,
    },

    #[error("Float troppo grande per {path}: massimo {max}, attuale {actual}")]
    FloatTooLarge {
        path: String,
        max: f64,
        actual: f64,
    },

    #[error("Array troppo corto per {path}: lunghezza minima {min}, attuale {actual}")]
    ArrayTooShort {
        path: String,
        min: usize,
        actual: usize,
    },

    #[error("Array troppo lungo per {path}: lunghezza massima {max}, attuale {actual}")]
    ArrayTooLong {
        path: String,
        max: usize,
        actual: usize,
    },

    #[error("Vincolo personalizzato fallito per {path}: {description} - {message}")]
    CustomConstraintFailed {
        path: String,
        description: String,
        message: String,
    },
}

/// Collezione di errori di validazione
#[derive(Debug, thiserror::Error)]
#[error("Errori di validazione della configurazione:\n{}", self.format_errors())]
pub struct ValidationErrors(pub Vec<ValidationError>);

impl ValidationErrors {
    /// Formatta tutti gli errori in un'unica stringa
    fn format_errors(&self) -> String {
        self.0.iter()
            .enumerate()
            .map(|(i, err)| format!("{}. {}", i + 1, err))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Estensione del trait Config per supportare la validazione
pub trait ValidationExt {
    /// Valida la configurazione rispetto a uno schema
    fn validate(&self, schema: &ValidationSchema) -> Result<(), ValidationErrors>;

    /// Applica i valori di default dello schema alla configurazione
    fn apply_defaults(&mut self, schema: &ValidationSchema);

    /// Valida e applica i valori di default in un'unica operazione
    fn validate_and_apply_defaults(&mut self, schema: &ValidationSchema) -> Result<(), ValidationErrors>;
}

impl ValidationExt for Config {
    fn validate(&self, schema: &ValidationSchema) -> Result<(), ValidationErrors> {
        schema.validate(self)
    }

    fn apply_defaults(&mut self, schema: &ValidationSchema) {
        schema.apply_defaults(self)
    }

    fn validate_and_apply_defaults(&mut self, schema: &ValidationSchema) -> Result<(), ValidationErrors> {
        // Prima applichiamo i default
        self.apply_defaults(schema);

        // Poi validiamo
        self.validate(schema)
    }
}

// Estendiamo l'enum ConfigError per includere gli errori di validazione
impl From<ValidationErrors> for ConfigError {
    fn from(errors: ValidationErrors) -> Self {
        ConfigError::Generic(format!("Errori di validazione: {}", errors))
    }
}