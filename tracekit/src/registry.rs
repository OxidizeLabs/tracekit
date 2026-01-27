//! Central registry for benchmark policies and workloads.
//!
//! This is the single source of truth for:
//! - Policy definitions (id, display name, constructor)
//! - Workload definitions (id, display name, spec)
//!
//! To add a new policy or workload, modify this file only.
//! All benchmarks and reports automatically pick up the changes.

use crate::workload::{Workload, WorkloadSpec};

// ============================================================================
// Policy Registry
// ============================================================================

// Note: The `for_each_policy!` macro for cachekit integration is provided by
// the `tracekit-cachekit` crate. This keeps the core `tracekit` crate independent
// of any specific cache implementation.

// ============================================================================
// Workload Registry
// ============================================================================

/// Workload case with metadata.
#[derive(Debug, Clone, Copy)]
pub struct WorkloadCase {
    /// Short identifier (e.g., "uniform", "zipfian_1.0").
    pub id: &'static str,
    /// Human-readable display name (e.g., "Uniform", "Zipfian 1.0").
    pub display_name: &'static str,
    /// Workload specification (without universe/seed).
    pub workload: Workload,
}

/// Standard workload suite - focused set that differentiates policies.
///
/// This is the primary benchmark set for policy comparison.
pub const STANDARD_WORKLOADS: &[WorkloadCase] = &[
    WorkloadCase {
        id: "uniform",
        display_name: "Uniform",
        workload: Workload::Uniform,
    },
    WorkloadCase {
        id: "hotset_90_10",
        display_name: "HotSet 90/10",
        workload: Workload::HotSet {
            hot_fraction: 0.1,
            hot_prob: 0.9,
        },
    },
    WorkloadCase {
        id: "scan",
        display_name: "Scan",
        workload: Workload::Scan,
    },
    WorkloadCase {
        id: "zipfian_1.0",
        display_name: "Zipfian 1.0",
        workload: Workload::Zipfian { exponent: 1.0 },
    },
    WorkloadCase {
        id: "scrambled_zipf",
        display_name: "Scrambled Zipfian",
        workload: Workload::ScrambledZipfian { exponent: 1.0 },
    },
    WorkloadCase {
        id: "latest",
        display_name: "Latest",
        workload: Workload::Latest { exponent: 0.8 },
    },
    WorkloadCase {
        id: "scan_resistance",
        display_name: "Scan Resistance",
        workload: Workload::ScanResistance {
            scan_fraction: 0.2,
            scan_length: 1000,
            point_exponent: 1.0,
        },
    },
    WorkloadCase {
        id: "flash_crowd",
        display_name: "Flash Crowd",
        workload: Workload::FlashCrowd {
            base_exponent: 1.0,
            flash_prob: 0.001,
            flash_duration: 1000,
            flash_keys: 10,
            flash_intensity: 100.0,
        },
    },
];

/// Extended workload suite - comprehensive set covering all workload types.
///
/// Use this for exhaustive testing or specialized reports.
pub const EXTENDED_WORKLOADS: &[WorkloadCase] = &[
    WorkloadCase {
        id: "uniform",
        display_name: "Uniform",
        workload: Workload::Uniform,
    },
    WorkloadCase {
        id: "hotset_90_10",
        display_name: "HotSet 90/10",
        workload: Workload::HotSet {
            hot_fraction: 0.1,
            hot_prob: 0.9,
        },
    },
    WorkloadCase {
        id: "scan",
        display_name: "Scan",
        workload: Workload::Scan,
    },
    WorkloadCase {
        id: "zipfian_1.0",
        display_name: "Zipfian 1.0",
        workload: Workload::Zipfian { exponent: 1.0 },
    },
    WorkloadCase {
        id: "zipfian_0.8",
        display_name: "Zipfian 0.8",
        workload: Workload::Zipfian { exponent: 0.8 },
    },
    WorkloadCase {
        id: "scrambled_zipf",
        display_name: "Scrambled Zipfian",
        workload: Workload::ScrambledZipfian { exponent: 1.0 },
    },
    WorkloadCase {
        id: "latest",
        display_name: "Latest",
        workload: Workload::Latest { exponent: 0.8 },
    },
    WorkloadCase {
        id: "shifting_hotspot",
        display_name: "Shifting Hotspot",
        workload: Workload::ShiftingHotspot {
            shift_interval: 10_000,
            hot_fraction: 0.1,
        },
    },
    WorkloadCase {
        id: "exponential",
        display_name: "Exponential",
        workload: Workload::Exponential { lambda: 0.05 },
    },
    WorkloadCase {
        id: "pareto",
        display_name: "Pareto",
        workload: Workload::Pareto { shape: 1.5 },
    },
    WorkloadCase {
        id: "scan_resistance",
        display_name: "Scan Resistance",
        workload: Workload::ScanResistance {
            scan_fraction: 0.2,
            scan_length: 1000,
            point_exponent: 1.0,
        },
    },
    WorkloadCase {
        id: "correlated",
        display_name: "Correlated",
        workload: Workload::Correlated {
            stride: 1,
            burst_len: 8,
            burst_prob: 0.3,
        },
    },
    WorkloadCase {
        id: "loop_small",
        display_name: "Loop (small)",
        workload: Workload::Loop {
            working_set_size: 512,
        },
    },
    WorkloadCase {
        id: "working_set_churn",
        display_name: "Working Set Churn",
        workload: Workload::WorkingSetChurn {
            working_set_size: 2048,
            churn_rate: 0.001,
        },
    },
    WorkloadCase {
        id: "bursty",
        display_name: "Bursty",
        workload: Workload::Bursty {
            hurst: 0.8,
            base_exponent: 1.0,
        },
    },
    WorkloadCase {
        id: "flash_crowd",
        display_name: "Flash Crowd",
        workload: Workload::FlashCrowd {
            base_exponent: 1.0,
            flash_prob: 0.001,
            flash_duration: 1000,
            flash_keys: 10,
            flash_intensity: 100.0,
        },
    },
    WorkloadCase {
        id: "mixture",
        display_name: "Mixture",
        workload: Workload::Mixture,
    },
];

/// Build a `WorkloadSpec` from a workload case and runtime parameters.
impl WorkloadCase {
    pub fn with_params(self, universe: u64, seed: u64) -> WorkloadSpec {
        WorkloadSpec {
            universe,
            workload: self.workload,
            seed,
        }
    }
}
