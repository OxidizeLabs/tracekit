# tracekit

Cache trace simulation toolkit for Rust. Generate synthetic workloads, replay traces, and measure cache performance.

## Crates

| Crate | Description |
|-------|-------------|
| [`tracekit`](tracekit/) | Core library: events, traits, workload generators, metrics |
| [`tracekit-formats`](tracekit-formats/) | Trace file parsers/writers (6+ formats: ARC, LIRS, CSV, Cachelib, JSONL, key-only) |
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

# Simulate with real-world traces
tracekit simulate --trace arc_trace.txt --format arc --capacity 1000
tracekit simulate --trace cachelib.csv --format cachelib --capacity 1000

# Convert between formats
tracekit rewrite --input trace.txt --input-format key-only --output trace.jsonl --output-format jsonl
tracekit rewrite --input arc_trace.txt --input-format arc --output trace.jsonl --output-format jsonl

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

tracekit supports **6+ real-world trace formats** including ARC, LIRS, CSV, Cachelib, and JSONL:

```rust
use std::fs::File;
use std::io::BufReader;
use tracekit::EventSource;

// Simple key-only format
use tracekit_formats::KeyOnlyReader;
let mut source = KeyOnlyReader::new(BufReader::new(File::open("trace.txt")?));

// ARC format (timestamp key size)
use tracekit_formats::ArcReader;
let mut source = ArcReader::new(BufReader::new(File::open("arc_trace.txt")?));

// LIRS format (block numbers)
use tracekit_formats::LirsReader;
let mut source = LirsReader::new(BufReader::new(File::open("lirs_trace.txt")?));

// CSV format (configurable)
use tracekit_formats::{CsvConfig, CsvReader};
let config = CsvConfig::default();
let mut source = CsvReader::new(BufReader::new(File::open("trace.csv")?), config);

// Cachelib format (Facebook/Meta traces)
use tracekit_formats::CachelibReader;
let mut source = CachelibReader::with_defaults(BufReader::new(File::open("cachelib.csv")?));

// Process events
while let Some(event) = source.next_event() {
    println!("Key: {}, Op: {:?}", event.key, event.op);
}
```

### Where to Get Real Traces

- **ARC traces:** [moka-rs/cache-trace](https://github.com/moka-rs/cache-trace/tree/main/arc)
- **LIRS traces:** [Caffeine simulator resources](https://github.com/ben-manes/caffeine/tree/master/simulator/src/main/resources)
- **Twitter traces:** [twitter/cache-trace](https://github.com/twitter/cache-trace)
- **Cachelib traces:** [cachelib.org](https://cachelib.org/docs/Cache_Library_User_Guides/Cachebench_FB_HW_eval/)
- **SNIA traces:** [iotta.snia.org](http://iotta.snia.org/)

See [`tracekit-formats/README.md`](tracekit-formats/README.md) for complete format documentation.

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
