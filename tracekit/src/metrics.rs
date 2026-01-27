//! Standard benchmark metrics for cache policy evaluation.
//!
//! Provides consistent measurement across all cache policies for:
//! - Hit/miss rates and throughput
//! - Latency distribution (p50, p95, p99, max)
//! - Memory efficiency
//! - Eviction behavior
//! - Adaptation speed

use std::time::Duration;

use crate::workload::WorkloadSpec;

// ============================================================================
// Core Metrics Structures
// ============================================================================

/// Complete benchmark results for a cache policy.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Name of the policy being tested.
    pub policy_name: String,
    /// Name of the workload used.
    pub workload_name: String,
    /// Cache capacity.
    pub capacity: usize,
    /// Key universe size.
    pub universe: u64,
    /// Total operations performed.
    pub operations: u64,
    /// Hit/miss statistics.
    pub hit_stats: HitStats,
    /// Throughput measurements.
    pub throughput: ThroughputStats,
    /// Latency distribution.
    pub latency: LatencyStats,
    /// Eviction statistics.
    pub eviction: EvictionStats,
}

impl BenchmarkResult {
    /// Format as a single-line summary.
    pub fn summary(&self) -> String {
        format!(
            "{}/{}: hit={:.2}% throughput={:.2}Mops/s p99={:.1}ns evictions={}",
            self.policy_name,
            self.workload_name,
            self.hit_stats.hit_rate() * 100.0,
            self.throughput.ops_per_sec / 1_000_000.0,
            self.latency.p99.as_nanos(),
            self.eviction.total_evictions,
        )
    }
}

/// Hit/miss statistics.
#[derive(Debug, Clone, Copy, Default)]
pub struct HitStats {
    pub hits: u64,
    pub misses: u64,
    pub inserts: u64,
    pub updates: u64,
}

impl HitStats {
    #[inline]
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    #[inline]
    pub fn miss_rate(&self) -> f64 {
        1.0 - self.hit_rate()
    }

    pub fn total_ops(&self) -> u64 {
        self.hits + self.misses
    }
}

/// Throughput measurements.
#[derive(Debug, Clone, Copy, Default)]
pub struct ThroughputStats {
    /// Total wall-clock duration.
    pub total_duration: Duration,
    /// Operations per second.
    pub ops_per_sec: f64,
    /// Gets per second (hits + misses).
    pub gets_per_sec: f64,
    /// Inserts per second.
    pub inserts_per_sec: f64,
}

impl ThroughputStats {
    pub fn from_counts(hits: u64, misses: u64, inserts: u64, duration: Duration) -> Self {
        let secs = duration.as_secs_f64();
        if secs == 0.0 {
            return Self::default();
        }
        let total_ops = hits + misses + inserts;
        Self {
            total_duration: duration,
            ops_per_sec: total_ops as f64 / secs,
            gets_per_sec: (hits + misses) as f64 / secs,
            inserts_per_sec: inserts as f64 / secs,
        }
    }
}

/// Latency distribution (collected via sampling).
#[derive(Debug, Clone, Copy, Default)]
pub struct LatencyStats {
    pub min: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub max: Duration,
    pub mean: Duration,
    pub sample_count: usize,
}

impl LatencyStats {
    /// Compute percentiles from a sorted slice of durations.
    pub fn from_samples(samples: &mut [Duration]) -> Self {
        if samples.is_empty() {
            return Self::default();
        }

        samples.sort_unstable();
        let n = samples.len();
        let sum: Duration = samples.iter().sum();

        Self {
            min: samples[0],
            p50: samples[n / 2],
            p95: samples[(n * 95) / 100],
            p99: samples[(n * 99) / 100],
            max: samples[n - 1],
            mean: sum / n as u32,
            sample_count: n,
        }
    }
}

/// Eviction behavior metrics.
#[derive(Debug, Clone, Copy, Default)]
pub struct EvictionStats {
    /// Total evictions during the benchmark.
    pub total_evictions: u64,
    /// Evictions per insert (after warmup).
    pub evictions_per_insert: f64,
}

// ============================================================================
// Latency Sampler
// ============================================================================

/// Samples operation latencies without measuring every operation.
///
/// Uses reservoir sampling to collect a fixed number of latency samples
/// with minimal overhead.
#[derive(Debug)]
pub struct LatencySampler {
    samples: Vec<Duration>,
    capacity: usize,
    count: u64,
    sample_rate: u64,
}

impl LatencySampler {
    /// Create a sampler that collects up to `capacity` samples.
    /// `sample_rate` controls how often to sample (1 = every op, 100 = every 100th op).
    pub fn new(capacity: usize, sample_rate: u64) -> Self {
        Self {
            samples: Vec::with_capacity(capacity),
            capacity,
            count: 0,
            sample_rate: sample_rate.max(1),
        }
    }

    /// Record a latency sample (only if selected for sampling).
    #[inline]
    pub fn record(&mut self, duration: Duration) {
        self.count += 1;
        if self.count % self.sample_rate != 0 {
            return;
        }

        if self.samples.len() < self.capacity {
            self.samples.push(duration);
        } else {
            // Reservoir sampling for uniform distribution
            let idx = (self.count / self.sample_rate) as usize;
            if idx < self.capacity {
                self.samples[idx] = duration;
            } else {
                // Simple modulo replacement for speed
                let replace_idx = (self.count as usize) % self.capacity;
                self.samples[replace_idx] = duration;
            }
        }
    }

    /// Compute latency statistics from collected samples.
    pub fn stats(&mut self) -> LatencyStats {
        LatencyStats::from_samples(&mut self.samples)
    }
}

// ============================================================================
// Benchmark Runner
// ============================================================================

/// Configuration for running a benchmark.
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Name for this benchmark run.
    pub name: String,
    /// Cache capacity.
    pub capacity: usize,
    /// Number of operations to run.
    pub operations: usize,
    /// Warmup operations before measurement.
    pub warmup_ops: usize,
    /// Workload specification.
    pub workload: WorkloadSpec,
    /// Sample rate for latency collection (1 = all, 100 = 1%).
    pub latency_sample_rate: u64,
    /// Maximum latency samples to collect.
    pub max_latency_samples: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            capacity: 4096,
            operations: 100_000,
            warmup_ops: 10_000,
            workload: WorkloadSpec {
                universe: 16_384,
                workload: crate::workload::Workload::Zipfian { exponent: 1.0 },
                seed: 42,
            },
            latency_sample_rate: 100,
            max_latency_samples: 10_000,
        }
    }
}

// ============================================================================
// Specialized Benchmarks
// ============================================================================

/// Measure scan resistance by interleaving point lookups with sequential scans.
///
/// Returns (baseline_hit_rate, scan_hit_rate, recovery_hit_rate).
/// A scan-resistant policy should have recovery_hit_rate close to baseline_hit_rate.
/// Results from scan resistance measurement.
#[derive(Debug, Clone, Copy)]
pub struct ScanResistanceResult {
    /// Hit rate before the scan.
    pub baseline_hit_rate: f64,
    /// Hit rate during the scan.
    pub scan_hit_rate: f64,
    /// Hit rate after recovery.
    pub recovery_hit_rate: f64,
    /// Ratio of recovery to baseline (1.0 = perfect recovery).
    pub resistance_score: f64,
}

impl ScanResistanceResult {
    pub fn summary(&self) -> String {
        format!(
            "baseline={:.2}% scan={:.2}% recovery={:.2}% score={:.2}",
            self.baseline_hit_rate * 100.0,
            self.scan_hit_rate * 100.0,
            self.recovery_hit_rate * 100.0,
            self.resistance_score,
        )
    }
}

/// Results from adaptation speed measurement.
#[derive(Debug, Clone)]
pub struct AdaptationResult {
    /// Final stable hit rate after adaptation.
    pub stable_hit_rate: f64,
    /// Operations needed to reach 50% of stable hit rate.
    pub ops_to_50_percent: usize,
    /// Operations needed to reach 80% of stable hit rate.
    pub ops_to_80_percent: usize,
    /// Hit rate at each measurement window.
    pub hit_rate_curve: Vec<f64>,
}

impl AdaptationResult {
    pub fn summary(&self) -> String {
        format!(
            "stable={:.2}% ops_to_50%={} ops_to_80%={}",
            self.stable_hit_rate * 100.0,
            self.ops_to_50_percent,
            self.ops_to_80_percent,
        )
    }
}

// ============================================================================
// Comparison Utilities
// ============================================================================

/// Compare hit rates across multiple workloads.
#[derive(Debug, Clone)]
pub struct PolicyComparison {
    pub policy_name: String,
    pub results: Vec<BenchmarkResult>,
}

impl PolicyComparison {
    pub fn new(policy_name: &str) -> Self {
        Self {
            policy_name: policy_name.to_string(),
            results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: BenchmarkResult) {
        self.results.push(result);
    }

    /// Print a comparison table.
    pub fn print_table(&self) {
        println!("Policy: {}", self.policy_name);
        println!(
            "{:<20} {:>10} {:>12} {:>10} {:>10}",
            "Workload", "Hit Rate", "Ops/sec", "p99 (ns)", "Evictions"
        );
        println!("{}", "-".repeat(66));
        for r in &self.results {
            println!(
                "{:<20} {:>9.2}% {:>12.0} {:>10} {:>10}",
                r.workload_name,
                r.hit_stats.hit_rate() * 100.0,
                r.throughput.ops_per_sec,
                r.latency.p99.as_nanos(),
                r.eviction.total_evictions,
            );
        }
    }
}

/// Standard workload suite for comparing policies.
pub fn standard_workload_suite(universe: u64, seed: u64) -> Vec<(&'static str, WorkloadSpec)> {
    use crate::workload::Workload;

    vec![
        (
            "uniform",
            WorkloadSpec {
                universe,
                workload: Workload::Uniform,
                seed,
            },
        ),
        (
            "zipfian_1.0",
            WorkloadSpec {
                universe,
                workload: Workload::Zipfian { exponent: 1.0 },
                seed,
            },
        ),
        (
            "zipfian_0.8",
            WorkloadSpec {
                universe,
                workload: Workload::Zipfian { exponent: 0.8 },
                seed,
            },
        ),
        (
            "hotset_90_10",
            WorkloadSpec {
                universe,
                workload: Workload::HotSet {
                    hot_fraction: 0.1,
                    hot_prob: 0.9,
                },
                seed,
            },
        ),
        (
            "scan",
            WorkloadSpec {
                universe,
                workload: Workload::Scan,
                seed,
            },
        ),
        (
            "scan_resistance",
            WorkloadSpec {
                universe,
                workload: Workload::ScanResistance {
                    scan_fraction: 0.2,
                    scan_length: 1000,
                    point_exponent: 1.0,
                },
                seed,
            },
        ),
        (
            "loop_small",
            WorkloadSpec {
                universe,
                workload: Workload::Loop {
                    working_set_size: 512,
                },
                seed,
            },
        ),
        (
            "shifting_hotspot",
            WorkloadSpec {
                universe,
                workload: Workload::ShiftingHotspot {
                    shift_interval: 10_000,
                    hot_fraction: 0.1,
                },
                seed,
            },
        ),
        (
            "flash_crowd",
            WorkloadSpec {
                universe,
                workload: Workload::FlashCrowd {
                    base_exponent: 1.0,
                    flash_prob: 0.001,
                    flash_duration: 1000,
                    flash_keys: 10,
                    flash_intensity: 100.0,
                },
                seed,
            },
        ),
    ]
}

// ============================================================================
// Memory Measurement (basic)
// ============================================================================

/// Estimate memory overhead per entry (requires std::mem::size_of on cache).
pub fn estimate_entry_overhead<C>(cache: &C, entries: usize) -> MemoryEstimate
where
    C: Sized,
{
    let cache_size = std::mem::size_of_val(cache);
    MemoryEstimate {
        total_bytes: cache_size,
        bytes_per_entry: if entries > 0 { cache_size / entries } else { 0 },
        entry_count: entries,
    }
}

/// Memory usage estimate.
#[derive(Debug, Clone, Copy)]
pub struct MemoryEstimate {
    pub total_bytes: usize,
    pub bytes_per_entry: usize,
    pub entry_count: usize,
}

impl MemoryEstimate {
    pub fn summary(&self) -> String {
        format!(
            "total={}KB entries={} bytes/entry={}",
            self.total_bytes / 1024,
            self.entry_count,
            self.bytes_per_entry,
        )
    }
}
