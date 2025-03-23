// Copiado e editado de: https://github.com/spersson/bvmap/tree/master

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use rand::{rng, Rng};
use slotmap::{DefaultKey, DenseSlotMap, HopSlotMap, SlotMap};
use basico::VecPool;

fn inserts(c: &mut Criterion) {
    let size = 10_000;
    let s4: SlotMap<DefaultKey, usize> = SlotMap::new();
    let s6: DenseSlotMap<DefaultKey, usize> = DenseSlotMap::new();
    let pool: VecPool<usize> = VecPool::new();

    let mut g = c.benchmark_group("Inserts");
    g.bench_function("SlotMap", |b| {
        b.iter_batched_ref(
            || s4.clone(),
            |i| {
                for a in 0..size {
                    i.insert(a);
                }
            },
            BatchSize::SmallInput,
        );
    });
    g.bench_function("DenseSlotMap", |b| {
        b.iter_batched_ref(
            || s6.clone(),
            |i| {
                for a in 0..size {
                    i.insert(a);
                }
            },
            BatchSize::SmallInput,
        );
    });
    
    g.bench_function("VecPool", |b| {
        b.iter_batched_ref(
            || pool.clone(),
            |i| {
                for a in 0..size {
                    i.alloc_node(a);
                }
            },
            BatchSize::SmallInput,
        );
    });
}

fn reinserts(c: &mut Criterion) {
    let size = 10_000;
    let mut s4: SlotMap<DefaultKey, usize> = SlotMap::new();
    let mut s4k = Vec::new();
    let mut s6: DenseSlotMap<DefaultKey, usize> = DenseSlotMap::new();
    let mut s6k = Vec::new();

    let mut pool: VecPool<usize> = VecPool::new();
    let mut poolk = Vec::new();
    

    for a in 0..size {
        s4k.push(s4.insert(a));
        s6k.push(s6.insert(a));
        poolk.push(pool.alloc_node(a));
    }
    for a in 0..size {
        s4.remove(s4k[a]);
        s6.remove(s6k[a]);
        pool.free_node(poolk[a]);
    }
    let mut g = c.benchmark_group("Re-inserts");
    g.bench_function("SlotMap", |b| {
        b.iter_batched_ref(
            || s4.clone(),
            |i| {
                for a in 0..size {
                    i.insert(a);
                }
            },
            BatchSize::SmallInput,
        );
    });
    g.bench_function("DenseSlotMap", |b| {
        b.iter_batched_ref(
            || s6.clone(),
            |i| {
                for a in 0..size {
                    i.insert(a);
                }
            },
            BatchSize::SmallInput,
        );
    });
    g.bench_function("VecPool", |b| {
        b.iter_batched_ref(
            || pool.clone(),
            |i| {
                for a in 0..size {
                    i.alloc_node(a);
                }
            },
            BatchSize::SmallInput,
        );
    });
}

fn remove(c: &mut Criterion) {
    let size = 10_000;
    let mut s4: SlotMap<DefaultKey, usize> = SlotMap::new();
    let mut s4k = Vec::new();
    let mut s6: DenseSlotMap<DefaultKey, usize> = DenseSlotMap::new();
    let mut s6k = Vec::new();
    
    let mut pool: VecPool<usize> = VecPool::new();
    let mut poolk = Vec::new();
    
    for a in 0..size {
        s4k.push(s4.insert(a));
        s6k.push(s6.insert(a));
        poolk.push(pool.alloc_node(a));
    }

    let mut g = c.benchmark_group("Remove");
    
    g.bench_function("SlotMap", |b| {
        b.iter_batched_ref(
            || s4.clone(),
            |i| {
                for a in 0..size {
                    i.remove(s4k[a]);
                }
            },
            BatchSize::SmallInput,
        );
    });
    g.bench_function("DenseSlotMap", |b| {
        b.iter_batched_ref(
            || s6.clone(),
            |i| {
                for a in 0..size {
                    i.remove(s6k[a]);
                }
            },
            BatchSize::SmallInput,
        );
    });
    g.bench_function("VecPool", |b| {
        b.iter_batched_ref(
            || pool.clone(),
            |i| {
                for a in 0..size {
                    i.free_node(poolk[a]);
                }
            },
            BatchSize::SmallInput,
        );
    });
}

fn get(c: &mut Criterion) {
    let size = 10_000;
    let mut rng = rng();
    
    let mut s4: SlotMap<DefaultKey, usize> = SlotMap::new();
    let mut s4k = Vec::new();
    let mut s6: DenseSlotMap<DefaultKey, usize> = DenseSlotMap::new();
    let mut s6k = Vec::new();
    
    let mut pool: VecPool<usize> = VecPool::new();
    let mut poolk = Vec::new();

    for a in 0..size {
        s4k.push(s4.insert(a));
        s6k.push(s6.insert(a));
        poolk.push(pool.alloc_node(a));
    }

    let mut g = c.benchmark_group("Get");
    g.bench_function("SlotMap", |b| {
        b.iter_batched_ref(
            || s4.clone(),
            |i| {
                for _ in 0..size {
                    black_box(i.get(s4k[rng.random_range(0..size)]));
                }
            },
            BatchSize::SmallInput,
        );
    });
    g.bench_function("DenseSlotMap", |b| {
        b.iter_batched_ref(
            || s6.clone(),
            |i| {
                for _ in 0..size {
                    black_box(i.get(s6k[rng.random_range(0..size)]));
                }
            },
            BatchSize::SmallInput,
        );
    });
    g.bench_function("VecPool", |b| {
        b.iter_batched_ref(
            || pool.clone(),
            |i| {
                for _ in 0..size {
                    black_box(i.get_node(poolk[rng.random_range(0..size)]));
                }
            },
            BatchSize::SmallInput,
        );
    });
}
/*
fn iter(c: &mut Criterion) {
    let size = 10_000;
    let mut rng = thread_rng();
    let mut s1: BvMap<usize, usize> = BvMap::new();
    let mut s1k = Vec::new();
    let mut s2: Stash<usize, usize> = Stash::new();
    let mut s2k = Vec::new();
    let mut s3: UniqueStash<usize> = UniqueStash::new();
    let mut s3k = Vec::new();
    let mut s4: SlotMap<DefaultKey, usize> = SlotMap::new();
    let mut s4k = Vec::new();
    let mut s5: HopSlotMap<DefaultKey, usize> = HopSlotMap::new();
    let mut s5k = Vec::new();
    let mut s6: DenseSlotMap<DefaultKey, usize> = DenseSlotMap::new();
    let mut s6k = Vec::new();
    let mut s7: Slab<usize> = Slab::new();
    let mut s7k = Vec::new();
    let mut s9: ExternStableVec<usize> = ExternStableVec::new();
    let mut s9k = Vec::new();
    let mut s10: InlineStableVec<usize> = InlineStableVec::new();
    let mut s10k = Vec::new();
    let mut s11: IdVec<usize> = IdVec::new();
    let mut s11k = Vec::new();
    let mut s12: CompactMap<usize> = CompactMap::new();
    let mut s12k = Vec::new();
    let mut s14: Arena<usize> = Arena::new();
    let mut s14k = Vec::new();

    for a in 0..size {
        s1k.push(s1.insert(a));
        s2k.push(s2.put(a));
        s3k.push(s3.put(a));
        s4k.push(s4.insert(a));
        s5k.push(s5.insert(a));
        s6k.push(s6.insert(a));
        s7k.push(s7.insert(a));
        s9k.push(s9.push(a));
        s10k.push(s10.push(a));
        s11k.push(s11.insert(a));
        s12k.push(s12.insert(a));
        s14k.push(s14.insert(a));
    }

    let mut g = c.benchmark_group("Iterate");
    g.bench_function("BvMap", |b| {
        b.iter_batched_ref(
            || s1.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("Stash", |b| {
        b.iter_batched_ref(
            || s2.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("UniqueStash", |b| {
        b.iter_batched_ref(
            || s3.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("SlotMap", |b| {
        b.iter_batched_ref(
            || s4.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("HopSlotMap", |b| {
        b.iter_batched_ref(
            || s5.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("DenseSlotMap", |b| {
        b.iter_batched_ref(
            || s6.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("Slab", |b| {
        b.iter_batched_ref(
            || s7.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("BeachMap", |b| {
        b.iter_batched(
            || {
                let mut s8: BeachMap<usize, usize> = BeachMap::new();
                let mut s8k = Vec::new();
                for a in 0..size {
                    s8k.push(s8.insert(a));
                }
                s8
            },
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        );
    });
    g.bench_function("ExternStableVec", |b| {
        b.iter_batched_ref(
            || s9.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("InlineStableVec", |b| {
        b.iter_batched_ref(
            || s10.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("IdVec", |b| {
        b.iter_batched_ref(
            || s11.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("CompactMap", |b| {
        b.iter_batched_ref(
            || s12.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("Froggy", |b| {
        b.iter_batched(
            || {
                let mut s13: Storage<usize> = Storage::new();
                let mut s13k = Vec::new();
                for a in 0..size {
                    s13k.push(s13.create(a));
                }
                (s13, s13k)
            },
            |(i, _k)| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        );
    });
    g.bench_function("GenerationalArena", |b| {
        b.iter_batched_ref(
            || s14.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.finish();

    for subset in ((size / 2)..size).rev() {
        let k = rng.gen_range(0, subset);
        s1.remove(s1k[k]);
        s1k.swap_remove(k);
        s2.take(s2k[k]);
        s2k.swap_remove(k);
        s3.take(s3k[k]);
        s3k.swap_remove(k);
        s4.remove(s4k[k]);
        s4k.swap_remove(k);
        s5.remove(s5k[k]);
        s5k.swap_remove(k);
        s6.remove(s6k[k]);
        s6k.swap_remove(k);
        s7.remove(s7k[k]);
        s7k.swap_remove(k);
        s9.remove(s9k[k]);
        s9k.swap_remove(k);
        s10.remove(s10k[k]);
        s10k.swap_remove(k);
        s11.remove(s11k[k]);
        s11k.swap_remove(k);
        s12.remove(s12k[k]);
        s12k.swap_remove(k);
        s14.remove(s14k[k]);
        s14k.swap_remove(k);
    }

    let mut g = c.benchmark_group("Iterate half-full");
    g.bench_function("BvMap", |b| {
        b.iter_batched_ref(
            || s1.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("Stash", |b| {
        b.iter_batched_ref(
            || s2.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("UniqueStash", |b| {
        b.iter_batched_ref(
            || s3.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("SlotMap", |b| {
        b.iter_batched_ref(
            || s4.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("HopSlotMap", |b| {
        b.iter_batched_ref(
            || s5.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("DenseSlotMap", |b| {
        b.iter_batched_ref(
            || s6.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("Slab", |b| {
        b.iter_batched_ref(
            || s7.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("BeachMap", |b| {
        b.iter_batched(
            || {
                let mut s8: BeachMap<usize, usize> = BeachMap::new();
                let mut s8k = Vec::new();
                for a in 0..size {
                    s8k.push(s8.insert(a));
                }
                for subset in ((size / 2)..size).rev() {
                    let k = rng.gen_range(0, subset);
                    s8.remove(s8k[k]);
                    s8k.swap_remove(k);
                }
                s8
            },
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        );
    });
    g.bench_function("ExternStableVec", |b| {
        b.iter_batched_ref(
            || s9.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("InlineStableVec", |b| {
        b.iter_batched_ref(
            || s10.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("IdVec", |b| {
        b.iter_batched_ref(
            || s11.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("CompactMap", |b| {
        b.iter_batched_ref(
            || s12.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("Froggy", |b| {
        b.iter_batched(
            || {
                let mut s13: Storage<usize> = Storage::new();
                let mut s13k = Vec::new();
                for a in 0..size {
                    s13k.push(s13.create(a));
                }
                for subset in ((size / 2)..size).rev() {
                    let k = rng.gen_range(0, subset);
                    s13k.swap_remove(k);
                }
                s13.sync_pending();
                (s13, s13k)
            },
            |(i, _k)| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        );
    });
    g.bench_function("GenerationalArena", |b| {
        b.iter_batched_ref(
            || s14.clone(),
            |i| {
                for a in i.iter() {
                    black_box(a);
                }
            },
            BatchSize::SmallInput,
        )
    });
}*/

criterion_group!(benches, inserts, reinserts, remove, get);//, iter);
criterion_main!(benches);