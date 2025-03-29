use std::collections::VecDeque;

use basico::{Queue, Stack};
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use rand::{rng, Rng};

/*fn despejar<T>(entrada: &mut LinkedStack<T>,saida: &mut LinkedStack<T>) -> Option<T> {
    if saida.len() <= 0 { 
        if entrada.len() <= 0 { return None; }

        // Despejar entrada na saída
        while entrada.len() > 0 {
            saida.push(entrada.pop().unwrap());
        }
    }
    return saida.pop();
}*/

fn linked_list(c: &mut Criterion) {
    let size = 10_000;
    
    let mut g = c.benchmark_group("Linked");
    g.bench_function("Linked Pool", |b| {
        b.iter_batched_ref(
            || basico::LinkedList::new(),
            |i| {
                for a in 0..size {
                    i.add_last(a);
                    i.add_first(a);
                    i.add_last(a);
                    i.add_first(a);

                    i.remove_last();
                    i.remove_last();
                }
            },
            BatchSize::SmallInput,
        );
    });

    // g.bench_function("Double Linked Stack Queue", |b| {
    //     b.iter_batched_ref(
    //         || (basico::LinkedStack::new(),basico::LinkedStack::new()),
    //         |(a,b)| {
    //             for k in 0..size {
    //                 a.push(k);
    //                 a.push(k);
    //                 a.push(k);
    //                 a.push(k);

    //                 despejar(a, b);
    //                 despejar(a, b);
    //             }
    //         },
    //         BatchSize::SmallInput,
    //     );
    // });

    // g.bench_function("Double Stack Queue", |b| {
    //     b.iter_batched_ref(
    //         || basico::DoubleStackQueue::new(),
    //         |i| {
    //             for a in 0..size {
    //                 i.enqueue(a);
    //                 i.enqueue(a);
    //                 i.enqueue(a);
    //                 i.enqueue(a);

    //                 i.dequeue();
    //                 i.dequeue();
    //             }
    //         },
    //         BatchSize::SmallInput,
    //     );
    // });

    g.bench_function("Deque (Rust)", |b| {
        b.iter_batched_ref(
            || VecDeque::new(),
            |i| {
                for a in 0..size {
                    i.push_back(a);
                    i.push_front(a);
                    i.push_back(a);
                    i.push_front(a);

                    i.pop_front();
                    i.pop_back();
                }
            },
            BatchSize::SmallInput,
        );
    });

    g.bench_function("Linked List (Rust)", |b| {
        b.iter_batched_ref(
            ||  std::collections::LinkedList::new(),
            |i| {
                for a in 0..size {
                    i.push_back(a);
                    i.push_front(a);
                    i.push_back(a);
                    i.push_front(a);

                    i.pop_front();
                    i.pop_back();
                }
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, linked_list);
criterion_main!(benches);