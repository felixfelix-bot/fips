//! Session Index Allocator
//!
//! The session index, its allocator and its error live in `fips-core`; this
//! module re-exports them, with the allocator drawing from
//! [`OsEntropy`](crate::identity::OsEntropy).

pub use fips_core::utils::index::{IndexError, SessionIndex};

/// The session index allocator, drawing its indices from
/// [`OsEntropy`](crate::identity::OsEntropy).
pub type IndexAllocator = fips_core::utils::index::IndexAllocator<crate::identity::OsEntropy>;
