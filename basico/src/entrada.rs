use std::io;

pub fn leia(prompt: &str) -> String {
    println!("{}", prompt);

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Erro ao ler terminal");

    // Precisa to_string() porque não pode retornar &str
    return input.trim().to_string();
}

// pub fn leia_i(prompt: &str, maybe_default: Option<i32>) -> i32 {
//     let input = leia(prompt);

//     if let Some(default) = maybe_default {
//         return input.parse::<i32>().unwrap_or(default);
//     } else {
//         return input.parse::<i32>().unwrap();
//     }
// }