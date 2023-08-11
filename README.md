# Constant time consistent hash

This repo contains implementation of [power consistent hash](https://arxiv.org/pdf/2307.12448.pdf) - constant expected time and constant memory [consistent hashing](https://en.wikipedia.org/wiki/Consistent_hashing). Minimal number of keys are remapped when the number of buckets changes.

Target use cases - load balancing and data sharding.

The hashing algorithm execution time doesn't depend on number of hashing time.


# Benchmark - hashing 1k 64 bit key batches

2.6GHz Intel Core i7 hashes 1k 64 bit keys in ~6.4 microseconds. The left axis is a number of consistent hash buckets:

![Benchmark of hashing 1k keys](./pch_bucket_scaling.svg)

With optional integration of [SeaHash](https://gitlab.redox-os.org/redox-os/seahash) to produce 64 bit key fingerprints hashing of 1k UUIDs takes around ~25 microseconds.
