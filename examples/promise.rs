use std::{thread, time::Duration};
use basico::promise::{EventLoop, EventLoopError, promise::Promise};

fn main() -> Result<(), EventLoopError> {
    let promessa: Promise<(), std::io::Error> = Promise::new(|e| {
        let e = e.into_send();
        thread::spawn(|| {
            match std::fs::read("test.txt".to_string()) {
                Ok(v) => e.resolve(v),
                Err(err) => e.reject(err),
            }
        });
    }).map(|bytes| {
        println!("Finished read file! Bytes read: '{}'", bytes.len());
        
        let str = String::from_utf8(bytes).map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

        Ok(str)
    }).then(|text| {
        println!("File content: '{}'", text);
        
        Promise::resolve(())
    }).catch(|err| {
        println!("Error reading file: {}", err);
        Promise::resolve(())
    });

    EventLoop::run_event_loop()
}