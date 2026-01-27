//! Cache trace simulation toolkit.
//!
//! ## Architecture
//! - [`Event`], [`Op`]: cache access events with optional weight/timestamp
//! - [`EventSource`]: trait for trace streams or generators
//! - [`CacheModel`]: minimal cache interface for simulation
//! - [`simulate`]: core simulation loop
//! - [`workload`]: 16+ synthetic workload generators
//! - [`metrics`]: benchmark metrics collection
//! - [`registry`]: policy/workload registries
//! - [`json_results`]: JSON serialization for results
//!
//! ## Example
//! ```ignore
//! use tracekit::{simulate, CacheModel, WorkloadSpec, Workload, BoundedGenerator};
//!
//! // Create a workload generator
//! let spec = WorkloadSpec {
//!     universe: 10_000,
//!     workload: Workload::Zipfian { exponent: 1.0 },
//!     seed: 42,
//! };
//! let mut source = BoundedGenerator::new(spec.generator(), 100_000);
//!
//! // Simulate with your cache implementation
//! let mut cache = MyCache::new(1000);
//! let stats = simulate(&mut cache, &mut source);
//! println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
//! ```

pub mod event;
pub mod json_results;
pub mod metrics;
pub mod model;
pub mod registry;
pub mod simulator;
pub mod source;
pub mod workload;

// Re-exports for convenience
pub use event::{Event, Op};
pub use model::CacheModel;
pub use simulator::{simulate, simulate_explicit};
pub use source::EventSource;
pub use workload::{BoundedGenerator, Workload, WorkloadGenerator, WorkloadSpec};

// Note: for_each_policy macro is automatically exported at crate root via #[macro_export]
