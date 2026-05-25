use std::env;
use utils::leia;

mod estruturas;
mod utils;

fn main() {
    println!("Bem-vindo ao basico, um conjunto de estruturas de dados e algoritmos em Rust!");
    println!("Escolha um exemplo para rodar e execute com");
    println!("cargo run --example <nome do exemplo>");

    // Listar diretório examples para mostrar os exemplos disponíveis
    let examples = std::fs::read_dir("examples").unwrap();
    println!("Exemplos disponíveis:");
    for example in examples {
        let example = example.unwrap();
        let path = example.path();
        if path.is_file() {
            if let Some(name) = path.file_stem() {
                println!("- {}", name.to_string_lossy());
            }
        }
    }
}