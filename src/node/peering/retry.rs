//! Cross-attempt retry state for auto-connect peers.
//!
//! [`RetryState`] is the durable per-peer schedule entry the peering reconciler
//! owns (it lives in [`crate::node::peering::reconcile::PeeringReconciler`], not
//! on a per-connection object, because a fresh connection is created per re-dial
//! so the escalating backoff count must persist across attempts). The decision
//! logic that reads and mutates it — the retry-dial phase and the
//! `on_handshake_timeout` / `on_link_dead` reflexes — lives in the sans-IO
//! reconciler core; the driver wrappers that feed it (retry-dial I/O, the
//! gate-guarded reflex call sites) live in [`super::driver`].

use crate::config::PeerConfig;
use fips_core::time::Clock;

/// The clock the peering retry schedule is measured on. It is still the wall
/// clock, as the schedule was before it took the core's time type; times the
/// control API reports go through `WallStamp`.
pub(crate) type PeeringClock = crate::time::WallClock;

/// An instant on [`PeeringClock`].
pub(crate) type PeeringInstant = <PeeringClock as Clock>::Instant;

/// The peering clock's instant `ms` milliseconds after its test epoch.
#[cfg(test)]
pub(crate) fn peering_at(ms: u64) -> PeeringInstant {
    crate::time::at::<PeeringClock>(ms)
}

/// Per-tick cap on retry-dial connection attempts (ceiling).
pub(in crate::node) const MAX_RETRY_CONNECTIONS_PER_TICK: usize = 16;

/// Tracks retry state for a peer across connection attempts.
pub struct RetryState {
    /// The peer config to use for initiating retries.
    pub peer_config: PeerConfig,

    /// Number of retries attempted so far.
    pub retry_count: u32,

    /// When the next retry should be attempted; `None` means it is due now.
    pub(crate) retry_after: Option<PeeringInstant>,

    /// Whether this is an auto-reconnect (unlimited retries, ignores max_retries).
    pub reconnect: bool,

    /// Optional absolute expiry for this retry entry.
    ///
    /// When set, retries are dropped after this point even if reconnect logic
    /// would otherwise continue.
    pub(crate) expires_at: Option<PeeringInstant>,
}

impl RetryState {
    /// Create a new retry state for a peer.
    pub fn new(peer_config: PeerConfig) -> Self {
        Self {
            peer_config,
            retry_count: 0,
            retry_after: None,
            reconnect: false,
            expires_at: None,
        }
    }
}
