use std::{thread, time::Duration};
use basico::promise::{promise::{Promise}, *};
fn main() -> Result<(), Error> {
    Promise::new(|e| {
        let e = e.into_send();
        thread::spawn(|| {
            match std::fs::read("test.txt".to_string()) {
                Ok(v) => e.resolve(v),
                Err(err) => e.reject(Box::new(err)),
            }
        });
    }).then(|bytes| {
        println!("Finished read file! Bytes read: '{}'", bytes.len());
        
        String::from_utf8(bytes).into()
    }).then(|text| {
        println!("File content: '{}'", text);
        
        Promise::resolve(())
    }).catch(|err| {
        println!("Error reading file: {}", err);
        Promise::resolve(())
    });

    let mut count = 0;
    EventLoop::set_interval(move |id| {
        println!("Interval! {}", count);
        count += 1;

        if count > 100000 {
            EventLoop::clear_timeout(id);
        }

        Ok(())
    }, Duration::default())?;

    EventLoop::run_event_loop()
}