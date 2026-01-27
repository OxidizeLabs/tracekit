//! Adapter connecting tracekit simulation to cachekit cache implementations.
//!
//! ## Architecture
//! This crate provides the `CachekitAdapter` wrapper that implements tracekit's
//! `CacheModel` trait for any cachekit cache policy.
//!
//! ## Example
//! ```ignore
//! use tracekit::{simulate, BoundedGenerator, WorkloadSpec, Workload};
//! use tracekit_cachekit::CachekitAdapter;
//! use cachekit::policy::lru::LruCore;
//!
//! let spec = WorkloadSpec {
//!     universe: 10_000,
//!     workload: Workload::Zipfian { exponent: 1.0 },
//!     seed: 42,
//! };
//! let mut source = BoundedGenerator::new(spec.generator(), 100_000);
//!
//! let cache = LruCore::<u64, ()>::new(1000);
//! let mut adapter = CachekitAdapter::new(cache);
//! let stats = simulate(&mut adapter, &mut source);
//! println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
//! ```

use tracekit::CacheModel;

/// Adapter wrapping a cachekit cache to implement `CacheModel`.
///
/// This allows any cachekit cache policy to be used with tracekit's
/// simulation infrastructure.
#[derive(Debug, Clone)]
pub struct CachekitAdapter<C> {
    cache: C,
}

impl<C> CachekitAdapter<C> {
    /// Create a new adapter wrapping the given cache.
    pub fn new(cache: C) -> Self {
        Self { cache }
    }

    /// Returns a reference to the underlying cache.
    pub fn inner(&self) -> &C {
        &self.cache
    }

    /// Returns a mutable reference to the underlying cache.
    pub fn inner_mut(&mut self) -> &mut C {
        &mut self.cache
    }

    /// Consumes the adapter and returns the underlying cache.
    pub fn into_inner(self) -> C {
        self.cache
    }
}

// ============================================================================
// CacheModel implementations for cachekit caches
// ============================================================================
//
// When cachekit is available as a dependency, uncomment and implement:
//
// ```rust
// use cachekit::traits::Cache;
//
// impl<K, V, C> CacheModel for CachekitAdapter<C>
// where
//     K: From<u64> + Eq + std::hash::Hash,
//     V: Default,
//     C: Cache<K, V>,
// {
//     fn get(&mut self, key: u64) -> bool {
//         self.cache.get(&K::from(key)).is_some()
//     }
//
//     fn insert(&mut self, key: u64) {
//         self.cache.insert(K::from(key), V::default());
//     }
//
//     fn delete(&mut self, key: u64) {
//         self.cache.remove(&K::from(key));
//     }
// }
// ```

/// Placeholder implementation for testing without cachekit.
///
/// This implements CacheModel for any type that has get/insert/remove methods
/// with the expected signatures. Remove this when cachekit is available.
impl<C> CacheModel for CachekitAdapter<C>
where
    C: SimpleCacheLike,
{
    fn get(&mut self, key: u64) -> bool {
        self.cache.get_key(key)
    }

    fn insert(&mut self, key: u64) {
        self.cache.insert_key(key);
    }

    fn delete(&mut self, key: u64) {
        self.cache.delete_key(key);
    }
}

/// Trait for simple cache-like types (for testing without cachekit).
///
/// This is a temporary trait until cachekit is available as a dependency.
pub trait SimpleCacheLike {
    fn get_key(&mut self, key: u64) -> bool;
    fn insert_key(&mut self, key: u64);
    fn delete_key(&mut self, key: u64);
}

// ============================================================================
// Policy iteration macro (moved from tracekit core)
// ============================================================================
//
// This macro requires cachekit as a dependency. Uncomment when available:
//
// /// Macro to execute monomorphic code for each cachekit policy.
// ///
// /// This avoids dynamic dispatch in benchmark hot paths while keeping
// /// policy iteration centralized.
// ///
// /// # Usage
// ///
// /// ```ignore
// /// for_each_policy! {
// ///     with |policy_id, display_name, make_cache| {
// ///         let mut cache = make_cache(CAPACITY);
// ///         // ... benchmark code ...
// ///     }
// /// }
// /// ```
// #[macro_export]
// macro_rules! for_each_policy {
//     (with |$policy_id:ident, $display_name:ident, $make_cache:ident| $body:block) => {{
//         use cachekit::policy::clock::ClockCache;
//         use cachekit::policy::heap_lfu::HeapLfuCache;
//         use cachekit::policy::lfu::LfuCache;
//         use cachekit::policy::lru::LruCore;
//         use cachekit::policy::lru_k::LrukCache;
//         use cachekit::policy::s3_fifo::S3FifoCache;
//         use cachekit::policy::two_q::TwoQCore;
//         use std::sync::Arc;
//
//         {
//             let $policy_id = "lru";
//             let $display_name = "LRU";
//             let $make_cache = |cap: usize| LruCore::<u64, u64>::new(cap);
//             $body
//         }
//         // ... more policies ...
//     }};
// }
