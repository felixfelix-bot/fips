//! Unit tests for `node::rate_limit`: the token bucket, the msg1 limiter and
//! the session-setup limiter. Attached as that module's `tests` child.

use super::*;
use crate::testutil::{TestClock, TestInstant};
use alloc::string::ToString;

// The limiters on the tests' clock. Instants are named by `t`.
type TokenBucket = super::TokenBucket<TestClock>;
type HandshakeRateLimiter = super::HandshakeRateLimiter<TestClock>;
type SessionSetupRateLimiter = super::SessionSetupRateLimiter<TestClock>;

/// The instant `ms` milliseconds into the test.
fn t(ms: u64) -> TestInstant {
    TestInstant(ms)
}

#[test]
fn test_token_bucket_basic() {
    let mut bucket = TokenBucket::with_params(10, 1.0, t(0));

    // Should have full capacity
    assert_eq!(bucket.capacity(), 10);
    assert!(bucket.tokens(t(0)) >= 9.9); // Allow for timing

    // Consume all tokens
    for _ in 0..10 {
        assert!(bucket.try_acquire(t(0)));
    }

    // Should be empty
    assert!(!bucket.try_acquire(t(0)));
    assert!(!bucket.available(t(0)));
}

#[test]
fn test_token_bucket_refill() {
    let mut bucket = TokenBucket::with_params(10, 100.0, t(0)); // 100 tokens/sec

    // Drain completely
    for _ in 0..10 {
        bucket.try_acquire(t(0));
    }
    assert!(!bucket.available(t(0)));

    // Refill over 50 ms of the bucket's clock.
    let elapsed_secs = 0.05;

    // Expected tokens = elapsed * rate, capped at capacity.
    // Allow ±20% tolerance around the elapsed time.
    let expected = (elapsed_secs * 100.0f64).min(10.0);
    let lo = (expected * 0.8).min(expected - 0.5).max(0.0);
    let hi = (expected * 1.2).max(expected + 0.5).min(10.0);

    let tokens = bucket.tokens(t(50));
    assert!(
        (lo..=hi).contains(&tokens),
        "tokens: {}, expected ~{:.2} (range {:.2}..={:.2})",
        tokens,
        expected,
        lo,
        hi
    );
}

#[test]
fn test_token_bucket_try_acquire_n() {
    let mut bucket = TokenBucket::with_params(10, 1.0, t(0));

    // Acquire 5
    assert!(bucket.try_acquire_n(5, t(0)));
    assert!(bucket.tokens(t(0)) >= 4.9 && bucket.tokens(t(0)) <= 5.1);

    // Acquire 5 more
    assert!(bucket.try_acquire_n(5, t(0)));

    // Can't acquire more
    assert!(!bucket.try_acquire_n(1, t(0)));
}

#[test]
fn test_token_bucket_reset() {
    let mut bucket = TokenBucket::with_params(10, 1.0, t(0));

    // Drain
    for _ in 0..10 {
        bucket.try_acquire(t(0));
    }

    // Reset
    bucket.reset(t(0));

    // Should be full again
    assert!(bucket.tokens(t(0)) >= 9.9);
}

#[test]
fn test_token_bucket_time_until_available() {
    let mut bucket = TokenBucket::with_params(10, 10.0, t(0)); // 10 tokens/sec

    // When full, should be zero
    assert_eq!(bucket.time_until_available(t(0)), Duration::ZERO);

    // Drain completely
    for _ in 0..10 {
        bucket.try_acquire(t(0));
    }

    // Should need ~100ms for one token at 10/sec
    let wait = bucket.time_until_available(t(0));
    assert!(wait.as_millis() >= 90 && wait.as_millis() <= 110);
}

fn test_limiter(bucket: TokenBucket, max_pending: usize) -> HandshakeRateLimiter {
    HandshakeRateLimiter::with_params(
        bucket,
        TokenBucket::with_params(1000, 100.0, t(0)),
        max_pending,
    )
}

#[test]
fn test_handshake_rate_limiter_basic() {
    let mut limiter = test_limiter(TokenBucket::new(t(0)), 100);

    assert!(limiter.can_start_handshake(Msg1Class::Stranger, t(0)));
    assert_eq!(limiter.pending_count(), 0);

    // Start a handshake
    let slot = limiter.start_handshake(Msg1Class::Stranger, t(0)).unwrap();
    assert_eq!(limiter.pending_count(), 1);

    // Complete it
    drop(slot);
    assert_eq!(limiter.pending_count(), 0);
}

#[test]
fn test_handshake_rate_limiter_max_pending() {
    let bucket = TokenBucket::with_params(1000, 100.0, t(0));
    let mut limiter = test_limiter(bucket, 3);

    // Start 3 handshakes
    let a = limiter.start_handshake(Msg1Class::Stranger, t(0)).unwrap();
    let _b = limiter.start_handshake(Msg1Class::Stranger, t(0)).unwrap();
    let _c = limiter.start_handshake(Msg1Class::Stranger, t(0)).unwrap();

    // Fourth should fail (max pending)
    assert!(!limiter.can_start_handshake(Msg1Class::Stranger, t(0)));
    assert_eq!(
        limiter
            .start_handshake(Msg1Class::Stranger, t(0))
            .unwrap_err(),
        Msg1Refusal::PendingLimit
    );

    // Complete one
    drop(a);

    // Now should be able to start another
    assert!(limiter.can_start_handshake(Msg1Class::Stranger, t(0)));
    assert!(limiter.start_handshake(Msg1Class::Stranger, t(0)).is_ok());
}

#[test]
fn test_handshake_rate_limiter_token_exhaustion() {
    let bucket = TokenBucket::with_params(5, 0.0, t(0)); // No refill
    let mut limiter = test_limiter(bucket, 100);

    // Start 5 handshakes (exhausts tokens), releasing each immediately
    for _ in 0..5 {
        let slot = limiter.start_handshake(Msg1Class::Stranger, t(0)).unwrap();
        drop(slot);
    }

    // Tokens exhausted, even though pending is 0
    assert_eq!(limiter.pending_count(), 0);
    assert!(!limiter.can_start_handshake(Msg1Class::Stranger, t(0)));
    assert_eq!(
        limiter
            .start_handshake(Msg1Class::Stranger, t(0))
            .unwrap_err(),
        Msg1Refusal::RateLimit
    );
}

#[test]
fn test_handshake_rate_limiter_reset() {
    let mut limiter = test_limiter(TokenBucket::new(t(0)), 100);

    // Start some handshakes
    let _a = limiter.start_handshake(Msg1Class::Stranger, t(0)).unwrap();
    let _b = limiter.start_handshake(Msg1Class::Stranger, t(0)).unwrap();
    assert_eq!(limiter.pending_count(), 2);

    // Reset
    limiter.reset(t(0));

    assert_eq!(limiter.pending_count(), 0);
    assert!(limiter.bucket().tokens >= DEFAULT_BURST_CAPACITY as f64 - 0.1);
}

/// The pending slot is released by `Drop`, not by any reachable manual
/// call, and the release saturates at zero rather than underflowing.
#[test]
fn pending_slot_releases_on_drop_and_saturates() {
    let mut limiter = test_limiter(TokenBucket::with_params(100, 100.0, t(0)), 100);

    let outer = limiter.start_handshake(Msg1Class::Stranger, t(0)).unwrap();
    {
        let _inner = limiter.start_handshake(Msg1Class::Stranger, t(0)).unwrap();
        assert_eq!(limiter.pending_count(), 2);
    }
    assert_eq!(
        limiter.pending_count(),
        1,
        "inner guard released its own slot"
    );

    // A reset zeroes the counter while a guard is still live; the
    // guard's later drop must not underflow it.
    limiter.reset(t(0));
    assert_eq!(limiter.pending_count(), 0);
    drop(outer);
    assert_eq!(limiter.pending_count(), 0, "release saturates at zero");
}

/// The refusal reason discriminates the two limbs. This is what the
/// `refused_by` log field in `handle_msg1` carries; the log line itself
/// is not asserted here.
#[test]
fn refusal_reason_names_the_limb() {
    // Pending limb: max_pending 1, one live guard, tokens plentiful.
    let mut pending_bound = test_limiter(TokenBucket::with_params(1000, 100.0, t(0)), 1);
    let _held = pending_bound
        .start_handshake(Msg1Class::Stranger, t(0))
        .unwrap();
    assert_eq!(
        pending_bound
            .start_handshake(Msg1Class::Stranger, t(0))
            .unwrap_err(),
        Msg1Refusal::PendingLimit
    );

    // Rate limb: pending headroom, zero-capacity/zero-refill bucket.
    let mut rate_bound = test_limiter(TokenBucket::with_params(0, 0.0, t(0)), 1000);
    assert_eq!(
        rate_bound
            .start_handshake(Msg1Class::Stranger, t(0))
            .unwrap_err(),
        Msg1Refusal::RateLimit
    );

    assert_eq!(Msg1Refusal::PendingLimit.to_string(), "pending_limit");
    assert_eq!(Msg1Refusal::RateLimit.to_string(), "rate_limit");
}

/// The two classes draw on separate buckets: draining one leaves the
/// other untouched.
#[test]
fn msg1_classes_draw_on_separate_buckets() {
    let mut limiter = HandshakeRateLimiter::with_params(
        TokenBucket::with_params(1, 0.0, t(0)),
        TokenBucket::with_params(3, 0.0, t(0)),
        1000,
    );

    drop(limiter.start_handshake(Msg1Class::Stranger, t(0)).unwrap());
    assert_eq!(
        limiter
            .start_handshake(Msg1Class::Stranger, t(0))
            .unwrap_err(),
        Msg1Refusal::RateLimit,
        "stranger bucket drained"
    );

    for _ in 0..3 {
        drop(
            limiter
                .start_handshake(Msg1Class::EstablishedLink, t(0))
                .expect("established bucket is independent of the stranger bucket"),
        );
    }
    assert_eq!(
        limiter
            .start_handshake(Msg1Class::EstablishedLink, t(0))
            .unwrap_err(),
        Msg1Refusal::RateLimit,
        "established bucket drains on its own terms"
    );
}

#[test]
fn derive_established_bucket_from_shipped_defaults() {
    // max_peers 128, rekey.after_secs 120, handshake_max_resends 5.
    let (burst, rate) = derive_established_bucket(128, 120, 5, 100, 10.0);
    assert_eq!(burst, 128);
    assert!(
        (rate - 6.4).abs() < 1e-9,
        "expected 6.4 tokens/sec, got {rate}"
    );

    // Scales with max_peers rather than needing a revisit.
    let (burst, rate) = derive_established_bucket(512, 120, 5, 100, 10.0);
    assert_eq!(burst, 512);
    assert!(
        (rate - 25.6).abs() < 1e-9,
        "expected 25.6 tokens/sec, got {rate}"
    );
}

/// `after_secs = 0` must not divide by zero. It is clamped to 1s, which
/// yields a *large* rate, not the floor — the floor binds at the other
/// end, for a very long rekey period. Both ends are asserted here
/// because the two are easy to conflate.
#[test]
fn derive_established_bucket_degenerate_rekey_periods() {
    let (burst, rate) = derive_established_bucket(128, 0, 5, 100, 10.0);
    assert_eq!(burst, 128);
    assert!(rate.is_finite(), "after_secs = 0 must not divide by zero");
    assert_eq!(
        rate,
        derive_established_bucket(128, 1, 5, 100, 10.0).1,
        "after_secs = 0 is clamped to 1s"
    );
    assert!(
        rate > ESTABLISHED_RATE_FLOOR,
        "a zero rekey period is the high end, not the floor"
    );

    // The floor binds for a very long / effectively disabled period.
    let (_, rate) = derive_established_bucket(1, 100_000, 5, 100, 10.0);
    assert_eq!(rate, ESTABLISHED_RATE_FLOOR);
}

/// `max_peers == 0` means unlimited. Deriving a burst from it would
/// yield a zero- or one-token bucket, which is worse than the bug this
/// second bucket exists to fix; the stranger parameters are reused.
#[test]
fn derive_established_bucket_unlimited_peers_reuses_stranger_params() {
    let (burst, rate) = derive_established_bucket(0, 120, 5, 100, 10.0);
    assert_eq!(burst, 100);
    assert_eq!(rate, 10.0);
}

fn addr(byte: u8) -> NodeAddr {
    NodeAddr::from_bytes([byte; 16])
}

#[test]
fn setup_limiter_draining_one_link_leaves_another_links_budget_untouched() {
    let mut limiter = SessionSetupRateLimiter::with_params((2, 0.001), (2, 0.001));
    let noisy = addr(0x01);
    let quiet = addr(0x02);

    assert!(limiter.try_admit(&noisy, Msg1Class::Stranger, t(0)));
    assert!(limiter.try_admit(&noisy, Msg1Class::Stranger, t(0)));
    assert!(
        !limiter.try_admit(&noisy, Msg1Class::Stranger, t(0)),
        "the noisy link's own bucket must run out"
    );
    assert!(
        limiter.try_admit(&quiet, Msg1Class::Stranger, t(0)),
        "a second link peer must not share the first one's budget"
    );
    assert_eq!(limiter.len(), 2);
}

#[test]
fn setup_limiter_draining_the_stranger_bucket_still_admits_established_peer_setups() {
    let mut limiter = SessionSetupRateLimiter::with_params((1, 0.001), (1, 0.001));
    let link = addr(0x01);

    assert!(limiter.try_admit(&link, Msg1Class::Stranger, t(0)));
    assert!(!limiter.try_admit(&link, Msg1Class::Stranger, t(0)));
    assert!(
        limiter.try_admit(&link, Msg1Class::EstablishedLink, t(0)),
        "rekey traffic must not be starved by a stranger flood on the \
         same link; suppressed rotation is silent and would show only as \
         a flat rekey_armed counter"
    );
}

/// A link's buckets are dropped once it has been idle for the full
/// [`SETUP_BUCKET_IDLE`], and not a millisecond before.
#[test]
fn setup_buckets_are_pruned_after_the_idle_period() {
    let mut limiter = SessionSetupRateLimiter::with_params((10, 1.0), (10, 1.0));
    let idle = addr(0x01);
    let busy = addr(0x02);
    let idle_ms = SETUP_BUCKET_IDLE.as_millis() as u64;

    assert!(limiter.try_admit(&idle, Msg1Class::Stranger, t(0)));
    assert!(limiter.try_admit(&busy, Msg1Class::Stranger, t(idle_ms - 1)));
    assert_eq!(
        limiter.len(),
        2,
        "a link idle for less than the idle period keeps its buckets"
    );

    assert!(limiter.try_admit(&busy, Msg1Class::Stranger, t(idle_ms)));
    assert_eq!(
        limiter.len(),
        1,
        "a link idle for the whole idle period is pruned"
    );
}
