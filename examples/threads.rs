use std::{any::Any, collections::VecDeque, thread::{self, sleep}, time::Duration};

use basico::estruturas::{LinkedList, Stack};

fn run_linked_list(size: usize) {
    let mut list: LinkedList<usize> = LinkedList::new();
    for i in 0..size {
        list.add_first(i);
        list.add_last(i);
    }
}

fn run_deque(size: usize) {
    let mut deque: VecDeque<usize> = VecDeque::new();
    for i in 0..size {
        deque.push_front(i);
        deque.push_back(i);
    }
}


fn main() -> Result<(), Box<dyn Any + Send>> {
    let size = 10000000;
    let thread1 = thread::spawn(move || {
        println!("Running linked list...");
        run_linked_list(size);
        println!("Linked list done!");
    });
    let thread2 = thread::spawn(move || {
        println!("Running deque...");
        run_deque(size);
        println!("Deque done!");
    });

    thread1.join()?;
    thread2.join()?;

    Ok(())
}