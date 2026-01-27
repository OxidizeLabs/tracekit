//! Key-only trace format (one key per line).
//!
//! This is the simplest trace format: one integer key per line.
//! All events are interpreted as Get operations.
//!
//! ## Example
//! ```text
//! 12345
//! 67890
//! 12345
//! 11111
//! ```

use std::io::{BufRead, Write};
use tracekit::{Event, EventSource};

/// Reads traces in key-only format (one key per line).
///
/// Each line is parsed as a u64 key and emitted as a Get event.
/// Invalid lines are skipped.
pub struct KeyOnlyReader<R> {
    reader: R,
    line: String,
}

impl<R: BufRead> KeyOnlyReader<R> {
    /// Create a new reader from any `BufRead` source.
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

impl<R: BufRead> EventSource for KeyOnlyReader<R> {
    fn next_event(&mut self) -> Option<Event> {
        loop {
            self.line.clear();
            match self.reader.read_line(&mut self.line) {
                Ok(0) => return None, // EOF
                Ok(_) => {
                    let trimmed = self.line.trim();
                    if trimmed.is_empty() {
                        continue; // Skip empty lines
                    }
                    if let Ok(key) = trimmed.parse::<u64>() {
                        return Some(Event::get(key));
                    }
                    // Skip invalid lines
                    continue;
                }
                Err(_) => return None,
            }
        }
    }
}

/// Writes traces in key-only format (one key per line).
pub struct KeyOnlyWriter<W> {
    writer: W,
}

impl<W: Write> KeyOnlyWriter<W> {
    /// Create a new writer.
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Write a single event (only the key is written).
    pub fn write_event(&mut self, event: &Event) -> std::io::Result<()> {
        writeln!(self.writer, "{}", event.key)
    }

    /// Write a key directly.
    pub fn write_key(&mut self, key: u64) -> std::io::Result<()> {
        writeln!(self.writer, "{}", key)
    }

    /// Flush the underlying writer.
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }

    /// Consumes the writer and returns the underlying sink.
    pub fn into_inner(self) -> W {
        self.writer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_key_only_reader() {
        let data = "123\n456\n789\n";
        let cursor = Cursor::new(data);
        let mut reader = KeyOnlyReader::new(cursor);

        assert_eq!(reader.next_event(), Some(Event::get(123)));
        assert_eq!(reader.next_event(), Some(Event::get(456)));
        assert_eq!(reader.next_event(), Some(Event::get(789)));
        assert_eq!(reader.next_event(), None);
    }

    #[test]
    fn test_key_only_reader_skips_invalid() {
        let data = "123\ninvalid\n456\n\n789\n";
        let cursor = Cursor::new(data);
        let mut reader = KeyOnlyReader::new(cursor);

        assert_eq!(reader.next_event(), Some(Event::get(123)));
        assert_eq!(reader.next_event(), Some(Event::get(456)));
        assert_eq!(reader.next_event(), Some(Event::get(789)));
        assert_eq!(reader.next_event(), None);
    }

    #[test]
    fn test_key_only_writer() {
        let mut buffer = Vec::new();
        {
            let mut writer = KeyOnlyWriter::new(&mut buffer);
            writer.write_key(123).unwrap();
            writer.write_key(456).unwrap();
            writer.flush().unwrap();
        }
        assert_eq!(String::from_utf8(buffer).unwrap(), "123\n456\n");
    }
}
