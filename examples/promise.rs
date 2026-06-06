use std::{thread, time::Duration};
use basico::promise::{promise::{Promise}, *};
fn main() -> Result<(), BoxedError> {
    let promessa = Promise::new(|e| {
        let e = e.into_send();
        thread::spawn(|| {
            match std::fs::read("test.txt".to_string()) {
                Ok(v) => e.resolve(v),
                Err(err) => e.reject(err),
            }
        });
    }).map(|bytes| {
        println!("Finished read file! Bytes read: '{}'", bytes.len());
        
        String::from_utf8(bytes)
    }).then(|text| {
        println!("File content: '{}'", text);
        
        Promise::resolve(text)
    }).catch(|err| {
        println!("Error reading file: {}", err);
        Promise::resolve("".to_string())
    });

    let mut count = 0;
    EventLoop::set_interval(move |id| {
        println!("Interval! {}", count);
        count += 1;

        if count > 10 {
            EventLoop::clear_timeout(id);
        }

        Ok(())
    }, Duration::default())?;

    EventLoop::run_event_loop()
}