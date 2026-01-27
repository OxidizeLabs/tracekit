# tracekit-formats

Trace file format parsers and writers for tracekit.

## Supported Formats

### Simple Text Formats

#### Key-Only Format
The simplest format: one key per line.

```text
12345
67890
12345
11111
```

**Usage:**
```rust
use tracekit_formats::KeyOnlyReader;
use std::fs::File;
use std::io::BufReader;

let file = File::open("trace.txt")?;
let mut reader = KeyOnlyReader::new(BufReader::new(file));
```

#### LIRS Format
Used in traces from the LIRS (Low Inter-reference Recency Set) paper. Same format as key-only, but semantically represents block numbers from storage traces.

```text
12345
67890
12345
```

**Source:** [LIRS paper traces](https://github.com/ben-manes/caffeine/tree/master/simulator/src/main/resources/com/github/benmanes/caffeine/cache/simulator/parser/lirs)

**Usage:**
```rust
use tracekit_formats::LirsReader;
let mut reader = LirsReader::new(BufReader::new(file));
```

### Structured Text Formats

#### ARC Format
Space-separated format from ARC (Adaptive Replacement Cache) research.

**Format:** `timestamp key [size]`

```text
1 12345 4096
2 67890 8192
3 12345 4096
```

**Source:** [moka-rs/cache-trace](https://github.com/moka-rs/cache-trace/tree/main/arc)

**Usage:**
```rust
use tracekit_formats::ArcReader;
let mut reader = ArcReader::new(BufReader::new(file));
```

#### CSV Format
Flexible CSV format with configurable columns.

```csv
key,op,weight,timestamp
12345,get,4096,1000
67890,insert,8192,2000
11111,delete,,3000
```

**Usage:**
```rust
use tracekit_formats::{CsvConfig, CsvReader};

// Default configuration (key, op, weight, ts)
let mut reader = CsvReader::with_defaults(BufReader::new(file));

// Custom configuration
let config = CsvConfig {
    key_col: 0,
    op_col: Some(1),
    weight_col: Some(2),
    ts_col: None,
    delimiter: ',',
    has_header: true,
};
let mut reader = CsvReader::new(BufReader::new(file), config);

// Key-only CSV
let config = CsvConfig::key_only();
let mut reader = CsvReader::new(BufReader::new(file), config);

// Tab-separated values
let config = CsvConfig::tsv();
let mut reader = CsvReader::new(BufReader::new(file), config);
```

#### JSON Lines (JSONL) Format
JSON objects, one per line. Supports the full Event model.

```jsonl
{"key":12345}
{"key":67890,"op":"insert","weight":4096}
{"key":12345,"op":"get","ts":1000}
{"key":11111,"op":"delete"}
```

**Feature flag:** `jsonl`

**Usage:**
```rust
use tracekit_formats::{JsonlReader, JsonlWriter};

// Reading
let mut reader = JsonlReader::new(BufReader::new(file));

// Writing
let mut writer = JsonlWriter::new(BufWriter::new(file));
writer.write_event(&event)?;
writer.flush()?;
```

#### Cachelib Format
CSV format from Facebook/Meta's Cachelib project.

**Format:** `timestamp,key,key_size,value_size,client_id,op_count,ttl`

```csv
timestamp,key,key_size,value_size,client_id,op_count,ttl
1000,12345,5,1024,1,1,3600
2000,abc123,6,2048,1,2,3600
```

Supports both numeric and string keys (string keys are hashed to u64).

**Source:** [Cachelib Cachebench traces](https://cachelib.org/docs/Cache_Library_User_Guides/Cachebench_FB_HW_eval/)

**Feature flag:** `cachelib`

**Usage:**
```rust
use tracekit_formats::CachelibReader;

// Default configuration
let mut reader = CachelibReader::with_defaults(BufReader::new(file));

// Custom configuration
use tracekit_formats::CachelibConfig;
let config = CachelibConfig {
    timestamp_col: 0,
    key_col: 1,
    key_size_col: Some(2),
    value_size_col: Some(3),
    op_col: None,
    has_header: true,
};
let mut reader = CachelibReader::new(BufReader::new(file), config);
```

## CLI Usage

### Simulate with different formats

```bash
# Key-only format
tracekit simulate --trace trace.txt --format key-only --capacity 1000

# ARC format
tracekit simulate --trace arc_trace.txt --format arc --capacity 1000

# LIRS format
tracekit simulate --trace lirs_trace.txt --format lirs --capacity 1000

# CSV format
tracekit simulate --trace trace.csv --format csv --capacity 1000

# Cachelib format
tracekit simulate --trace cachelib_trace.csv --format cachelib --capacity 1000

# JSONL format
tracekit simulate --trace trace.jsonl --format jsonl --capacity 1000
```

### Convert between formats

```bash
# ARC to JSONL
tracekit rewrite --input trace.arc --input-format arc \
  --output trace.jsonl --output-format jsonl

# Cachelib to key-only
tracekit rewrite --input cachelib.csv --input-format cachelib \
  --output keys.txt --output-format key-only

# CSV to JSONL
tracekit rewrite --input trace.csv --input-format csv \
  --output trace.jsonl --output-format jsonl
```

## Where to Get Real Traces

### Academic Traces

1. **ARC Traces**
   - [moka-rs/cache-trace](https://github.com/moka-rs/cache-trace/tree/main/arc)
   - Various cache simulation traces

2. **LIRS Traces**
   - [Caffeine simulator resources](https://github.com/ben-manes/caffeine/tree/master/simulator/src/main/resources/com/github/benmanes/caffeine/cache/simulator/parser/lirs)
   - Storage and database workload traces

3. **Twitter Cache Traces**
   - [Twitter cache traces](https://github.com/twitter/cache-trace)
   - Production cache workload from Twitter (now X)

4. **Meta/Facebook Traces**
   - [Cachelib traces](https://cachelib.org/docs/Cache_Library_User_Guides/Cachebench_FB_HW_eval/)
   - Production cache workloads from Meta

5. **SNIA Storage Traces**
   - [SNIA IOTTA Repository](http://iotta.snia.org/)
   - Enterprise storage workloads

6. **WikiBench**
   - [WikiBench traces](http://www.wikibench.eu/)
   - Wikipedia access patterns

### Research Papers with Traces

- **ARC Paper:** Megiddo & Modha, "ARC: A Self-Tuning, Low Overhead Replacement Cache" (FAST 2003)
- **LIRS Paper:** Jiang & Zhang, "LIRS: An Efficient Low Inter-reference Recency Set Replacement Policy" (SIGMETRICS 2002)
- **LRB Paper:** Yang et al., "Learning Relaxed Belady for Content Distribution Network Caching" (NSDI 2020)

## Feature Flags

- `arc`: Enable ARC format support (default)
- `lirs`: Enable LIRS format support (default)
- `csv`: Enable CSV format support (default)
- `cachelib`: Enable Cachelib format support
- `jsonl`: Enable JSONL format support
- `compression`: Enable gzip compression support (future)
- `full`: Enable all features

```toml
[dependencies]
tracekit-formats = { version = "0.1", features = ["full"] }
```

## Adding New Formats

To add a new trace format:

1. Create a new module in `src/` (e.g., `src/myformat.rs`)
2. Implement `EventSource` trait for your reader
3. Add the module to `lib.rs`
4. Add to the CLI enum in `tracekit-cli/src/cmd_simulate.rs`
5. Update documentation

Example skeleton:

```rust
use std::io::BufRead;
use tracekit::{Event, EventSource};

pub struct MyFormatReader<R> {
    reader: R,
    // ... state fields
}

impl<R: BufRead> MyFormatReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R: BufRead> EventSource for MyFormatReader<R> {
    fn next_event(&mut self) -> Option<Event> {
        // Parse your format here
        todo!()
    }
}
```

## License

Dual-licensed under MIT and Apache-2.0.
