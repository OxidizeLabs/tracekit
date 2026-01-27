//! `rewrite` command - Convert between trace formats.

use clap::Args;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use tracekit::EventSource;
use tracekit_formats::KeyOnlyReader;

#[derive(Args)]
pub struct RewriteArgs {
    /// Input trace file
    #[arg(short, long)]
    input: PathBuf,

    /// Output trace file
    #[arg(short, long)]
    output: PathBuf,

    /// Input format
    #[arg(long, value_enum, default_value = "key-only")]
    input_format: Format,

    /// Output format
    #[arg(long, value_enum, default_value = "key-only")]
    output_format: Format,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Format {
    /// Simple format: one key per line
    KeyOnly,
    /// JSON Lines format
    Jsonl,
    /// ARC trace format (space-separated: timestamp key size)
    Arc,
    /// LIRS trace format (one block number per line)
    Lirs,
    /// CSV format
    Csv,
    /// Cachelib CSV format
    Cachelib,
}

pub fn run(args: RewriteArgs) -> Result<(), Box<dyn std::error::Error>> {
    let input_file = File::open(&args.input)?;
    let output_file = File::create(&args.output)?;
    let reader = BufReader::new(input_file);
    let writer = BufWriter::new(output_file);

    let mut count = 0u64;

    // Read events from input format (using Box<dyn EventSource> for flexibility)
    let mut source: Box<dyn EventSource> = match args.input_format {
        Format::KeyOnly => Box::new(KeyOnlyReader::new(reader)),
        Format::Jsonl => Box::new(tracekit_formats::JsonlReader::new(reader)),
        Format::Arc => Box::new(tracekit_formats::ArcReader::new(reader)),
        Format::Lirs => Box::new(tracekit_formats::LirsReader::new(reader)),
        Format::Csv => {
            use tracekit_formats::{CsvConfig, CsvReader};
            Box::new(CsvReader::new(reader, CsvConfig::key_only()))
        }
        Format::Cachelib => Box::new(tracekit_formats::CachelibReader::with_defaults(reader)),
    };

    // Write events to output format
    match args.output_format {
        Format::KeyOnly => {
            let mut out = tracekit_formats::KeyOnlyWriter::new(writer);
            while let Some(event) = source.next_event() {
                out.write_key(event.key)?;
                count += 1;
            }
            out.flush()?;
        }
        Format::Jsonl => {
            let mut out = tracekit_formats::JsonlWriter::new(writer);
            while let Some(event) = source.next_event() {
                out.write_event(&event)?;
                count += 1;
            }
            out.flush()?;
        }
        Format::Arc | Format::Lirs | Format::Csv | Format::Cachelib => {
            eprintln!(
                "Warning: Output format {:?} uses the same representation as key-only.",
                args.output_format
            );
            let mut out = tracekit_formats::KeyOnlyWriter::new(writer);
            while let Some(event) = source.next_event() {
                out.write_key(event.key)?;
                count += 1;
            }
            out.flush()?;
        }
    }

    eprintln!(
        "Converted {} events: {} -> {}",
        count,
        args.input.display(),
        args.output.display()
    );

    Ok(())
}
