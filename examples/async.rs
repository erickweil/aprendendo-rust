use basico::a_sync::{AsyncExecutor, Yield};

async fn counting(n: u64) {
    for i in 0..n {
        println!("Counting: {}", i);
        Yield::now().await;
    }
}

fn main() {
    AsyncExecutor::spawn(counting(100));
    AsyncExecutor::run().unwrap();
}