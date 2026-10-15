//! The rate limiters live in `fips-core`; this module re-exports them, with
//! their clock fixed to [`HostClock`](crate::time::HostClock).

pub use fips_core::node::rate_limit::{Msg1Class, derive_established_bucket};

/// The token bucket, on the host clock.
pub(crate) type TokenBucket = fips_core::node::rate_limit::TokenBucket<crate::time::HostClock>;
/// The msg1 rate limiter, on the host clock.
pub(crate) type HandshakeRateLimiter =
    fips_core::node::rate_limit::HandshakeRateLimiter<crate::time::HostClock>;
/// The per-link session-setup rate limiter, on the host clock.
pub(crate) type SessionSetupRateLimiter =
    fips_core::node::rate_limit::SessionSetupRateLimiter<crate::time::HostClock>;
/// The lookup signing budget, on the host clock.
pub(crate) type LookupSignRateLimiter =
    fips_core::node::rate_limit::LookupSignRateLimiter<crate::time::HostClock>;
