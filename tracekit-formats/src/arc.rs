//! ARC trace format parser.
//!
//! ## Format
//! The ARC (Adaptive Replacement Cache) trace format is a simple space-separated
//! text format used in cache simulation research. Each line contains:
//!
//! ```text
//! timestamp key [size]
//! ```
//!
//! - `timestamp`: Unix timestamp or logical time (ignored in basic simulation)
//! - `key`: Block/object identifier (parsed as u64)
//! - `size`: Optional size in bytes (for size-aware policies)
//!
//! ## Example
//! ```text
//! 1 12345 4096
//! 2 67890 8192
//! 3 12345 4096
//! ```
//!
//! ## Source
//! Traces in this format are commonly available in cache simulation research:
//! - [moka-rs/cache-trace](https://github.com/moka-rs/cache-trace/tree/main/arc)
//! - Various academic papers on cache replacement policies

use std::io::BufRead;
use tracekit::{Event, EventSource};

/// Reads traces in ARC format (space-separated: timestamp key [size]).
pub struct ArcReader<R> {
    reader: R,
    line: String,
}

impl<R: BufRead> ArcReader<R> {
    /// Create a new ARC reader.
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            line: String::new(),
        }
    }

    /// Returns a reference to the underlying reader.
    pub fn inner(&self) -> &R {
        &self.reader
    }

    /// Consumes the reader and returns the underlying source.
    pub fn into_inner(self) -> R {
        self.reader
    }
}

impl<R: BufRead> EventSource for ArcReader<R> {
    fn next_event(&mut self) -> Option<Event> {
        loop {
            self.line.clear();
            match self.reader.read_line(&mut self.line) {
                Ok(0) => return None, // EOF
                Ok(_) => {
                    let trimmed = self.line.trim();
                    // Skip empty lines and comments
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }

                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() < 2 {
                        continue; // Invalid line, skip
                    }

                    // Parse: timestamp key [size]
                    let key = match parts[1].parse::<u64>() {
                        Ok(k) => k,
                        Err(_) => continue, // Skip invalid key
                    };

                    let weight = if parts.len() >= 3 {
                        parts[2].parse::<u32>().ok()
                    } else {
                        None
                    };

                    // ARC format only contains Gets (lookups), no explicit inserts/deletes
                    let event = Event::get(key);
                    let event = if let Some(w) = weight {
                        event.with_weight(w)
                    } else {
                        event
                    };

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
    use tracekit::Op;

    #[test]
    fn test_arc_reader_basic() {
        let data = "1 12345 4096\n2 67890 8192\n3 12345 4096\n";
        let cursor = Cursor::new(data);
        let mut reader = ArcReader::new(cursor);

        let e1 = reader.next_event().unwrap();
        assert_eq!(e1.key, 12345);
        assert_eq!(e1.op, Op::Get);
        assert_eq!(e1.weight, Some(4096));

        let e2 = reader.next_event().unwrap();
        assert_eq!(e2.key, 67890);
        assert_eq!(e2.weight, Some(8192));

        let e3 = reader.next_event().unwrap();
        assert_eq!(e3.key, 12345);

        assert!(reader.next_event().is_none());
    }

    #[test]
    fn test_arc_reader_no_size() {
        let data = "1 12345\n2 67890\n";
        let cursor = Cursor::new(data);
        let mut reader = ArcReader::new(cursor);

        let e1 = reader.next_event().unwrap();
        assert_eq!(e1.key, 12345);
        assert_eq!(e1.weight, None);

        let e2 = reader.next_event().unwrap();
        assert_eq!(e2.key, 67890);
        assert_eq!(e2.weight, None);
    }

    #[test]
    fn test_arc_reader_skip_comments() {
        let data = "# Comment line\n1 12345 4096\n# Another comment\n2 67890 8192\n";
        let cursor = Cursor::new(data);
        let mut reader = ArcReader::new(cursor);

        let e1 = reader.next_event().unwrap();
        assert_eq!(e1.key, 12345);

        let e2 = reader.next_event().unwrap();
        assert_eq!(e2.key, 67890);

        assert!(reader.next_event().is_none());
    }

    #[test]
    fn test_arc_reader_skip_invalid() {
        let data = "1 12345 4096\ninvalid line\n2 67890 8192\n";
        let cursor = Cursor::new(data);
        let mut reader = ArcReader::new(cursor);

        let e1 = reader.next_event().unwrap();
        assert_eq!(e1.key, 12345);

        let e2 = reader.next_event().unwrap();
        assert_eq!(e2.key, 67890);

        assert!(reader.next_event().is_none());
    }
}
