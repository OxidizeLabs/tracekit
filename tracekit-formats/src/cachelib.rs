//! Cachelib trace format parser.
//!
//! ## Format
//! The Cachelib trace format from Facebook/Meta is a binary format used for
//! cache workload characterization. It's commonly used in production cache
//! simulation and academic research.
//!
//! The format consists of binary records where each record contains:
//! - Operation type (1 byte): 0=get, 1=set, 2=delete
//! - Key size (4 bytes, little-endian)
//! - Key data (variable length)
//! - [Optional] Value size (4 bytes, little-endian, for set operations)
//! - [Optional] TTL (4 bytes, little-endian)
//!
//! ## Simplified CSV Format
//! Many Cachelib traces are also distributed in CSV format for easier analysis:
//! ```csv
//! timestamp,key,key_size,value_size,client_id,op_count,ttl
//! 1000,abc123,6,1024,1,1,3600
//! ```
//!
//! This parser supports the CSV variant. For binary format support, use the
//! `binary` feature flag.
//!
//! ## Source
//! - [Cachelib project](https://cachelib.org/)
//! - [Cachebench traces](https://cachelib.org/docs/Cache_Library_User_Guides/Cachebench_FB_HW_eval/)

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::BufRead;
use tracekit::{Event, EventSource, Op};

/// Configuration for Cachelib CSV parsing.
#[derive(Debug, Clone)]
pub struct CachelibConfig {
    /// Column index for timestamp.
    pub timestamp_col: usize,
    /// Column index for key.
    pub key_col: usize,
    /// Column index for key size.
    pub key_size_col: Option<usize>,
    /// Column index for value size (weight).
    pub value_size_col: Option<usize>,
    /// Column index for operation type.
    pub op_col: Option<usize>,
    /// Whether the first line is a header.
    pub has_header: bool,
}

impl Default for CachelibConfig {
    fn default() -> Self {
        Self {
            timestamp_col: 0,
            key_col: 1,
            key_size_col: Some(2),
            value_size_col: Some(3),
            op_col: None, // If not present, default to Get
            has_header: true,
        }
    }
}

/// Reads traces in Cachelib CSV format.
pub struct CachelibReader<R> {
    reader: R,
    config: CachelibConfig,
    line: String,
    first_line: bool,
}

impl<R: BufRead> CachelibReader<R> {
    /// Create a new Cachelib reader with the given configuration.
    pub fn new(reader: R, config: CachelibConfig) -> Self {
        Self {
            reader,
            config,
            line: String::new(),
            first_line: true,
        }
    }

    /// Create a Cachelib reader with default configuration.
    pub fn with_defaults(reader: R) -> Self {
        Self::new(reader, CachelibConfig::default())
    }

    /// Returns a reference to the underlying reader.
    pub fn inner(&self) -> &R {
        &self.reader
    }

    /// Consumes the reader and returns the underlying source.
    pub fn into_inner(self) -> R {
        self.reader
    }

    /// Hash a string key to u64 (for non-numeric keys).
    fn hash_key(key: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    fn parse_op(s: &str) -> Op {
        match s.to_lowercase().as_str() {
            "set" | "add" | "1" => Op::Insert,
            "delete" | "del" | "2" => Op::Delete,
            "get" | "0" => Op::Get,
            _ => Op::Get,
        }
    }
}

impl<R: BufRead> EventSource for CachelibReader<R> {
    fn next_event(&mut self) -> Option<Event> {
        loop {
            self.line.clear();
            match self.reader.read_line(&mut self.line) {
                Ok(0) => return None, // EOF
                Ok(_) => {
                    // Skip header if configured
                    if self.first_line && self.config.has_header {
                        self.first_line = false;
                        continue;
                    }
                    self.first_line = false;

                    let trimmed = self.line.trim();
                    // Skip empty lines and comments
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }

                    let parts: Vec<&str> = trimmed.split(',').collect();

                    // Parse key (required)
                    if parts.len() <= self.config.key_col {
                        continue;
                    }
                    let key_str = parts[self.config.key_col].trim();
                    let key = key_str
                        .parse::<u64>()
                        .unwrap_or_else(|_| Self::hash_key(key_str));

                    // Parse timestamp (optional, for Event.ts)
                    let ts = if parts.len() > self.config.timestamp_col {
                        parts[self.config.timestamp_col].trim().parse::<u64>().ok()
                    } else {
                        None
                    };

                    // Parse value size as weight (optional)
                    let weight = if let Some(col) = self.config.value_size_col {
                        if parts.len() > col {
                            parts[col].trim().parse::<u32>().ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Parse operation (optional)
                    let op = if let Some(col) = self.config.op_col {
                        if parts.len() > col && !parts[col].trim().is_empty() {
                            Self::parse_op(parts[col].trim())
                        } else {
                            Op::Get
                        }
                    } else {
                        Op::Get
                    };

                    let mut event = Event {
                        key,
                        op,
                        weight,
                        ts,
                    };
                    if let Some(w) = weight {
                        event = event.with_weight(w);
                    }
                    if let Some(t) = ts {
                        event = event.with_ts(t);
                    }

                    return Some(event);
                }
                Err(_) => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_cachelib_reader_with_header() {
        let data = "timestamp,key,key_size,value_size,client_id,op_count,ttl
1000,12345,5,1024,1,1,3600
2000,67890,5,2048,1,2,3600
";
        let cursor = Cursor::new(data);
        let mut reader = CachelibReader::with_defaults(cursor);

        let e1 = reader.next_event().unwrap();
        assert_eq!(e1.key, 12345);
        assert_eq!(e1.weight, Some(1024));
        assert_eq!(e1.ts, Some(1000));

        let e2 = reader.next_event().unwrap();
        assert_eq!(e2.key, 67890);
        assert_eq!(e2.weight, Some(2048));
        assert_eq!(e2.ts, Some(2000));

        assert!(reader.next_event().is_none());
    }

    #[test]
    fn test_cachelib_reader_string_keys() {
        let data = "timestamp,key,key_size,value_size
1000,abc123,6,1024
2000,def456,6,2048
";
        let cursor = Cursor::new(data);
        let mut reader = CachelibReader::with_defaults(cursor);

        let e1 = reader.next_event().unwrap();
        // String keys are hashed to u64
        assert!(e1.key > 0);
        assert_eq!(e1.weight, Some(1024));

        let e2 = reader.next_event().unwrap();
        assert!(e2.key > 0);
        assert_eq!(e2.weight, Some(2048));

        assert!(reader.next_event().is_none());
    }

    #[test]
    fn test_cachelib_reader_no_header() {
        let data = "1000,12345,5,1024\n2000,67890,5,2048\n";
        let cursor = Cursor::new(data);
        let config = CachelibConfig {
            has_header: false,
            ..Default::default()
        };
        let mut reader = CachelibReader::new(cursor, config);

        let e1 = reader.next_event().unwrap();
        assert_eq!(e1.key, 12345);
        assert_eq!(e1.weight, Some(1024));

        let e2 = reader.next_event().unwrap();
        assert_eq!(e2.key, 67890);

        assert!(reader.next_event().is_none());
    }
}
