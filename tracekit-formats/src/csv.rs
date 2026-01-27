//! CSV trace format parser.
//!
//! ## Format
//! A flexible CSV format that supports various column layouts. By default, expects:
//!
//! ```csv
//! key,op,weight,timestamp
//! 12345,get,,
//! 67890,insert,4096,1000
//! 12345,delete,,
//! ```
//!
//! The parser is configurable to handle:
//! - Different column orders
//! - Optional headers
//! - Different delimiters (comma, tab, space)
//! - Missing columns (defaults: op=get, weight=None, ts=None)
//!
//! ## Example with header
//! ```csv
//! key,operation,size
//! 12345,read,4096
//! 67890,write,8192
//! ```
//!
//! ## Example without header (positional)
//! ```text
//! 12345
//! 67890,insert
//! 11111,get,4096
//! ```

use std::io::BufRead;
use tracekit::{Event, EventSource, Op};

/// Configuration for CSV parsing.
#[derive(Debug, Clone)]
pub struct CsvConfig {
    /// Column index for key (0-based).
    pub key_col: usize,
    /// Column index for operation (None if not present).
    pub op_col: Option<usize>,
    /// Column index for weight (None if not present).
    pub weight_col: Option<usize>,
    /// Column index for timestamp (None if not present).
    pub ts_col: Option<usize>,
    /// Delimiter character.
    pub delimiter: char,
    /// Whether the first line is a header (skip it).
    pub has_header: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        Self {
            key_col: 0,
            op_col: Some(1),
            weight_col: Some(2),
            ts_col: Some(3),
            delimiter: ',',
            has_header: false,
        }
    }
}

impl CsvConfig {
    /// Simple key-only CSV (one column).
    pub fn key_only() -> Self {
        Self {
            key_col: 0,
            op_col: None,
            weight_col: None,
            ts_col: None,
            delimiter: ',',
            has_header: false,
        }
    }

    /// Tab-separated format.
    pub fn tsv() -> Self {
        Self {
            delimiter: '\t',
            ..Default::default()
        }
    }
}

/// Reads traces in CSV format with configurable columns.
pub struct CsvReader<R> {
    reader: R,
    config: CsvConfig,
    line: String,
    first_line: bool,
}

impl<R: BufRead> CsvReader<R> {
    /// Create a new CSV reader with the given configuration.
    pub fn new(reader: R, config: CsvConfig) -> Self {
        Self {
            reader,
            config,
            line: String::new(),
            first_line: true,
        }
    }

    /// Create a CSV reader with default configuration.
    pub fn with_defaults(reader: R) -> Self {
        Self::new(reader, CsvConfig::default())
    }

    /// Returns a reference to the underlying reader.
    pub fn inner(&self) -> &R {
        &self.reader
    }

    /// Consumes the reader and returns the underlying source.
    pub fn into_inner(self) -> R {
        self.reader
    }

    fn parse_op(s: &str) -> Op {
        match s.to_lowercase().as_str() {
            "insert" | "write" | "set" | "put" | "w" => Op::Insert,
            "delete" | "remove" | "del" | "d" => Op::Delete,
            _ => Op::Get,
        }
    }
}

impl<R: BufRead> EventSource for CsvReader<R> {
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

                    let parts: Vec<&str> = trimmed.split(self.config.delimiter).collect();

                    // Parse key (required)
                    if parts.len() <= self.config.key_col {
                        continue; // Not enough columns
                    }
                    let key = match parts[self.config.key_col].trim().parse::<u64>() {
                        Ok(k) => k,
                        Err(_) => continue, // Skip invalid key
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

                    // Parse weight (optional)
                    let weight = if let Some(col) = self.config.weight_col {
                        if parts.len() > col {
                            parts[col].trim().parse::<u32>().ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Parse timestamp (optional)
                    let ts = if let Some(col) = self.config.ts_col {
                        if parts.len() > col {
                            parts[col].trim().parse::<u64>().ok()
                        } else {
                            None
                        }
                    } else {
                        None
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
    fn test_csv_reader_full() {
        let data = "12345,get,4096,1000\n67890,insert,8192,2000\n11111,delete,,3000\n";
        let cursor = Cursor::new(data);
        let mut reader = CsvReader::with_defaults(cursor);

        let e1 = reader.next_event().unwrap();
        assert_eq!(e1.key, 12345);
        assert_eq!(e1.op, Op::Get);
        assert_eq!(e1.weight, Some(4096));
        assert_eq!(e1.ts, Some(1000));

        let e2 = reader.next_event().unwrap();
        assert_eq!(e2.key, 67890);
        assert_eq!(e2.op, Op::Insert);

        let e3 = reader.next_event().unwrap();
        assert_eq!(e3.key, 11111);
        assert_eq!(e3.op, Op::Delete);
        assert_eq!(e3.weight, None);

        assert!(reader.next_event().is_none());
    }

    #[test]
    fn test_csv_reader_key_only() {
        let data = "12345\n67890\n11111\n";
        let cursor = Cursor::new(data);
        let config = CsvConfig::key_only();
        let mut reader = CsvReader::new(cursor, config);

        let e1 = reader.next_event().unwrap();
        assert_eq!(e1.key, 12345);
        assert_eq!(e1.op, Op::Get);

        let e2 = reader.next_event().unwrap();
        assert_eq!(e2.key, 67890);

        let e3 = reader.next_event().unwrap();
        assert_eq!(e3.key, 11111);

        assert!(reader.next_event().is_none());
    }

    #[test]
    fn test_csv_reader_with_header() {
        let data = "key,operation,size\n12345,read,4096\n67890,write,8192\n";
        let cursor = Cursor::new(data);
        let config = CsvConfig {
            has_header: true,
            ..Default::default()
        };
        let mut reader = CsvReader::new(cursor, config);

        let e1 = reader.next_event().unwrap();
        assert_eq!(e1.key, 12345);

        let e2 = reader.next_event().unwrap();
        assert_eq!(e2.key, 67890);

        assert!(reader.next_event().is_none());
    }

    #[test]
    fn test_csv_reader_tsv() {
        let data = "12345\tget\t4096\n67890\tinsert\t8192\n";
        let cursor = Cursor::new(data);
        let config = CsvConfig::tsv();
        let mut reader = CsvReader::new(cursor, config);

        let e1 = reader.next_event().unwrap();
        assert_eq!(e1.key, 12345);
        assert_eq!(e1.weight, Some(4096));

        let e2 = reader.next_event().unwrap();
        assert_eq!(e2.key, 67890);
        assert_eq!(e2.op, Op::Insert);

        assert!(reader.next_event().is_none());
    }
}
