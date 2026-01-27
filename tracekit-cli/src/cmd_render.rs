//! `render` command - Render benchmark results to documentation.
//!
//! This command converts benchmark JSON artifacts to Markdown documentation
//! suitable for GitHub Pages.

use clap::Args;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracekit::json_results::{BenchmarkArtifact, ResultRow};

/// HTML template for interactive charts
const CHARTS_TEMPLATE: &str = include_str!("charts_template.html");

#[derive(Args)]
pub struct RenderArgs {
    /// Input JSON results file
    #[arg()]
    input: PathBuf,

    /// Output directory for documentation
    #[arg(default_value = "docs/benchmarks/latest")]
    output_dir: PathBuf,
}

pub fn run(args: RenderArgs) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Reading benchmark results from: {}", args.input.display());

    // Read and parse JSON
    let json_content = fs::read_to_string(&args.input)?;
    let artifact: BenchmarkArtifact = serde_json::from_str(&json_content)?;

    // Create output directory
    fs::create_dir_all(&args.output_dir)?;

    // Generate Markdown
    let markdown = generate_markdown(&artifact);

    // Write index.md
    let index_path = args.output_dir.join("index.md");
    fs::write(&index_path, markdown)?;

    // Copy results.json
    let json_dest = args.output_dir.join("results.json");
    fs::copy(&args.input, &json_dest)?;

    // Generate charts.html
    let charts_path = args.output_dir.join("charts.html");
    fs::write(&charts_path, CHARTS_TEMPLATE)?;

    eprintln!("Generated documentation:");
    eprintln!("   - {}", index_path.display());
    eprintln!("   - {}", json_dest.display());
    eprintln!("   - {}", charts_path.display());

    Ok(())
}

fn generate_markdown(artifact: &BenchmarkArtifact) -> String {
    let mut md = String::new();

    // Header
    md.push_str("# Benchmark Results\n\n");

    // Quick links
    md.push_str(
        "**Quick Links**: [Interactive Charts](charts.html) | [Raw JSON](results.json)\n\n",
    );
    md.push_str("---\n\n");

    // Metadata
    md.push_str("## Environment\n\n");
    md.push_str(&format!("- **Date**: {}\n", artifact.metadata.timestamp));
    if let Some(ref commit) = artifact.metadata.git_commit {
        md.push_str(&format!("- **Commit**: `{}`\n", commit));
    }
    if let Some(ref branch) = artifact.metadata.git_branch {
        md.push_str(&format!("- **Branch**: `{}`\n", branch));
    }
    md.push_str(&format!("- **Dirty**: {}\n", artifact.metadata.git_dirty));
    md.push_str(&format!(
        "- **Rustc**: {}\n",
        artifact.metadata.rustc_version
    ));
    md.push_str(&format!("- **Host**: {}\n", artifact.metadata.host_triple));
    if let Some(ref cpu) = artifact.metadata.cpu_model {
        md.push_str(&format!("- **CPU**: {}\n", cpu));
    }
    md.push('\n');

    // Configuration
    md.push_str("## Configuration\n\n");
    let config = &artifact.metadata.config;
    md.push_str(&format!("- **Capacity**: {}\n", config.capacity));
    md.push_str(&format!("- **Universe**: {}\n", config.universe));
    md.push_str(&format!("- **Operations**: {}\n", config.operations));
    md.push_str(&format!("- **Seed**: {}\n", config.seed));
    md.push('\n');

    // Group results by case type
    let by_case = group_by_case(&artifact.results);

    // Hit Rate Table
    if let Some(hit_rate_results) = by_case.get("hit_rate") {
        md.push_str("## Hit Rate Comparison\n\n");
        md.push_str(&generate_hit_rate_table(hit_rate_results));
        md.push('\n');
    }

    // Throughput Table
    if let Some(comprehensive_results) = by_case.get("comprehensive") {
        md.push_str("## Throughput (Million ops/sec)\n\n");
        md.push_str(&generate_throughput_table(comprehensive_results));
        md.push('\n');

        md.push_str("## Latency P99 (nanoseconds)\n\n");
        md.push_str(&generate_latency_table(comprehensive_results));
        md.push('\n');
    }

    // Scan Resistance
    if let Some(scan_results) = by_case.get("scan_resistance") {
        md.push_str("## Scan Resistance\n\n");
        md.push_str(&generate_scan_resistance_table(scan_results));
        md.push('\n');
    }

    // Adaptation Speed
    if let Some(adaptation_results) = by_case.get("adaptation") {
        md.push_str("## Adaptation Speed\n\n");
        md.push_str(&generate_adaptation_table(adaptation_results));
        md.push('\n');
    }

    // Policy Selection Guide
    md.push_str("## Policy Selection Guide\n\n");
    md.push_str(&generate_policy_guide());
    md.push('\n');

    // Footer
    md.push_str("---\n\n");
    md.push_str(&format!(
        "*Generated from `results.json` (schema v{})*\n",
        artifact.schema_version
    ));

    md
}

fn group_by_case(results: &[ResultRow]) -> HashMap<String, Vec<&ResultRow>> {
    let mut grouped: HashMap<String, Vec<&ResultRow>> = HashMap::new();
    for result in results {
        grouped
            .entry(result.case_id.clone())
            .or_default()
            .push(result);
    }
    grouped
}

fn generate_hit_rate_table(results: &[&ResultRow]) -> String {
    let mut md = String::new();

    // Group by policy and workload
    let mut by_policy: HashMap<String, HashMap<String, f64>> = HashMap::new();
    let mut workloads = Vec::new();

    for result in results {
        if let Some(ref hit_stats) = result.metrics.hit_stats {
            by_policy
                .entry(result.policy_name.clone())
                .or_default()
                .insert(result.workload_name.clone(), hit_stats.hit_rate);

            if !workloads.contains(&result.workload_name) {
                workloads.push(result.workload_name.clone());
            }
        }
    }

    // Sort policies and workloads
    let mut policies: Vec<_> = by_policy.keys().cloned().collect();
    policies.sort();
    workloads.sort();

    // Table header
    md.push_str("| Policy |");
    for workload in &workloads {
        md.push_str(&format!(" {} |", workload));
    }
    md.push('\n');

    // Separator
    md.push_str("|--------|");
    for _ in &workloads {
        md.push_str("-------:|");
    }
    md.push('\n');

    // Table rows
    for policy in &policies {
        md.push_str(&format!("| **{}** |", policy));
        if let Some(workload_results) = by_policy.get(policy) {
            for workload in &workloads {
                if let Some(&hit_rate) = workload_results.get(workload) {
                    md.push_str(&format!(" {:.2}% |", hit_rate * 100.0));
                } else {
                    md.push_str(" - |");
                }
            }
        }
        md.push('\n');
    }

    md
}

fn generate_throughput_table(results: &[&ResultRow]) -> String {
    let mut md = String::new();

    let mut by_policy: HashMap<String, HashMap<String, f64>> = HashMap::new();
    let mut workloads = Vec::new();

    for result in results {
        if let Some(ref throughput) = result.metrics.throughput {
            by_policy
                .entry(result.policy_name.clone())
                .or_default()
                .insert(
                    result.workload_name.clone(),
                    throughput.ops_per_sec / 1_000_000.0,
                );

            if !workloads.contains(&result.workload_name) {
                workloads.push(result.workload_name.clone());
            }
        }
    }

    let mut policies: Vec<_> = by_policy.keys().cloned().collect();
    policies.sort();
    workloads.sort();

    md.push_str("| Policy |");
    for workload in &workloads {
        md.push_str(&format!(" {} |", workload));
    }
    md.push('\n');

    md.push_str("|--------|");
    for _ in &workloads {
        md.push_str("-------:|");
    }
    md.push('\n');

    for policy in &policies {
        md.push_str(&format!("| **{}** |", policy));
        if let Some(workload_results) = by_policy.get(policy) {
            for workload in &workloads {
                if let Some(&mops) = workload_results.get(workload) {
                    md.push_str(&format!(" {:.2} |", mops));
                } else {
                    md.push_str(" - |");
                }
            }
        }
        md.push('\n');
    }

    md
}

fn generate_latency_table(results: &[&ResultRow]) -> String {
    let mut md = String::new();

    let mut by_policy: HashMap<String, HashMap<String, u64>> = HashMap::new();
    let mut workloads = Vec::new();

    for result in results {
        if let Some(ref latency) = result.metrics.latency {
            by_policy
                .entry(result.policy_name.clone())
                .or_default()
                .insert(result.workload_name.clone(), latency.p99_ns);

            if !workloads.contains(&result.workload_name) {
                workloads.push(result.workload_name.clone());
            }
        }
    }

    let mut policies: Vec<_> = by_policy.keys().cloned().collect();
    policies.sort();
    workloads.sort();

    md.push_str("| Policy |");
    for workload in &workloads {
        md.push_str(&format!(" {} |", workload));
    }
    md.push('\n');

    md.push_str("|--------|");
    for _ in &workloads {
        md.push_str("-------:|");
    }
    md.push('\n');

    for policy in &policies {
        md.push_str(&format!("| **{}** |", policy));
        if let Some(workload_results) = by_policy.get(policy) {
            for workload in &workloads {
                if let Some(&p99_ns) = workload_results.get(workload) {
                    md.push_str(&format!(" {} |", p99_ns));
                } else {
                    md.push_str(" - |");
                }
            }
        }
        md.push('\n');
    }

    md
}

fn generate_scan_resistance_table(results: &[&ResultRow]) -> String {
    let mut md = String::new();

    md.push_str("| Policy | Baseline | During Scan | Recovery | Score |\n");
    md.push_str("|--------|----------|-------------|----------|-------|\n");

    let mut policies: Vec<_> = results.iter().map(|r| (r.policy_name.clone(), r)).collect();
    policies.sort_by(|a, b| a.0.cmp(&b.0));

    for (policy_name, result) in policies {
        if let Some(ref scan_stats) = result.metrics.scan_resistance {
            md.push_str(&format!(
                "| **{}** | {:.2}% | {:.2}% | {:.2}% | {:.3} |\n",
                policy_name,
                scan_stats.baseline_hit_rate * 100.0,
                scan_stats.scan_hit_rate * 100.0,
                scan_stats.recovery_hit_rate * 100.0,
                scan_stats.resistance_score
            ));
        }
    }

    md.push_str("\n*Score = Recovery/Baseline (1.0 = perfect recovery)*\n");

    md
}

fn generate_adaptation_table(results: &[&ResultRow]) -> String {
    let mut md = String::new();

    md.push_str("| Policy | Stable Hit Rate | Ops to 50% | Ops to 80% |\n");
    md.push_str("|--------|-----------------|------------|------------|\n");

    let mut policies: Vec<_> = results.iter().map(|r| (r.policy_name.clone(), r)).collect();
    policies.sort_by(|a, b| a.0.cmp(&b.0));

    for (policy_name, result) in policies {
        if let Some(ref adaptation_stats) = result.metrics.adaptation {
            md.push_str(&format!(
                "| **{}** | {:.2}% | {} | {} |\n",
                policy_name,
                adaptation_stats.stable_hit_rate * 100.0,
                adaptation_stats.ops_to_50_percent,
                adaptation_stats.ops_to_80_percent
            ));
        }
    }

    md.push_str("\n*Lower ops-to-X% is better (faster adaptation)*\n");

    md
}

fn generate_policy_guide() -> String {
    let mut md = String::new();

    md.push_str("| Use Case | Recommended Policy | Why |\n");
    md.push_str("|----------|-------------------|-----|\n");
    md.push_str("| **General purpose, skewed workloads** | LRU, LFU, S3-FIFO | Best hit rates on Zipfian/skewed patterns |\n");
    md.push_str(
        "| **Scan-heavy workloads** | S3-FIFO, Heap-LFU | Scan-resistant, protect hot entries |\n",
    );
    md.push_str("| **Low latency required** | LRU, Clock | Fastest operations, O(1) overhead |\n");
    md.push_str("| **Memory constrained** | LRU, Clock | Minimal metadata overhead |\n");
    md.push_str("| **Frequency-aware** | LFU, Heap-LFU, LRU-K | Track access frequency for better decisions |\n");
    md.push_str("| **Shifting patterns** | S3-FIFO, 2Q | Adapt to changing access patterns |\n");
    md.push_str(
        "| **Multi-access patterns** | 2Q, S3-FIFO | Handle mixed one-hit and frequent items |\n",
    );

    md
}
