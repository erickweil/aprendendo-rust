use std::env;

mod entrada;
use entrada::leia;

mod fib;
mod memoria;

fn main() {
    let args: Vec<String> = env::args().collect();
    loop {
        let escolha: String;
        if let Some(arg) = args.get(1) {
            escolha = arg.to_string();
        } else {
            println!("Qual programa quer executar? (aperte enter para terminar)");
            println!("- fib");
            println!("- memoria");
            
            escolha = leia("").to_lowercase();
        }
        
        if escolha == "" {
            break;
        }

        loop {
            if escolha == "fib" {
                fib::fib();
            } else if escolha == "memoria" {
                memoria::memoria();
            } else {
                println!("Nenhum programa com este nome '{}'", escolha);
            }

            let resposta = leia("Deseja executar denovo? [S]/n").to_lowercase();
            if resposta != "s" && resposta != "" {
                break;
            }
        }        

        if let Some(_) = args.get(1) {
            break;
        }
    }
}