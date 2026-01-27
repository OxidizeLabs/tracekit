//! `tracegen` command - Generate synthetic traces.

use clap::Args;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use tracekit::{BoundedGenerator, Workload, WorkloadSpec};

#[derive(Args)]
pub struct TracegenArgs {
    /// Workload type
    #[arg(short, long, value_enum, default_value = "zipfian")]
    workload: WorkloadType,

    /// Key universe size
    #[arg(short, long, default_value = "10000")]
    universe: u64,

    /// Number of events to generate
    #[arg(short, long, default_value = "100000")]
    count: usize,

    /// Random seed for reproducibility
    #[arg(short, long, default_value = "42")]
    seed: u64,

    /// Zipfian exponent (for zipfian/scrambled workloads)
    #[arg(long, default_value = "1.0")]
    exponent: f64,

    /// Hot fraction (for hotset workload)
    #[arg(long, default_value = "0.1")]
    hot_fraction: f64,

    /// Hot probability (for hotset workload)
    #[arg(long, default_value = "0.9")]
    hot_prob: f64,

    /// Output file (stdout if not specified)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "key-only")]
    format: OutputFormat,
}

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum WorkloadType {
    Uniform,
    Zipfian,
    Scrambled,
    Hotset,
    Scan,
    Latest,
}

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    KeyOnly,
    Jsonl,
}

pub fn run(args: TracegenArgs) -> Result<(), Box<dyn std::error::Error>> {
    let workload = match args.workload {
        WorkloadType::Uniform => Workload::Uniform,
        WorkloadType::Zipfian => Workload::Zipfian {
            exponent: args.exponent,
        },
        WorkloadType::Scrambled => Workload::ScrambledZipfian {
            exponent: args.exponent,
        },
        WorkloadType::Hotset => Workload::HotSet {
            hot_fraction: args.hot_fraction,
            hot_prob: args.hot_prob,
        },
        WorkloadType::Scan => Workload::Scan,
        WorkloadType::Latest => Workload::Latest {
            exponent: args.exponent,
        },
    };

    let spec = WorkloadSpec {
        universe: args.universe,
        workload,
        seed: args.seed,
    };

    let mut source = BoundedGenerator::new(spec.generator(), args.count);

    // Create output writer
    let writer: Box<dyn Write> = match &args.output {
        Some(path) => Box::new(BufWriter::new(File::create(path)?)),
        None => Box::new(BufWriter::new(std::io::stdout())),
    };

    match args.format {
        OutputFormat::KeyOnly => {
            let mut writer = tracekit_formats::KeyOnlyWriter::new(writer);
            use tracekit::EventSource;
            while let Some(event) = source.next_event() {
                writer.write_key(event.key)?;
            }
            writer.flush()?;
        }
        OutputFormat::Jsonl => {
            let mut writer = tracekit_formats::JsonlWriter::new(writer);
            use tracekit::EventSource;
            while let Some(event) = source.next_event() {
                writer.write_event(&event)?;
            }
            writer.flush()?;
        }
    }

    if let Some(path) = &args.output {
        eprintln!("Generated {} events to {}", args.count, path.display());
    }

    Ok(())
}
