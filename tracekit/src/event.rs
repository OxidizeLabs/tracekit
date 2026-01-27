//! Cache access event types.
//!
//! ## Architecture
//! Events represent individual cache operations in a trace. Each event has a key
//! and an operation type, with optional weight and timestamp for future extensions.
//!
//! ## Key Components
//! - [`Event`]: A single cache access event
//! - [`Op`]: Operation type (Get, Insert, Delete)

/// A cache access event in a trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Event {
    /// Cache key (application-defined).
    pub key: u64,
    /// Operation type.
    pub op: Op,
    /// Optional weight for size-aware policies (v0.2+).
    pub weight: Option<u32>,
    /// Optional timestamp for TTL/time-aware policies (v0.2+).
    pub ts: Option<u64>,
}

impl Event {
    /// Create a Get event (most common case).
    #[inline]
    pub const fn get(key: u64) -> Self {
        Self {
            key,
            op: Op::Get,
            weight: None,
            ts: None,
        }
    }

    /// Create an Insert event.
    #[inline]
    pub const fn insert(key: u64) -> Self {
        Self {
            key,
            op: Op::Insert,
            weight: None,
            ts: None,
        }
    }

    /// Create a Delete event.
    #[inline]
    pub const fn delete(key: u64) -> Self {
        Self {
            key,
            op: Op::Delete,
            weight: None,
            ts: None,
        }
    }

    /// Set the weight for this event.
    #[inline]
    pub const fn with_weight(mut self, weight: u32) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Set the timestamp for this event.
    #[inline]
    pub const fn with_ts(mut self, ts: u64) -> Self {
        self.ts = Some(ts);
        self
    }
}

/// Cache operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Op {
    /// Cache lookup.
    #[default]
    Get,
    /// Insert or update a key.
    Insert,
    /// Remove a key.
    Delete,
}
