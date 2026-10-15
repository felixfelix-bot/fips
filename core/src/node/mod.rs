//! Node-level bookkeeping that is not a protocol layer: the handshake,
//! session-setup and lookup-signing rate limiters, and the time-series stats
//! history. Both measure time on the host's clock, taken as a type parameter
//! bounded by [`crate::time::Clock`], with instants passed in.

pub mod rate_limit;
pub mod stats_history;
