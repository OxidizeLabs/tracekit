//! LIRS trace format parser.
//!
//! ## Format
//! The LIRS (Low Inter-reference Recency Set) trace format is a simple newline-separated
//! format where each line contains a single block number. This format is used in traces
//! from the original LIRS paper and related research.
//!
//! Each line contains:
//! ```text
//! block_number
//! ```
//!
//! The block number is treated as a cache key. All accesses are treated as Get operations.
//!
//! ## Example
//! ```text
//! 12345
//! 67890
//! 12345
//! 11111
//! ```
//!
//! ## Source
//! - Original LIRS paper traces
//! - Storage workload traces from filesystem and database benchmarks

use std::io::BufRead;
use tracekit::{Event, EventSource};

/// Reads traces in LIRS format (one block number per line).
///
/// This format is identical to the `KeyOnlyReader` but is provided separately
/// to maintain semantic clarity about the trace source and to allow for
/// future LIRS-specific extensions.
pub struct LirsReader<R> {
    reader: R,
    line: String,
}

impl<R: BufRead> LirsReader<R> {
    /// Create a new LIRS reader.
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

impl<R: BufRead> EventSource for LirsReader<R> {
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

                    match trimmed.parse::<u64>() {
                        Ok(key) => return Some(Event::get(key)),
                        Err(_) => continue, // Skip invalid lines
                    }
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
    fn test_lirs_reader_basic() {
        let data = "12345\n67890\n12345\n11111\n";
        let cursor = Cursor::new(data);
        let mut reader = LirsReader::new(cursor);

        let e1 = reader.next_event().unwrap();
        assert_eq!(e1.key, 12345);
        assert_eq!(e1.op, Op::Get);

        let e2 = reader.next_event().unwrap();
        assert_eq!(e2.key, 67890);

        let e3 = reader.next_event().unwrap();
        assert_eq!(e3.key, 12345);

        let e4 = reader.next_event().unwrap();
        assert_eq!(e4.key, 11111);

        assert!(reader.next_event().is_none());
    }

    #[test]
    fn test_lirs_reader_skip_empty() {
        let data = "12345\n\n67890\n\n\n11111\n";
        let cursor = Cursor::new(data);
        let mut reader = LirsReader::new(cursor);

        let e1 = reader.next_event().unwrap();
        assert_eq!(e1.key, 12345);

        let e2 = reader.next_event().unwrap();
        assert_eq!(e2.key, 67890);

        let e3 = reader.next_event().unwrap();
        assert_eq!(e3.key, 11111);

        assert!(reader.next_event().is_none());
    }

    #[test]
    fn test_lirs_reader_skip_comments() {
        let data = "# Comment\n12345\n# Another comment\n67890\n";
        let cursor = Cursor::new(data);
        let mut reader = LirsReader::new(cursor);

        let e1 = reader.next_event().unwrap();
        assert_eq!(e1.key, 12345);

        let e2 = reader.next_event().unwrap();
        assert_eq!(e2.key, 67890);

        assert!(reader.next_event().is_none());
    }
}
