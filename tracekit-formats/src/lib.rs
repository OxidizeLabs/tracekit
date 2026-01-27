//! Trace file format parsers and writers for tracekit.
//!
//! ## Architecture
//! This crate provides `EventSource` implementations for reading cache traces
//! from various file formats, as well as writers for generating trace files.
//!
//! ## Supported Formats
//!
//! ### Simple Text Formats
//! - [`KeyOnlyReader`]/[`KeyOnlyWriter`]: One key per line (simplest format)
//! - [`LirsReader`]: LIRS trace format (one block number per line)
//!
//! ### Structured Text Formats
//! - [`JsonlReader`]/[`JsonlWriter`]: JSON Lines format (feature: `jsonl`)
//! - [`CsvReader`]: Configurable CSV format
//! - [`ArcReader`]: ARC trace format (space-separated: timestamp key size)
//! - [`CachelibReader`]: Cachelib CSV format (feature: `cachelib`)
//!
//! ## Features
//! - `jsonl`: Enable JSONL format support
//! - `cachelib`: Enable Cachelib format support
//! - `compression`: Enable gzip compression support (future)
//! - `full`: Enable all features

// Simple text formats
mod key_only;
mod lirs;

// Structured text formats
mod arc;
mod csv;

#[cfg(feature = "jsonl")]
mod jsonl;

#[cfg(feature = "cachelib")]
mod cachelib;

// Public exports
pub use arc::ArcReader;
pub use csv::{CsvConfig, CsvReader};
pub use key_only::{KeyOnlyReader, KeyOnlyWriter};
pub use lirs::LirsReader;

#[cfg(feature = "jsonl")]
pub use jsonl::{JsonlReader, JsonlWriter};

#[cfg(feature = "cachelib")]
pub use cachelib::{CachelibConfig, CachelibReader};
