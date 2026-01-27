# Working with Real-World Cache Traces

This guide explains how to use tracekit with real-world cache traces for evaluating your cache implementation.

## Quick Start

### 1. Download a Real Trace

Start with ARC traces (simplest format):

```bash
# Clone the cache-trace repository
git clone https://github.com/moka-rs/cache-trace.git
cd cache-trace/arc

# You'll find traces like:
# - S3.lis (IBM database trace)
# - DS1.lis (storage trace)
# - P1.lis, P2.lis, ... (various workloads)
```

### 2. Run Simulation

```bash
# Simulate with your cache
tracekit simulate \
  --trace cache-trace/arc/S3.lis \
  --format arc \
  --capacity 10000

# Output:
# Simulation Results:
#   Trace: cache-trace/arc/S3.lis
#   Cache capacity: 10000
#   Total requests: 4000000
#   Hits: 2500000
#   Misses: 1500000
#   Hit rate: 62.50%
#   Inserts: 1500000
```

### 3. Compare with Synthetic Workloads

```bash
# Generate a similar synthetic workload
tracekit tracegen \
  --workload zipfian \
  --exponent 0.8 \
  --universe 100000 \
  --count 4000000 \
  -o synthetic.txt

# Simulate with the same cache
tracekit simulate --trace synthetic.txt --capacity 10000
```

## Supported Trace Formats

### 1. ARC Format (Recommended for Beginners)

**Format:** Space-separated `timestamp key [size]`

```text
1 12345 4096
2 67890 8192
3 12345 4096
```

**Sources:**
- [moka-rs/cache-trace](https://github.com/moka-rs/cache-trace/tree/main/arc)
- IBM database traces
- Storage system traces

**Usage:**
```rust
use tracekit_formats::ArcReader;
use std::fs::File;
use std::io::BufReader;

let file = File::open("trace.arc")?;
let mut reader = ArcReader::new(BufReader::new(file));
```

### 2. LIRS Format

**Format:** One block number per line

```text
12345
67890
12345
```

**Sources:**
- Original LIRS paper traces
- [Caffeine simulator resources](https://github.com/ben-manes/caffeine/tree/master/simulator/src/main/resources)

**Usage:**
```rust
use tracekit_formats::LirsReader;
let mut reader = LirsReader::new(BufReader::new(file));
```

### 3. Cachelib Format (Production Traces)

**Format:** CSV with header

```csv
timestamp,key,key_size,value_size,client_id,op_count,ttl
1000,abc123,6,1024,1,1,3600
```

**Sources:**
- [Meta Cachelib traces](https://cachelib.org/docs/Cache_Library_User_Guides/Cachebench_FB_HW_eval/)
- Production CDN workloads
- Social media cache patterns

**Usage:**
```rust
use tracekit_formats::CachelibReader;
let mut reader = CachelibReader::with_defaults(BufReader::new(file));
```

### 4. CSV Format (Universal)

Flexible format for custom traces:

```rust
use tracekit_formats::{CsvConfig, CsvReader};

let config = CsvConfig {
    key_col: 0,
    op_col: Some(1),
    weight_col: Some(2),
    ts_col: Some(3),
    delimiter: ',',
    has_header: true,
};

let mut reader = CsvReader::new(BufReader::new(file), config);
```

## Trace Analysis Workflow

### Step 1: Understand Your Trace

```bash
# Run the analysis example
cargo run --example real_trace < your_trace.txt
```

This shows:
- Total requests
- Unique keys
- Operation distribution (Get/Insert/Delete)
- Average object size
- Reuse distance (temporal locality)

### Step 2: Choose Appropriate Cache Size

```bash
# Test multiple cache sizes
for size in 1000 5000 10000 50000; do
  echo "Testing capacity: $size"
  tracekit simulate --trace trace.txt --format arc --capacity $size
done
```

### Step 3: Compare Cache Policies

```rust
use tracekit::{simulate, EventSource};
use tracekit_formats::ArcReader;
use std::fs::File;

// Your cache implementations
let mut lru_cache = MyLruCache::new(10000);
let mut lfu_cache = MyLfuCache::new(10000);
let mut fifo_cache = MyFifoCache::new(10000);

// Test each policy
let file = File::open("trace.arc")?;
let mut source = ArcReader::new(BufReader::new(file));
let lru_stats = simulate(&mut lru_cache, &mut source);

// Repeat for other policies...
```

## Real Trace Collections

### Academic Research Traces

1. **Twitter Cache Traces**
   - Repository: [twitter/cache-trace](https://github.com/twitter/cache-trace)
   - Content: Production KV cache from Twitter
   - Size: Billions of requests
   - Format: Custom binary (needs conversion)

2. **Meta/Facebook Cachelib Traces**
   - Source: [cachelib.org](https://cachelib.org/)
   - Content: CDN, storage, and social graph caches
   - Format: CSV or binary

3. **SNIA Storage Traces**
   - Source: [iotta.snia.org](http://iotta.snia.org/)
   - Content: Enterprise storage I/O
   - Format: Various (mostly CSV)

4. **WikiBench**
   - Source: [wikibench.eu](http://www.wikibench.eu/)
   - Content: Wikipedia access patterns
   - Format: Custom (needs parsing)

### Converting Traces

```bash
# Convert any format to JSONL for inspection
tracekit rewrite \
  --input trace.arc \
  --input-format arc \
  --output trace.jsonl \
  --output-format jsonl

# Convert to key-only for simplicity
tracekit rewrite \
  --input cachelib.csv \
  --input-format cachelib \
  --output keys.txt \
  --output-format key-only
```

## Advanced Usage

### Handling Large Traces

For traces too large to fit in memory, use streaming:

```rust
use tracekit::{CacheModel, EventSource};
use tracekit_formats::ArcReader;
use std::fs::File;
use std::io::BufReader;

let file = File::open("huge_trace.arc")?;
let reader = BufReader::with_capacity(1024 * 1024, file); // 1MB buffer
let mut source = ArcReader::new(reader);

// Process in streaming fashion
let mut cache = MyCache::new(10000);
let mut hits = 0u64;
let mut misses = 0u64;

while let Some(event) = source.next_event() {
    if cache.get(event.key) {
        hits += 1;
    } else {
        misses += 1;
        cache.insert(event.key);
    }

    // Progress reporting
    if (hits + misses) % 1_000_000 == 0 {
        let rate = hits as f64 / (hits + misses) as f64;
        println!("Processed {}M requests, hit rate: {:.2}%",
                 (hits + misses) / 1_000_000, rate * 100.0);
    }
}
```

### Compressed Traces

Many traces are distributed gzipped. Use `flate2`:

```rust
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::BufReader;
use tracekit_formats::ArcReader;

let file = File::open("trace.arc.gz")?;
let decoder = GzDecoder::new(file);
let reader = BufReader::new(decoder);
let mut source = ArcReader::new(reader);
```

### Custom Trace Parsing

If you have a custom trace format:

```rust
use std::io::BufRead;
use tracekit::{Event, EventSource, Op};

struct MyCustomReader<R> {
    reader: R,
    line: String,
}

impl<R: BufRead> MyCustomReader<R> {
    fn new(reader: R) -> Self {
        Self { reader, line: String::new() }
    }
}

impl<R: BufRead> EventSource for MyCustomReader<R> {
    fn next_event(&mut self) -> Option<Event> {
        loop {
            self.line.clear();
            match self.reader.read_line(&mut self.line) {
                Ok(0) => return None,
                Ok(_) => {
                    // Parse your format here
                    let key = parse_key(&self.line)?;
                    let op = parse_op(&self.line).unwrap_or(Op::Get);
                    return Some(Event { key, op, weight: None, ts: None });
                }
                Err(_) => return None,
            }
        }
    }
}
```

## Best Practices

### 1. Validate Your Parser

Always check that your parser correctly reads the trace:

```bash
# Count lines in original trace
wc -l trace.txt

# Count events read by tracekit
tracekit rewrite --input trace.txt --input-format arc \
  --output /dev/null --output-format key-only
# Should report the same count (minus headers/comments)
```

### 2. Compare with Known Results

If the trace comes with published results, verify you get similar hit rates:

```bash
# Many papers report hit rates at specific cache sizes
# Use those as benchmarks
tracekit simulate --trace S3.lis --format arc --capacity 8192
```

### 3. Start Small

Test with a subset before processing the full trace:

```bash
# Take first 100K lines
head -n 100000 huge_trace.txt > sample.txt
tracekit simulate --trace sample.txt --format arc --capacity 1000
```

### 4. Document Your Traces

Keep metadata about your traces:

```bash
# Create a traces.md file
cat > traces.md <<EOF
# Trace Collection

## S3.lis
- Source: IBM Storage trace from ARC paper
- Size: 4M requests
- Unique keys: ~1M
- Time period: Database workload
- Reference hit rate (capacity=8192): 78.3%

## twitter_cluster52.trace
- Source: Twitter cache trace
- Size: 500M requests
- Characteristics: Heavy tail, high temporal locality
EOF
```

## Troubleshooting

### "Invalid key" or parse errors

Check the trace format matches what you specified:

```bash
# Look at first few lines
head -20 trace.txt

# Try different formats
tracekit simulate --trace trace.txt --format key-only --capacity 100
tracekit simulate --trace trace.txt --format arc --capacity 100
tracekit simulate --trace trace.txt --format csv --capacity 100
```

### Unexpected hit rates

1. Check if the trace includes inserts/writes (affects hit rate calculation)
2. Verify the cache size is appropriate for the working set
3. Look for timestamp discontinuities (trace splicing)

### Performance issues

1. Use `BufReader` with appropriate buffer size (default 8KB)
2. For huge traces, process in chunks or use sampling
3. Profile your cache implementation, not just the parser

## Next Steps

1. **Benchmark your cache**: Use the standard workload suite to establish baseline
2. **Test on real traces**: Validate with at least 3-5 different real-world traces
3. **Compare with literature**: Find papers using the same traces and compare results
4. **Contribute**: Share your trace parsers and analysis with the community

## Resources

- [Caffeine Wiki - Simulator](https://github.com/ben-manes/caffeine/wiki/Simulator)
- [libCacheSim traces](https://github.com/cacheMon/libCacheSim)
- [Research trace repositories](https://github.com/moka-rs/cache-trace)
- [SNIA trace repository](http://iotta.snia.org/)

## Contributing

Found a trace format we don't support? Contributions welcome!

See `tracekit-formats/README.md` for how to add a new parser.
