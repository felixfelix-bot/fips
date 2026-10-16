//! Time, supplied by the host.
//!
//! The core never reads a clock. A type that measures time takes the host's
//! clock as a type parameter bounded by [`Clock`] and takes the clock's
//! instants as method arguments; the host reads its clock and passes the
//! instant in. The host fixes the parameter once, in a type alias at the path
//! its callers already use, as it does for [`crate::entropy::EntropySource`].
//!
//! Durations are [`core::time::Duration`], which needs neither `std` nor an
//! allocator. The arithmetic between instants and durations is the three
//! methods on [`Instant`].

pub use core::time::Duration;

/// A point on a host's monotonic clock.
///
/// Opaque: only differences between two instants of one clock mean anything.
/// An implementation is a small `Copy` value, such as a tick count, and never
/// allocates.
pub trait Instant: Copy + Ord + core::fmt::Debug + Send + Sync + 'static {
    /// Time from `earlier` to `self`, or zero when `earlier` is the later one.
    fn saturating_duration_since(self, earlier: Self) -> Duration;

    /// `self + d`, or `None` past the end of the representation.
    fn checked_add(self, d: Duration) -> Option<Self>;

    /// `self - d`, or `None` before its start.
    fn checked_sub(self, d: Duration) -> Option<Self>;
}

/// A host's monotonic clock.
///
/// Successive reads never go backwards. The core names a clock only as a type
/// parameter and takes its instants as arguments; it never calls `now`, which
/// is for the host.
pub trait Clock {
    /// The instants this clock reads.
    type Instant: Instant;

    /// The current instant.
    fn now(&self) -> Self::Instant;
}

#[cfg(test)]
mod tests;
