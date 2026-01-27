//! Trace file format parsers and writers for tracekit.
//!
//! ## Architecture
//! This crate provides `EventSource` implementations for reading cache traces
//! from various file formats, as well as writers for generating trace files.
//!
//! ## Key Components
//! - [`KeyOnlyReader`]: Reads traces with one key per line
//! - [`KeyOnlyWriter`]: Writes traces with one key per line
//!
//! ## Features
//! - `jsonl`: Enable JSONL format support
//! - `compression`: Enable gzip compression support
//! - `full`: Enable all features

mod key_only;

#[cfg(feature = "jsonl")]
mod jsonl;

pub use key_only::{KeyOnlyReader, KeyOnlyWriter};

#[cfg(feature = "jsonl")]
pub use jsonl::{JsonlReader, JsonlWriter};
