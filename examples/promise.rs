use std::{cell::RefCell, io, rc::Rc, thread};
use basico::promise::{promise::{Promise}, *};

fn main() {
    let mut counter = Rc::new(RefCell::new(0));

    let counter_clone = counter.clone();
    EventLoop::start(move || {
        EventLoop::set_timeout(|| {
            panic!("Nunca deveria ser executado!");
        }, 10000).unwrap();

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

        Promise::new(move |e| {
            println!("Agendando setTimeout de 5s...");
            EventLoop::set_timeout(move || {
                e.resolve(());
            }, 5000).unwrap();
        }).then(move |_| {
            println!("Encerrado setTimeout após 5s...");

            EventLoop::stop();

            Promise::resolve(())
        });

        println!("Fim EventLoop::start()...");
    }).unwrap();

    println!("Fim main! Counter: {}", *counter.borrow());
}