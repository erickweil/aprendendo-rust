mod fib;
mod entrada;
use entrada::leia;

fn main() {
    loop {
        println!("Qual programa quer executar? (aperte enter para terminar)");
        println!("- fib");

        let escolha = leia("").to_lowercase();

        if escolha == "" {
            break;
        }

        loop {
            if escolha == "fib" {
                fib::fib();
            } else {
                println!("Nenhum programa com este nome...");
            }

            let resposta = leia("Deseja executar denovo? [S]/n").to_lowercase();
            if resposta != "s" && resposta != "" {
                break;
            }
        }
    }
}