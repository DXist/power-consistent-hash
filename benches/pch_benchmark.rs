use std::{collections::hash_map::DefaultHasher, hash::Hasher};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use power_consistent_hash::PowerConsistentHasher;

fn bench_pch_1000(c: &mut Criterion) {
    const NUM_OF_KEYS: u64 = 1000;
    // initial number of cluster nodes with sharded data
    const REPLICATION_FACTOR: u32 = 3;

    let mut group = c.benchmark_group("pch");
    // num of hash operation per bench iteration
    group.throughput(Throughput::Elements(NUM_OF_KEYS));
    // spread keys over u32 range
    let keys: Vec<u64> = (0..NUM_OF_KEYS)
        .map(|i| {
            let mut hasher = DefaultHasher::new();
            hasher.write(&i.to_le_bytes());
            hasher.finish()
        })
        .collect();

    // simulate cluster horizontal scaling
    for power in [3, 5, 7, 9, 11, 13, 15] {
        let num_of_buckets = 2_u32.pow(power) * REPLICATION_FACTOR;
        let hasher = PowerConsistentHasher::try_new(num_of_buckets).unwrap();
        group.bench_with_input(
            BenchmarkId::new("bucket_scaling", num_of_buckets),
            &(hasher, keys.as_slice()),
            |b, (hasher, keys)| {
                b.iter(|| {
                    keys.iter().for_each(|key| {
                        hasher.hash_u64(*key);
                    })
                });
            },
        );
    }
}

criterion_group!(benches, bench_pch_1000);
criterion_main!(benches);
