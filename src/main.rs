use std::env;
use utils::leia;

mod exemplos;
mod estruturas;
mod utils;

const DIVISORIA: &str = "--------------------------------------------------------------------------------";

fn main() {    
    let args: Vec<String> = env::args().collect();
    loop {
        let escolha: String;
        if let Some(arg) = args.get(1) {
            escolha = arg.to_string();

            println!("Programa escolhido: {}", escolha);
        } else {
            println!("Qual programa quer executar? (aperte enter para terminar)");
            println!("- fib");
            println!("- memoria");
            println!("- hilbert");
            println!("- life");
            println!("- snake");
            println!("- minesweeper");
            println!("- expression");
            println!("- dungeon");
            println!("- sudoku");
            
            escolha = leia(">").to_lowercase();
        }
        
        if escolha == "" {
            break;
        }

        println!("{}",DIVISORIA);
        loop {
            if escolha == "fib" {
                exemplos::fib();
            } else if escolha == "memoria" {
                exemplos::memoria();
            } else if escolha == "hilbert" {
                exemplos::hilbert();
            } else if escolha == "life" {
                exemplos::life();
            } else if escolha == "snake" {
                exemplos::snake();
            } else if escolha == "minesweeper" {
                exemplos::mine_sweeper();
            } else if escolha == "expression" {
                exemplos::expression();
            } else if escolha == "dungeon" {
                exemplos::dungeon();
            } else if escolha == "sudoku" {
                exemplos::sudoku();
            } else {
                println!("Nenhum programa com este nome '{}'", escolha);
            }

            println!("{}",DIVISORIA);
            let resposta = leia("Deseja executar denovo? [S]/n \n>").to_lowercase();
            if resposta != "s" && resposta != "" {
                break;
            } else {
                println!("{}",DIVISORIA);
            }
        }        

        if let Some(_) = args.get(1) {
            break;
        }
    }
}