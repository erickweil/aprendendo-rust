use std::{thread, time::Duration};
use basico::promise::{promise::{Promise}, *};

fn main() -> Result<(), Error> {

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

    let mut count = 0;
    EventLoop::set_interval(move |id| {
        println!("Interval! {}", count);
        count += 1;

        Ok(())
    }, Duration::from_secs(1))?;

    EventLoop::run_event_loop()
}