//! Caching Entities
//!
//! The coordinate cache lives in `fips-core`; this module re-exports it, with
//! its hasher keyed from [`OsEntropy`](crate::identity::OsEntropy).

pub use fips_core::cache::{
    ADDRS_PER_ENTRY, CacheEntry, CacheError, CacheStats, CoordSource, DEFAULT_COORD_CACHE_SIZE,
    DEFAULT_COORD_CACHE_TTL_MS, Funding, HintOutcome, PROBATION_SHARE_DIVISOR, Placed, Segment,
    VERIFIED_TTL_MS,
};

/// The coordinate cache, its hasher keyed from [`OsEntropy`](crate::identity::OsEntropy).
pub type CoordCache = fips_core::cache::CoordCache<crate::identity::OsEntropy>;
