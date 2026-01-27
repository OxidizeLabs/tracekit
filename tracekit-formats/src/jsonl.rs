//! JSONL trace format (one JSON object per line).
//!
//! This format supports the full Event model including operation type,
//! weight, and timestamp.
//!
//! ## Example
//! ```text
//! {"key":12345}
//! {"key":67890,"op":"insert"}
//! {"key":12345,"op":"get","weight":100}
//! {"key":11111,"op":"delete"}
//! ```

use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};
use tracekit::{Event, EventSource, Op};

/// JSON representation of an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonEvent {
    key: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    op: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    weight: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ts: Option<u64>,
}

impl From<JsonEvent> for Event {
    fn from(je: JsonEvent) -> Self {
        let op = match je.op.as_deref() {
            Some("insert") | Some("Insert") | Some("INSERT") => Op::Insert,
            Some("delete") | Some("Delete") | Some("DELETE") => Op::Delete,
            _ => Op::Get,
        };
        Event {
            key: je.key,
            op,
            weight: je.weight,
            ts: je.ts,
        }
    }
}

impl From<&Event> for JsonEvent {
    fn from(e: &Event) -> Self {
        let op = match e.op {
            Op::Get => None, // Default, don't serialize
            Op::Insert => Some("insert".to_string()),
            Op::Delete => Some("delete".to_string()),
        };
        JsonEvent {
            key: e.key,
            op,
            weight: e.weight,
            ts: e.ts,
        }
    }
}

/// Reads traces in JSONL format (one JSON object per line).
pub struct JsonlReader<R> {
    reader: R,
    line: String,
}

impl<R: BufRead> JsonlReader<R> {
    /// Create a new JSONL reader.
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

impl<R: BufRead> EventSource for JsonlReader<R> {
    fn next_event(&mut self) -> Option<Event> {
        loop {
            self.line.clear();
            match self.reader.read_line(&mut self.line) {
                Ok(0) => return None, // EOF
                Ok(_) => {
                    let trimmed = self.line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match serde_json::from_str::<JsonEvent>(trimmed) {
                        Ok(je) => return Some(je.into()),
                        Err(_) => continue, // Skip invalid lines
                    }
                }
                Err(_) => return None,
            }
        }
    }
}

/// Writes traces in JSONL format (one JSON object per line).
pub struct JsonlWriter<W> {
    writer: W,
}

impl<W: Write> JsonlWriter<W> {
    /// Create a new JSONL writer.
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Write a single event.
    pub fn write_event(&mut self, event: &Event) -> std::io::Result<()> {
        let je = JsonEvent::from(event);
        serde_json::to_writer(&mut self.writer, &je)?;
        writeln!(self.writer)
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
    fn test_jsonl_reader_basic() {
        let data = r#"{"key":123}
{"key":456,"op":"insert"}
{"key":789,"op":"delete"}
"#;
        let cursor = Cursor::new(data);
        let mut reader = JsonlReader::new(cursor);

        let e1 = reader.next_event().unwrap();
        assert_eq!(e1.key, 123);
        assert_eq!(e1.op, Op::Get);

        let e2 = reader.next_event().unwrap();
        assert_eq!(e2.key, 456);
        assert_eq!(e2.op, Op::Insert);

        let e3 = reader.next_event().unwrap();
        assert_eq!(e3.key, 789);
        assert_eq!(e3.op, Op::Delete);

        assert!(reader.next_event().is_none());
    }

    #[test]
    fn test_jsonl_reader_with_weight_ts() {
        let data = r#"{"key":123,"weight":100,"ts":1000}"#;
        let cursor = Cursor::new(data);
        let mut reader = JsonlReader::new(cursor);

        let e = reader.next_event().unwrap();
        assert_eq!(e.key, 123);
        assert_eq!(e.weight, Some(100));
        assert_eq!(e.ts, Some(1000));
    }

    #[test]
    fn test_jsonl_writer() {
        let mut buffer = Vec::new();
        {
            let mut writer = JsonlWriter::new(&mut buffer);
            writer.write_event(&Event::get(123)).unwrap();
            writer
                .write_event(&Event::insert(456).with_weight(100))
                .unwrap();
            writer.flush().unwrap();
        }
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"key\":123"));
        assert!(output.contains("\"key\":456"));
        assert!(output.contains("\"weight\":100"));
    }
}
