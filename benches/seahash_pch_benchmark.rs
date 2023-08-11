use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::Throughput;
use criterion::{criterion_group, criterion_main};
use power_consistent_hash::PowerConsistentHasher;
use uuid::Uuid;

fn bench_seahash_pch_1000(c: &mut Criterion) {
    const NUM_OF_KEYS: u64 = 1000;
    // initial number of cluster nodes with sharded data
    const REPLICATION_FACTOR: u32 = 3;

    let mut group = c.benchmark_group("seahash_pch");
    // num of hash operation per bench iteration
    group.throughput(Throughput::Elements(NUM_OF_KEYS));
    let items: Vec<Uuid> = (0..NUM_OF_KEYS).map(|_| Uuid::new_v4()).collect();
    // simulate cluster horizontal scaling
    for power in [3, 5, 7, 9, 11, 13, 15] {
        let num_of_buckets = 2_u32.pow(power) * REPLICATION_FACTOR;
        let hasher = PowerConsistentHasher::try_new(num_of_buckets).unwrap();
        group.bench_with_input(
            BenchmarkId::new("bucket_scaling", num_of_buckets),
            &(hasher, items.as_slice()),
            |b, (hasher, items)| {
                b.iter(|| {
                    items.iter().for_each(|item| {
                        hasher.hash_bytes(item.as_bytes());
                    })
                });
            },
        );
    }
}

criterion_group!(seahash_benches, bench_seahash_pch_1000);
criterion_main!(seahash_benches);
