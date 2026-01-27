//! Cache model trait for hit-rate simulation.
//!
//! ## Architecture
//! `CacheModel` provides the minimal interface needed to simulate cache behavior.
//! It abstracts over different cache implementations (LRU, LFU, S3-FIFO, etc.)
//! to enable policy-agnostic trace replay.
//!
//! ## Key Components
//! - [`CacheModel`]: Minimal cache interface for simulation

/// Minimal cache model for hit-rate simulation.
///
/// Implementations should track their own capacity and eviction policy.
/// The simulator only needs to know if a key is present and how to insert.
pub trait CacheModel {
    /// Attempt a cache lookup. Returns `true` on hit, `false` on miss.
    fn get(&mut self, key: u64) -> bool;

    /// Insert or update a key with unit weight.
    fn insert(&mut self, key: u64);

    /// Remove a key from the cache.
    ///
    /// Default implementation is a no-op for caches that don't support deletion.
    fn delete(&mut self, _key: u64) {}
}
