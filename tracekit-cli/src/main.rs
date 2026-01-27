//! tracekit CLI - Command-line tools for trace simulation.
//!
//! ## Commands
//! - `tracegen`: Generate synthetic traces from workload specifications
//! - `simulate`: Run cache simulation on a trace file
//! - `rewrite`: Convert between trace formats
//! - `render`: Render benchmark results to documentation

use clap::{Parser, Subcommand};

mod cmd_render;
mod cmd_rewrite;
mod cmd_simulate;
mod cmd_tracegen;

#[derive(Parser)]
#[command(name = "tracekit")]
#[command(author, version, about = "Cache trace simulation toolkit", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate synthetic traces from workload specifications
    Tracegen(cmd_tracegen::TracegenArgs),
    /// Run cache simulation on a trace file (placeholder)
    Simulate(cmd_simulate::SimulateArgs),
    /// Convert between trace formats
    Rewrite(cmd_rewrite::RewriteArgs),
    /// Render benchmark results to documentation
    Render(cmd_render::RenderArgs),
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Tracegen(args) => cmd_tracegen::run(args),
        Commands::Simulate(args) => cmd_simulate::run(args),
        Commands::Rewrite(args) => cmd_rewrite::run(args),
        Commands::Render(args) => cmd_render::run(args),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
