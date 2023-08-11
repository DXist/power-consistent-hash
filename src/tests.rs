use super::PowerConsistentHasher;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use tracing::{debug, trace};

#[test]
fn test_consistent_hashing_backwards_compatibility() {
    let hasher = PowerConsistentHasher::try_new(3).unwrap();
    let key0 = 0;
    assert_eq!(hasher.hash(key0), 0);
    let key1 = 8;
    assert_eq!(hasher.hash(key1), 0);
    let key2 = 9;
    assert_eq!(hasher.hash(key2), 1);
    let key3 = 10;
    assert_eq!(hasher.hash(key3), 2);
    assert_eq!(hasher.hash(key3), 2);
    let key4 = 11;
    assert_eq!(hasher.hash(key4), 1);
    let key5 = 12;
    assert_eq!(hasher.hash(key5), 0);
}

#[test]
fn test_consistent_hashing_disbalance() {
    // tracing_subscriber::fmt::init();
    const BUCKET_COUNT: u32 = 96;
    let mut buckets = [0; BUCKET_COUNT as usize];
    let h = PowerConsistentHasher::try_new(BUCKET_COUNT).unwrap();
    const KEY_COUNT: u64 = 1000000;
    for k in 0_u64..KEY_COUNT {
        let mut hasher = DefaultHasher::new();
        hasher.write(&k.to_le_bytes());
        let key = hasher.finish();
        let b = h.hash(key);
        buckets[b as usize] += 1;
    }
    debug!(buckets = format_args!("{:#?}", buckets));
    let max = buckets.iter().max().unwrap();
    let min = buckets.iter().min().unwrap();
    let delta = (max - min) as f32 / KEY_COUNT as f32;
    debug!(disbalance = delta);
    assert!(delta < 0.001);
}

#[test]
fn test_consistency() {
    tracing_subscriber::fmt::init();
    const KEY_COUNT: usize = 100;
    let mut hashed_buckets = [0; KEY_COUNT];

    const INITIAL_BUCKET_COUNT: u32 = 5;

    let mut bucket_count: u32 = INITIAL_BUCKET_COUNT;
    let h = PowerConsistentHasher::try_new(bucket_count).unwrap();
    for k in 0..KEY_COUNT {
        let mut hasher = DefaultHasher::new();
        hasher.write(&k.to_le_bytes());
        let key = hasher.finish();
        let b = h.hash(key);
        hashed_buckets[k] = b;
    }

    const NEW_BUCKET_COUNT: u32 = 6;
    bucket_count = NEW_BUCKET_COUNT;
    let mut relocated: u32 = 0;
    let h = PowerConsistentHasher::try_new(bucket_count).unwrap();
    for k in 0..KEY_COUNT {
        let mut hasher = DefaultHasher::new();
        hasher.write(&k.to_le_bytes());
        let key = hasher.finish();
        let b = h.hash(key);
        if hashed_buckets[k] != b {
            relocated += 1;
        }
        trace!(k = k, before = hashed_buckets[k], after = b);
    }
    const EXPECTED_RELOCATED_RATIO: f32 = 1. / NEW_BUCKET_COUNT as f32;
    let relocated_ratio = relocated as f32 / KEY_COUNT as f32;

    debug!(
        relocated = relocated,
        relocated_ratio = relocated_ratio,
        expected_relocated_ratio = EXPECTED_RELOCATED_RATIO
    );
    assert!((relocated_ratio - EXPECTED_RELOCATED_RATIO).abs() < 0.01);
}
