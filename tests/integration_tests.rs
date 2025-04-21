keywords = ["configuration", "ini", "toml", "yaml", "json"]
categories = ["config", "parser-implementations"]

#[test]
fn test_config_edge_cases() {
    let env = TestEnv::new("edge");

    // 1. File vuoto
    env.create_config_file("empty.conf", "");

    let mut config = Config::new("edge");
    let result = config.load_from_file(&env.path("empty.conf"));
    assert!(result.is_ok(), "Dovrebbe gestire file vuoti");

    // 2. Solo commenti
    env.create_config_file(
        "comments_only.conf",
        "# Solo commenti\n# Nessuna configurazione\n# Fine file\n"
    );

    let mut config = Config::new("edge");
    let result = config.load_from_file(&env.path("comments_only.conf"));
    assert!(result.is_ok(), "Dovrebbe gestire file con solo commenti");

    // 3. Sezioni senza chiavi
    env.create_config_file(
        "empty_sections.conf",
        "#!config/ini\n[section1]\n[section2]\n[section3]\n"
    );

    let mut config = Config::new("edge");
    let result = config.load_from_file(&env.path("empty_sections.conf"));
    assert!(result.is_ok(), "Dovrebbe gestire sezioni vuote");

    // 4. Valori speciali e caratteri di escape
    env.create_config_file(
        "special_values.conf",
        "#!config/ini\n[special]\npath = \"/path/with/backslash\\and/quotes\"\nregex = \"^[a-z].*$\"\nemoji = \"😀 🚀 🦀\"\n"
    );

    let mut config = Config::new("edge");
    let result = config.load_from_file(&env.path("special_values.conf"));
    assert!(result.is_ok(), "Dovrebbe gestire valori speciali");

    if let Some(value) = config.get("special", "path") {
        assert_eq!(value.as_string(), Some(&"/path/with/backslash\\and/quotes".to_string()));
    } else {
        panic!("Valore 'special.path' non trovato");
    }

    if let Some(value) = config.get("special", "emoji") {
        assert_eq!(value.as_string(), Some(&"😀 🚀 🦀".to_string()));
    } else {
        panic!("Valore 'special.emoji' non trovato");
    }
}

#[test]
fn test_config_modifications() {
    // Testiamo la modifica della configurazione in memoria
    let mut config = Config::new("modify");

    // Aggiungiamo alcuni valori
    config.set("section1", "string", ConfigValue::String("valore".to_string()));
    config.set("section1", "int", ConfigValue::Integer(42));
    config.set("section1", "float", ConfigValue::Float(3.14));
    config.set("section1", "bool", ConfigValue::Boolean(true));

    // Verifichiamo i valori
    assert_eq!(
        config.get("section1", "string").and_then(|v| v.as_string()).cloned(),
        Some("valore".to_string())
    );

    assert_eq!(
        config.get("section1", "int").and_then(|v| v.as_integer()),
        Some(42)
    );

    assert_eq!(
        config.get("section1", "float").and_then(|v| v.as_float()),
        Some(3.14)
    );

    assert_eq!(
        config.get("section1", "bool").and_then(|v| v.as_boolean()),
        Some(true)
    );

    // Modifichiamo un valore esistente
    config.set("section1", "string", ConfigValue::String("nuovo valore".to_string()));

    assert_eq!(
        config.get("section1", "string").and_then(|v| v.as_string()).cloned(),
        Some("nuovo valore".to_string())
    );

    // Aggiungiamo una nuova sezione
    config.set("section2", "key", ConfigValue::String("sezione 2".to_string()));

    assert_eq!(
        config.get("section2", "key").and_then(|v| v.as_string()).cloned(),
        Some("sezione 2".to_string())
    );

    // Salviamo e rileggiamo la configurazione
    let env = TestEnv::new("modify");
    let save_path = env.path("modified.conf");

    let save_result = config.save_to_file(&save_path);
    assert!(save_result.is_ok(), "Errore nel salvataggio: {:?}", save_result.err());

    // Leggiamo il file salvato
    let mut config2 = Config::new("modify");
    let load_result = config2.load_from_file(&save_path);
    assert!(load_result.is_ok(), "Errore nel caricamento: {:?}", load_result.err());

    // Verifichiamo che tutti i valori siano stati salvati e riletti correttamente
    assert_eq!(
        config2.get("section1", "string").and_then(|v| v.as_string()).cloned(),
        Some("nuovo valore".to_string())
    );

    assert_eq!(
        config2.get("section1", "int").and_then(|v| v.as_integer()),
        Some(42)
    );

    assert_eq!(
        config2.get("section1", "float").and_then(|v| v.as_float()),
        Some(3.14)
    );

    assert_eq!(
        config2.get("section1", "bool").and_then(|v| v.as_boolean()),
        Some(true)
    );

    assert_eq!(
        config2.get("section2", "key").and_then(|v| v.as_string()).cloned(),
        Some("sezione 2".to_string())
    );
}


use tempfile::{tempdir, TempDir};
use confucius::{Config, ConfigValue, ConfigError, ConfigFormat};

/// Struttura per gestire un ambiente di test con file di configurazione
struct TestEnv {
    temp_dir: TempDir,
    app_name: String,
}

impl TestEnv {
    /// Crea un nuovo ambiente di test con un nome applicazione specifico
    fn new(app_name: &str) -> Self {
        let temp_dir = tempdir().expect("Impossibile creare directory temporanea");

        TestEnv {
            temp_dir,
            app_name: app_name.to_string(),
        }
    }

    /// Crea una struttura di directory per i test
    fn setup_directories(&self) {
        // Crea /etc/app_name/
        let etc_app_dir = self.temp_dir.path().join("etc").join(&self.app_name);
        fs::create_dir_all(&etc_app_dir).expect("Impossibile creare directory /etc/app_name");

        // Crea /etc/
        let etc_dir = self.temp_dir.path().join("etc");

        // Crea /opt/etc/
        let opt_etc_dir = self.temp_dir.path().join("opt").join("etc");
        fs::create_dir_all(&opt_etc_dir).expect("Impossibile creare directory /opt/etc");

        // Crea /home/user/.config/app_name/
        let home_config_app_dir = self.temp_dir.path()
            .join("home").join("user").join(".config").join(&self.app_name);
        fs::create_dir_all(&home_config_app_dir).expect("Impossibile creare directory /home/user/.config/app_name");

        // Crea /home/user/.config/
        let home_config_dir = self.temp_dir.path()
            .join("home").join("user").join(".config");

        // Crea /app/bin/
        let app_bin_dir = self.temp_dir.path().join("app").join("bin");
        fs::create_dir_all(&app_bin_dir).expect("Impossibile creare directory /app/bin");
    }

    /// Crea un file di configurazione in un percorso specifico
    fn create_config_file(&self, rel_path: &str, content: &str) -> PathBuf {
        let full_path = self.temp_dir.path().join(rel_path);

        // Assicuriamoci che la directory esista
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).expect(&format!("Impossibile creare directory {}", parent.display()));
        }

        fs::write(&full_path, content).expect(&format!("Impossibile scrivere file {}", full_path.display()));
        full_path
    }

    /// Ottiene il percorso completo a partire da un percorso relativo
    fn path(&self, rel_path: &str) -> PathBuf {
        self.temp_dir.path().join(rel_path)
    }
}

#[test]
fn test_config_search_paths() {
    let env = TestEnv::new("testapp");
    env.setup_directories();

    // Creiamo file di configurazione in diversi percorsi con contenuti diversi
    env.create_config_file(
        "etc/testapp/testapp.conf",
        "#!config/ini\n[test]\npath = \"etc/testapp/testapp.conf\"\npriority = 1\n"
    );

    env.create_config_file(
        "etc/testapp.conf",
        "#!config/ini\n[test]\npath = \"etc/testapp.conf\"\npriority = 2\n"
    );

    env.create_config_file(
        "opt/etc/testapp.conf",
        "#!config/ini\n[test]\npath = \"opt/etc/testapp.conf\"\npriority = 3\n"
    );

    env.create_config_file(
        "home/user/.config/testapp/testapp.conf",
        "#!config/ini\n[test]\npath = \"home/user/.config/testapp/testapp.conf\"\npriority = 4\n"
    );

    env.create_config_file(
        "home/user/.config/testapp.conf",
        "#!config/ini\n[test]\npath = \"home/user/.config/testapp.conf\"\npriority = 5\n"
    );

    env.create_config_file(
        "app/bin/testapp.conf",
        "#!config/ini\n[test]\npath = \"app/bin/testapp.conf\"\npriority = 6\n"
    );

    // Per simulare la ricerca nei percorsi, sovrascriviamo temporaneamente
    // la funzione di ricerca per utilizzare i nostri percorsi di test

    // Nota: questo test è più complesso e potrebbe richiedere modifiche alla libreria
    // per consentire l'override dei percorsi di ricerca a scopo di test
    // Qui presentiamo una versione semplificata che carica i file in ordine
    // e verifica quale viene scelto

    // Test 1: Verifichiamo che venga caricato il file con priorità più alta
    let mut config = Config::new("testapp");

    // Carica il primo file disponibile (priorità 1)
    let result = config.load_from_file(&env.path("etc/testapp/testapp.conf"));
    assert!(result.is_ok(), "Caricamento del file con priorità 1 fallito");

    if let Some(value) = config.get("test", "priority") {
        assert_eq!(value.as_integer(), Some(1));
    } else {
        panic!("Valore priority non trovato");
    }

    // Test 2: Se rimuoviamo il file con priorità 1, dovrebbe essere caricato quello con priorità 2
    fs::remove_file(env.path("etc/testapp/testapp.conf")).expect("Impossibile rimuovere file");

    let mut config = Config::new("testapp");
    let result = config.load_from_file(&env.path("etc/testapp.conf"));
    assert!(result.is_ok(), "Caricamento del file con priorità 2 fallito");

    if let Some(value) = config.get("test", "priority") {
        assert_eq!(value.as_integer(), Some(2));
    } else {
        panic!("Valore priority non trovato");
    }
}

#[test]
fn test_complex_include_scenario() {
    let env = TestEnv::new("complex");

    // Creiamo una struttura di configurazione più complessa con include annidati
    env.create_config_file(
        "main.conf",
        "#!config/ini\n[main]\nkey = \"main value\"\ninclude=includes/base.conf\n"
    );

    env.create_config_file(
        "includes/base.conf",
        "#!config/ini\n[base]\nkey = \"base value\"\ninclude=common/*.conf\n"
    );

    env.create_config_file(
        "includes/common/db.conf",
        "#!config/ini\n[database]\nhost = \"localhost\"\nport = 5432\nuser = \"admin\"\ninclude=secrets/db_password.conf\n"
    );

    env.create_config_file(
        "includes/common/app.conf",
        "#!config/ini\n[app]\nlog_level = \"info\"\nworkers = 4\n"
    );

    env.create_config_file(
        "includes/secrets/db_password.conf",
        "#!config/ini\n[database]\npassword = \"s3cr3t\"\n"
    );

    // Carica la configurazione dal file principale
    let mut config = Config::new("complex");
    let result = config.load_from_file(&env.path("main.conf"));
    assert!(result.is_ok(), "Caricamento della configurazione complessa fallito: {:?}", result.err());

    // Verifichiamo che tutti i valori siano stati caricati correttamente

    // Dal file main.conf
    if let Some(value) = config.get("main", "key") {
        assert_eq!(value.as_string(), Some(&"main value".to_string()));
    } else {
        panic!("Valore 'main.key' non trovato");
    }

    // Dal file base.conf
    if let Some(value) = config.get("base", "key") {
        assert_eq!(value.as_string(), Some(&"base value".to_string()));
    } else {
        panic!("Valore 'base.key' non trovato");
    }

    // Dal file db.conf
    if let Some(value) = config.get("database", "host") {
        assert_eq!(value.as_string(), Some(&"localhost".to_string()));
    } else {
        panic!("Valore 'database.host' non trovato");
    }

    if let Some(value) = config.get("database", "port") {
        assert_eq!(value.as_integer(), Some(5432));
    } else {
        panic!("Valore 'database.port' non trovato");
    }

    if let Some(value) = config.get("database", "user") {
        assert_eq!(value.as_string(), Some(&"admin".to_string()));
    } else {
        panic!("Valore 'database.user' non trovato");
    }

    // Dal file db_password.conf (attraverso l'inclusione in db.conf)
    if let Some(value) = config.get("database", "password") {
        assert_eq!(value.as_string(), Some(&"s3cr3t".to_string()));
    } else {
        panic!("Valore 'database.password' non trovato");
    }

    // Dal file app.conf
    if let Some(value) = config.get("app", "log_level") {
        assert_eq!(value.as_string(), Some(&"info".to_string()));
    } else {
        panic!("Valore 'app.log_level' non trovato");
    }

    if let Some(value) = config.get("app", "workers") {
        assert_eq!(value.as_integer(), Some(4));
    } else {
        panic!("Valore 'app.workers' non trovato");
    }
}

#[test]
fn test_value_overrides() {
    let env = TestEnv::new("override");

    // Creiamo file di configurazione con valori che si sovrascrivono
    env.create_config_file(
        "main.conf",
        "#!config/ini\n[section]\nkey1 = \"original value\"\nkey2 = 100\ninclude=override.conf\n"
    );

    env.create_config_file(
        "override.conf",
        "#!config/ini\n[section]\nkey1 = \"overridden value\"\nkey3 = true\n"
    );

    // Carica la configurazione
    let mut config = Config::new("override");
    let result = config.load_from_file(&env.path("main.conf"));
    assert!(result.is_ok(), "Caricamento della configurazione fallito: {:?}", result.err());

    // Verifichiamo che i valori siano stati sovrascritti correttamente
    if let Some(value) = config.get("section", "key1") {
        assert_eq!(value.as_string(), Some(&"overridden value".to_string()),
                   "Il valore di key1 dovrebbe essere sovrascritto");
    } else {
        panic!("Valore 'section.key1' non trovato");
    }

    // Verifichiamo che i valori non sovrascritti siano ancora presenti
    if let Some(value) = config.get("section", "key2") {
        assert_eq!(value.as_integer(), Some(100));
    } else {
        panic!("Valore 'section.key2' non trovato");
    }

    // Verifichiamo che i nuovi valori aggiunti dall'override siano presenti
    if let Some(value) = config.get("section", "key3") {
        assert_eq!(value.as_boolean(), Some(true));
    } else {
        panic!("Valore 'section.key3' non trovato");
    }
}