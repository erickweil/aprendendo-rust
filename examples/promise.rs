use std::{cell::RefCell, io, rc::Rc, thread};
use basico::promise::*;

fn main() {
    let mut counter = Rc::new(RefCell::new(0));

    let counter_clone = counter.clone();
    EventLoop::start(move || {
        EventLoop::set_interval(move || {
            let mut counter = counter_clone.borrow_mut();

            println!("SetInterval 1s... Counter: {}", *counter);
            *counter += 1;

            // Parar o loop após 3 execuções
            Ok(*counter < 3)
        }, 1000)?;

        Promise::new(move |e| {
            EventLoop::set_timeout(move || {
                e.resolve(());
                Ok(())
            }, 5000).unwrap();
        }).then(move |_| {
            println!("Promise após 5s...");

            Promise::Resolved(())
        });

        println!("Fim EventLoop::start()...");
        Ok(())
    }).unwrap();

    println!("Fim main! Counter: {}", *counter.borrow());
}