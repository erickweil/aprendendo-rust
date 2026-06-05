use std::{cell::RefCell, io, rc::Rc, sync::{Arc, Mutex}, thread};
use basico::promise::{promise::{Promise}, *};


fn test_create_file(path: String, contents: String) -> Promise<()> {
    Promise::new(move |e| {
        let e = e.into_send();
        thread::spawn(move || {
            match std::fs::write(path, contents) {
                Ok(v) => e.resolve(v),
                Err(err) => e.reject(Box::new(err)),
            }
        });
    })
}

fn test_read_file(path: String) -> Promise<String> {
    Promise::new(move |e| {
        let e = e.into_send();
        thread::spawn(move || {
            match std::fs::read_to_string(path) {
                Ok(v) => e.resolve(v),
                Err(err) => e.reject(Box::new(err)),
            }
        });
    })
}


fn main() {
    let mut counter = Rc::new(RefCell::new(0));

    let counter_clone = counter.clone();
    EventLoop::start(move || {

        EventLoop::set_interval(move |interval_id| {
            let mut counter = counter_clone.borrow_mut();

            println!("SetInterval 1s... Counter: {}", *counter);
            *counter += 1;

            // Parar o loop após 3 execuções
            if *counter >= 3 {
                println!("Parando o loop após 3 execuções...");
                EventLoop::clear_timeout(interval_id).unwrap();
            }
        }, 1000).unwrap();

        // Testando a criação e leitura de arquivo usando Promises
        test_create_file("test.txt".to_string(),"Hello, World!".to_string())
        .then(|_| {
            test_read_file("test.txt".to_string())
        }).then(|contents| {
            println!("File contents: {}", contents);

            Promise::resolve(())
        }).catch(|err| {
            eprintln!("Error: {}", err);

            Promise::resolve(())
        });

        println!("Fim EventLoop::start()...");
    }).unwrap();

    println!("Fim main! Counter: {}", *counter.borrow());
}