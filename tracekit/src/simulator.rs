//! Cache simulation engine.
//!
//! ## Architecture
//! The simulator drives trace replay through a cache model, collecting hit/miss
//! statistics. It's the core loop that connects event sources to cache models.
//!
//! ## Core Operations
//! - Get: Check cache, record hit/miss, insert on miss
//! - Insert: Direct insertion
//! - Delete: Remove from cache

use crate::event::Op;
use crate::metrics::HitStats;
use crate::model::CacheModel;
use crate::source::EventSource;

/// Run a trace simulation, returning hit statistics.
///
/// On a cache miss during a Get operation, the key is automatically inserted.
/// This models the common "read-through" cache pattern.
pub fn simulate<C, S>(cache: &mut C, source: &mut S) -> HitStats
where
    C: CacheModel,
    S: EventSource,
{
    let mut stats = HitStats::default();

    while let Some(event) = source.next_event() {
        match event.op {
            Op::Get => {
                if cache.get(event.key) {
                    stats.hits += 1;
                } else {
                    stats.misses += 1;
                    cache.insert(event.key);
                    stats.inserts += 1;
                }
            }
            Op::Insert => {
                cache.insert(event.key);
                stats.inserts += 1;
            }
            Op::Delete => {
                cache.delete(event.key);
            }
        }
    }

    stats
}

/// Run a simulation without auto-insert on miss.
///
/// Use this when the trace explicitly contains Insert events and you don't
/// want automatic insertion on cache misses.
pub fn simulate_explicit<C, S>(cache: &mut C, source: &mut S) -> HitStats
where
    C: CacheModel,
    S: EventSource,
{
    let mut stats = HitStats::default();

    while let Some(event) = source.next_event() {
        match event.op {
            Op::Get => {
                if cache.get(event.key) {
                    stats.hits += 1;
                } else {
                    stats.misses += 1;
                }
            }
            Op::Insert => {
                cache.insert(event.key);
                stats.inserts += 1;
            }
            Op::Delete => {
                cache.delete(event.key);
            }
        }
    }

    stats
}
