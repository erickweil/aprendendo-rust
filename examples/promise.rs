use std::io;

use basico::promise::*;

fn esperar(ms: u64) -> Promise<()> {
    Promise::new(move |e| {
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            e.resolve(());
        });
    })
}

fn executar() -> Promise<()> {
    println!("Contando...");
    esperar(1000).then(move |_| {
        println!("1...");
        esperar(1000)
    }).then(move |_| {
        println!("2...");
        esperar(1000)
    }).then(move |_| {
        println!("3...");
        esperar(1000)
        //Promise::reject(Box::new(io::Error::other("Erro na contagem!")))
    }).finally(move || {
        println!("Contagem finalizada!");
    })
}

fn main() -> Result<(), Error> {
    let result = Promise::wait_blocking(executar())?;
    println!("Tudo terminou! {:?}", result);

    Ok(())
}
