use std::fs;
use std::path::Path;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use confucius::{Config, ConfigValue};

// Crea un file di configurazione di test di dimensioni medie
fn create_test_config() -> String {
    let mut content = String::from("#!config/ini\n");

    // Aggiungiamo 10 sezioni con 10 chiavi ciascuna
    for section_idx in 0..10 {
        content.push_str(&format!("\n[section{}]\n", section_idx));

        // Chiavi di diversi tipi
        for key_idx in 0..10 {
            match key_idx % 4 {
                0 => content.push_str(&format!("string_key{} = \"valore stringa {}\"\n", key_idx, key_idx)),
                1 => content.push_str(&format!("int_key{} = {}\n", key_idx, key_idx * 100)),
                2 => content.push_str(&format!("float_key{} = {}.{}\n", key_idx, key_idx, key_idx)),
                3 => content.push_str(&format!("bool_key{} = {}\n", key_idx, key_idx % 2 == 0)),
                _ => unreachable!(),
            }
        }
    }

    content
}

fn bench_parse_config(c: &mut Criterion) {
    let content = create_test_config();
    let file_path = Path::new("bench_config.conf");
    fs::write(file_path, &content).expect("Impossibile scrivere il file di benchmark");

    c.bench_function("parse_config", |b| {
        b.iter(|| {
            let mut config = Config::new("bench");
            black_box(config.load_from_file(file_path).expect("Errore nel caricamento"));
        });
    });

    // Pulizia
    let _ = fs::remove_file(file_path);
}

fn bench_get_set_values(c: &mut Criterion) {
    let content = create_test_config();
    let file_path = Path::new("get_set_bench.conf");
    fs::write(file_path, &content).expect("Impossibile scrivere il file di benchmark");

    let mut config = Config::new("bench");
    config.load_from_file(file_path).expect("Errore nel caricamento");

    c.bench_function("get_values", |b| {
        b.iter(|| {
            // Lettura di valori 
            for section_idx in [1, 3, 5, 8].iter() {
                for key_idx in [2, 4, 6, 9].iter() {
                    black_box(config.get(&format!("section{}", section_idx),
                                         &format!("string_key{}", key_idx)));
                    black_box(config.get(&format!("section{}", section_idx),
                                         &format!("int_key{}", key_idx % 10)));
                }
            }
        });
    });

    c.bench_function("set_values", |b| {
        b.iter(|| {
            // Modifica di valori
            black_box(config.set("benchmark", "string", ConfigValue::String("valore benchmark".to_string())));
            black_box(config.set("benchmark", "int", ConfigValue::Integer(12345)));
            black_box(config.set("benchmark", "float", ConfigValue::Float(3.14159)));
            black_box(config.set("benchmark", "bool", ConfigValue::Boolean(true)));
        });
    });

    // Pulizia
    let _ = fs::remove_file(file_path);
}

fn bench_save_config(c: &mut Criterion) {
    let content = create_test_config();
    let file_path = Path::new("save_bench.conf");
    let out_path = Path::new("saved_bench.conf");

    fs::write(file_path, &content).expect("Impossibile scrivere il file di benchmark");

    let mut config = Config::new("bench");
    config.load_from_file(file_path).expect("Errore nel caricamento");

    c.bench_function("save_config", |b| {
        b.iter(|| {
            black_box(config.save_to_file(out_path).expect("Errore nel salvataggio"));
        });
    });

    // Pulizia
    let _ = fs::remove_file(file_path);
    let _ = fs::remove_file(out_path);
}

criterion_group!(benches, bench_parse_config, bench_get_set_values, bench_save_config);
criterion_main!(benches);