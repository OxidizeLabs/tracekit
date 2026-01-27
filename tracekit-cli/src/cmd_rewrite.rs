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

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum Format {
    KeyOnly,
    Jsonl,
}

pub fn run(args: RewriteArgs) -> Result<(), Box<dyn std::error::Error>> {
    let input_file = File::open(&args.input)?;
    let output_file = File::create(&args.output)?;
    let reader = BufReader::new(input_file);
    let writer = BufWriter::new(output_file);

    let mut count = 0u64;

    match (args.input_format, args.output_format) {
        (Format::KeyOnly, Format::KeyOnly) => {
            let mut source = KeyOnlyReader::new(reader);
            let mut out = tracekit_formats::KeyOnlyWriter::new(writer);
            while let Some(event) = source.next_event() {
                out.write_key(event.key)?;
                count += 1;
            }
            out.flush()?;
        }
        (Format::KeyOnly, Format::Jsonl) => {
            let mut source = KeyOnlyReader::new(reader);
            let mut out = tracekit_formats::JsonlWriter::new(writer);
            while let Some(event) = source.next_event() {
                out.write_event(&event)?;
                count += 1;
            }
            out.flush()?;
        }
        (Format::Jsonl, Format::KeyOnly) => {
            let mut source = tracekit_formats::JsonlReader::new(reader);
            let mut out = tracekit_formats::KeyOnlyWriter::new(writer);
            while let Some(event) = source.next_event() {
                out.write_key(event.key)?;
                count += 1;
            }
            out.flush()?;
        }
        (Format::Jsonl, Format::Jsonl) => {
            let mut source = tracekit_formats::JsonlReader::new(reader);
            let mut out = tracekit_formats::JsonlWriter::new(writer);
            while let Some(event) = source.next_event() {
                out.write_event(&event)?;
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
