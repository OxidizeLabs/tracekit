//! Example: Reading and analyzing real-world cache traces.
//!
//! This example demonstrates how to:
//! 1. Read traces in various formats (ARC, LIRS, CSV, Cachelib)
//! 2. Perform basic trace analysis
//! 3. Compare trace characteristics
//!
//! Run with: cargo run --example real_trace

use std::collections::HashSet;
use tracekit::{Event, EventSource};

fn main() {
    println!("=== Real Trace Analysis Example ===\n");

    // Example 1: ARC format trace
    println!("1. Parsing ARC format trace:");
    let arc_data = "1 12345 4096\n2 67890 8192\n3 12345 4096\n4 11111 2048\n";
    analyze_trace("ARC", arc_data.as_bytes());

    // Example 2: LIRS format trace
    println!("\n2. Parsing LIRS format trace:");
    let lirs_data = "12345\n67890\n12345\n11111\n67890\n";
    analyze_trace("LIRS", lirs_data.as_bytes());

    // Example 3: CSV format trace
    println!("\n3. Parsing CSV format trace:");
    let csv_data = "key,op,weight\n12345,get,4096\n67890,insert,8192\n12345,get,4096\n";
    analyze_trace("CSV", csv_data.as_bytes());

    // Example 4: Cachelib format trace
    println!("\n4. Parsing Cachelib format trace:");
    let cachelib_data = "timestamp,key,key_size,value_size
1000,12345,5,1024
2000,67890,5,2048
3000,12345,5,1024
";
    analyze_trace("Cachelib", cachelib_data.as_bytes());

    println!("\n=== Analysis Complete ===");
    println!("\nTo analyze your own traces:");
    println!("  cargo run --example real_trace < your_trace.txt");
}

fn analyze_trace(format: &str, data: &[u8]) {
    use std::io::Cursor;
    use tracekit_formats::*;

    let cursor = Cursor::new(data);
    let mut source: Box<dyn EventSource> = match format {
        "ARC" => Box::new(ArcReader::new(cursor)),
        "LIRS" => Box::new(LirsReader::new(cursor)),
        "CSV" => Box::new(CsvReader::with_defaults(cursor)),
        "Cachelib" => Box::new(CachelibReader::with_defaults(cursor)),
        _ => Box::new(KeyOnlyReader::new(cursor)),
    };

    let mut stats = TraceStats::default();

    while let Some(event) = source.next_event() {
        stats.process(event);
    }

    stats.print(format);
}

#[derive(Default)]
struct TraceStats {
    total_requests: u64,
    unique_keys: HashSet<u64>,
    gets: u64,
    inserts: u64,
    deletes: u64,
    total_bytes: u64,
    requests_with_weight: u64,
}

impl TraceStats {
    fn process(&mut self, event: Event) {
        self.total_requests += 1;
        self.unique_keys.insert(event.key);

        match event.op {
            tracekit::Op::Get => self.gets += 1,
            tracekit::Op::Insert => self.inserts += 1,
            tracekit::Op::Delete => self.deletes += 1,
        }

        if let Some(weight) = event.weight {
            self.total_bytes += weight as u64;
            self.requests_with_weight += 1;
        }
    }

    fn print(&self, format: &str) {
        println!("  Format: {}", format);
        println!("  Total requests: {}", self.total_requests);
        println!("  Unique keys: {}", self.unique_keys.len());
        println!("  Operations:");
        println!(
            "    - Gets: {} ({:.1}%)",
            self.gets,
            100.0 * self.gets as f64 / self.total_requests as f64
        );
        println!(
            "    - Inserts: {} ({:.1}%)",
            self.inserts,
            100.0 * self.inserts as f64 / self.total_requests as f64
        );
        println!(
            "    - Deletes: {} ({:.1}%)",
            self.deletes,
            100.0 * self.deletes as f64 / self.total_requests as f64
        );

        if self.requests_with_weight > 0 {
            let avg_size = self.total_bytes / self.requests_with_weight;
            println!("  Average object size: {} bytes", avg_size);
            println!("  Total data volume: {} bytes", self.total_bytes);
        }

        let reuse_distance = self.total_requests as f64 / self.unique_keys.len() as f64;
        println!("  Average reuse distance: {:.2}", reuse_distance);
    }
}
