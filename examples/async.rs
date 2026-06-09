use std::time::Duration;

use basico::a_sync::{AsyncExecutor, Sleep, Yield};

async fn counting(id: u64, n: u64) {
    for i in 0..n {
        println!("ID {}, Counting: {}", id, i);
        //Yield::now().await;
        Sleep::duration(Duration::from_secs(1)).await;
    }

    println!("ID {}, Done!", id);
}

fn main() {
    AsyncExecutor::spawn(counting(5, 5));
    AsyncExecutor::spawn(counting(2, 2));
    AsyncExecutor::spawn(counting(10, 10));
    AsyncExecutor::run().unwrap();
}