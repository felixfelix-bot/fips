//! Unit tests for `node::rate_limit`'s lookup signing budget. Attached as
//! that module's `sign_limiter_tests` child, a separate file so the tests keep
//! that module path.

use super::*;

fn addr(val: u8) -> NodeAddr {
    let mut bytes = [0u8; 16];
    bytes[0] = val;
    NodeAddr::from_bytes(bytes)
}

// The `DiscoveryBackoff` and `DiscoveryForwardRateLimiter` tests that
// accompanied this limiter in `src/node/discovery_rate_limit.rs` are not
// repeated here: both types moved into `crate::proto::lookup::limits` as
// `LookupBackoff` and `LookupForwardRateLimiter`, and all thirteen of those
// cases live there, name for name, in `core/src/proto/lookup/tests/limits.rs`,
// ported to the clockless millisecond API. Only the signing budget, which
// is `Instant`-based and so stays in the shell, is exercised below.

#[test]
fn test_sign_budget_is_spent_per_peer_and_does_not_touch_another_peer() {
    let mut limiter = LookupSignRateLimiter::with_params(4.0, 0.0);
    for _ in 0..4 {
        assert!(limiter.should_sign(&addr(1)));
    }
    assert!(
        !limiter.should_sign(&addr(1)),
        "the burst is the whole budget when nothing refills it"
    );
    assert!(
        limiter.should_sign(&addr(2)),
        "one peer spending its budget must not spend another's"
    );
    assert_eq!(limiter.len(), 2);
}

#[test]
fn test_sign_budget_of_zero_burst_is_read_as_unlimited() {
    let mut limiter = LookupSignRateLimiter::with_params(0.0, 0.0);
    for _ in 0..1000 {
        assert!(limiter.should_sign(&addr(1)));
    }
    assert_eq!(limiter.len(), 0, "unlimited keeps no per-peer state");
}
