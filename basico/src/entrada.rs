use std::io;

pub fn leia(prompt: &str) -> String {
    println!("{}", prompt);

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Erro ao ler terminal");

    // Precisa to_string() porque não pode retornar &str
    return input.trim().to_string();
}