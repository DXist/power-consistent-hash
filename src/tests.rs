use std::{collections::hash_map::DefaultHasher, hash::Hasher};

use tracing::{debug, trace};

use super::PowerConsistentHasher;

#[test]
fn test_consistent_hashing_backwards_compatibility() {
    let hasher = PowerConsistentHasher::try_new(96).unwrap();
    let key0 = 0;
    assert_eq!(hasher.hash_u64(key0), 0);
    let key1 = 8;
    assert_eq!(hasher.hash_u64(key1), 12);
    let key2 = 9;
    assert_eq!(hasher.hash_u64(key2), 12);
    let key3 = 10;
    assert_eq!(hasher.hash_u64(key3), 13);
    assert_eq!(hasher.hash_u64(key3), 13);
    let key4 = 11;
    assert_eq!(hasher.hash_u64(key4), 15);
    let key5 = 12;
    assert_eq!(hasher.hash_u64(key5), 11);
    let key6 = 999;
    assert_eq!(hasher.hash_u64(key6), 89);
}
#[cfg(feature = "seahash")]
#[test]
fn test_seahash_consistent_hashing_backwards_compatibility() {
    let hasher = PowerConsistentHasher::try_new(96).unwrap();
    let key0 = 0_u64;
    assert_eq!(hasher.hash_bytes(&key0.to_le_bytes()), 29);
    let key1 = 8_u64;
    assert_eq!(hasher.hash_bytes(&key1.to_le_bytes()), 43);
    let key2 = 9_u64;
    assert_eq!(hasher.hash_bytes(&key2.to_le_bytes()), 16);
    let key3 = 10_u64;
    assert_eq!(hasher.hash_bytes(&key3.to_le_bytes()), 86);
    assert_eq!(hasher.hash_bytes(&key3.to_le_bytes()), 86);
    let key4 = 11_u64;
    assert_eq!(hasher.hash_bytes(&key4.to_le_bytes()), 59);
    let key5 = 12_u64;
    assert_eq!(hasher.hash_bytes(&key5.to_le_bytes()), 91);
    let key6 = 999_u64;
    assert_eq!(hasher.hash_bytes(&key6.to_le_bytes()), 27);
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
        let b = h.hash_u64(key);
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
        let b = h.hash_u64(key);
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
        let b = h.hash_u64(key);
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
