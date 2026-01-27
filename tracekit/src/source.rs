//! Event source trait for trace streams and generators.
//!
//! ## Architecture
//! `EventSource` is the primary abstraction for consuming cache events, whether
//! from file-based traces or synthetic generators.
//!
//! ## Key Components
//! - [`EventSource`]: Trait for streaming cache events

use crate::event::Event;

/// Stream of cache events from a trace or generator.
///
/// This trait provides a pull-based interface for consuming events. It's more
/// flexible than `Iterator<Item=Event>` for stateful sources like network
/// streams or memory-mapped files.
pub trait EventSource {
    /// Returns the next event, or `None` at end-of-trace.
    fn next_event(&mut self) -> Option<Event>;

    /// Hint for total event count (for progress bars).
    ///
    /// Returns `None` if the count is unknown (e.g., infinite generators).
    fn size_hint(&self) -> Option<usize> {
        None
    }
}

/// Blanket implementation for iterators of events.
impl<I> EventSource for I
where
    I: Iterator<Item = Event>,
{
    fn next_event(&mut self) -> Option<Event> {
        self.next()
    }
}
