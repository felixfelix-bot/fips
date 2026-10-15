# core/src/node

Node-level bookkeeping that is not a protocol layer: the rate limiters
the node applies before expensive work, and the time-series history of
its statistics. The name mirrors the root crate's `src/node/`, where
these files came from and where the node state machine still lives; it
is the home that state machine moves into later.

## Files

- `mod.rs`: declares the two modules.
- `rate_limit.rs`: `TokenBucket`; `HandshakeRateLimiter`, the msg1
  limiter with a stranger bucket, an established-link bucket and a shared
  pending-handshake count (`PendingHandshake` releases a slot on drop);
  `derive_established_bucket`, which sizes the established-link bucket
  from configuration; `SessionSetupRateLimiter`, two buckets per FMP link
  peer for inbound FSP SessionSetup; and `LookupSignRateLimiter`, a
  bucket per link peer for the signatures this node makes answering
  lookups for itself. The module doc says why the msg1 limiter is
  unkeyed and the setup limiter is keyed on the link peer.
- `stats_history.rs`: `StatsHistory`, a fast ring (1 s x 3600) and a
  slow ring (1 min x 1440) per node-level metric, and the same per
  tracked peer, with NaN for gaps and `Series` serializing NaN as JSON
  `null`. Peers silent for `PEER_EVICTION_SECS` are dropped.
- `tests/`: the unit tests. Its own README describes them.

## How the root crate uses it

The root re-exports both modules from `src/node/rate_limit.rs` and
`src/node/stats_history.rs`, with type aliases that fix the clock to its
`HostClock`, so its callers keep their paths. The callers read the host
clock and pass the instant in.

## Invariants

- **Time is an argument.** Every type here is generic over
  `C: time::Clock` and takes `now: C::Instant`. Nothing here reads a
  clock, and every instant one value is given must come from the same
  clock.
- **Elapsed times saturate.** An instant earlier than the stored one
  counts as no time passed, as `saturating_duration_since` gives.
- **Per-peer maps are `BTreeMap`.** Their keys are link peers or tracked
  peers, bounded by the peer table and touched per handshake, per setup
  message or per tick, not per data packet. Iteration is sorted by
  `NodeAddr`.
- **Test-only accessors the root's tests use** are gated
  `#[cfg(any(test, feature = "test-hooks"))]`; the rest stay
  `#[cfg(test)]`.
