# tracekit

Cache trace simulation toolkit for Rust. Generate synthetic workloads, replay traces, and measure cache performance.

## Crates

| Crate | Description |
|-------|-------------|
| [`tracekit`](tracekit/) | Core library: events, traits, workload generators, metrics |
| [`tracekit-formats`](tracekit-formats/) | Trace file parsers/writers (key-only, JSONL) |
| [`tracekit-cachekit`](tracekit-cachekit/) | Adapter for cachekit cache implementations |
| [`tracekit-cli`](tracekit-cli/) | CLI tools: tracegen, simulate, rewrite, render |

## Quick Start

```rust
use tracekit::{simulate, CacheModel, BoundedGenerator, WorkloadSpec, Workload};

// Create a Zipfian workload generator
let spec = WorkloadSpec {
    universe: 10_000,
    workload: Workload::Zipfian { exponent: 1.0 },
    seed: 42,
};
let mut source = BoundedGenerator::new(spec.generator(), 100_000);

// Simulate with your cache (implements CacheModel)
let mut cache = MyCache::new(1000);
let stats = simulate(&mut cache, &mut source);
println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
```

## CLI Usage

```sh
# Generate a trace file
tracekit tracegen --workload zipfian --exponent 1.0 --universe 10000 --count 100000 -o trace.txt

# Simulate with a simple LRU cache
tracekit simulate --trace trace.txt --capacity 1000

# Convert between formats
tracekit rewrite --input trace.txt --input-format key-only --output trace.jsonl --output-format jsonl

# Render benchmark results to documentation
tracekit render results.json docs/benchmarks/
```

## Workload Types

tracekit includes 16+ synthetic workload generators:

| Workload | Description |
|----------|-------------|
| `Uniform` | Uniform random keys |
| `Zipfian` | Zipfian distribution (skewed, real-world patterns) |
| `ScrambledZipfian` | Zipfian with hashed keys (YCSB default) |
| `HotSet` | Hot/cold split (e.g., 90% accesses to 10% of keys) |
| `Scan` | Sequential scan |
| `Latest` | Recently inserted keys are more likely to be accessed |
| `ShiftingHotspot` | Popular keys change over time |
| `ScanResistance` | Mixed point lookups and scans |
| `FlashCrowd` | Sudden traffic spikes |
| `Bursty` | Traffic arrives in bursts |
| `Correlated` | Sequential access patterns |
| `Loop` | Cyclic working set |
| And more... | |

## Implementing CacheModel

To simulate your cache, implement the `CacheModel` trait:

```rust
use tracekit::CacheModel;

struct MyCache {
    // ...
}

impl CacheModel for MyCache {
    fn get(&mut self, key: u64) -> bool {
        // Return true on hit, false on miss
        todo!()
    }

    fn insert(&mut self, key: u64) {
        // Insert or update the key
        todo!()
    }

    fn delete(&mut self, key: u64) {
        // Optional: remove the key
    }
}
```

## Reading Trace Files

```rust
use std::fs::File;
use std::io::BufReader;
use tracekit::EventSource;
use tracekit_formats::KeyOnlyReader;

let file = File::open("trace.txt")?;
let reader = BufReader::new(file);
let mut source = KeyOnlyReader::new(reader);

while let Some(event) = source.next_event() {
    println!("Key: {}", event.key);
}
```

## Development

```sh
# Run all tests
cargo test --workspace

# Check all crates
cargo check --workspace

# Build CLI
cargo build --package tracekit-cli --release
```

## License

Dual-licensed under MIT and Apache-2.0. See `LICENSE-MIT` and `LICENSE-APACHE`.
