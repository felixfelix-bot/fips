//! The stats history lives in `fips-core`; this module re-exports it, with
//! its clock fixed to [`HostClock`](crate::time::HostClock).

pub use fips_core::node::stats_history::{
    ALL_METRICS, ALL_PEER_METRICS, FAST_RING_CAPACITY, Granularity, Metric, PEER_EVICTION_SECS,
    PeerMetric, SLOW_RING_CAPACITY, Series, Snapshot,
};

/// The stats history, on the host clock.
pub(crate) type StatsHistory = fips_core::node::stats_history::StatsHistory<crate::time::HostClock>;
/// One peer's state at a stats tick, on the host clock.
pub(crate) type PeerSnapshot = fips_core::node::stats_history::PeerSnapshot<crate::time::HostClock>;
