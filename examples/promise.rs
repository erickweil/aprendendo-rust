use std::thread;
use basico::promise::{promise::{Promise}, *};

fn main() -> Result<(), Error> {
    EventLoop::start(|| {
        Promise::new(|e| {
            let e = e.into_send();
            thread::spawn(|| {
                match std::fs::read_to_string("test.txt".to_string()) {
                    Ok(v) => e.resolve(v),
                    Err(err) => e.reject(Box::new(err)),
                }
            });
        }).then(|contents| {
            println!("Finished read file! contents: '{}'", contents);
            Promise::resolve(())
        }).catch(|err| {
            println!("Error reading file: {}", err);
            Promise::resolve(())
        });

        Ok(())
    })
}