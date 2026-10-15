//! Tests for the host clocks, the wall stamps, deadlines and the test hooks.

use std::time::{Duration, Instant as StdInstant};

use fips_core::time::{Clock, Instant};

use super::*;

/// Slack allowed between two reads of a real clock in one test.
const SLACK_MS: u64 = 1_000;

/// Whether `value` is within [`SLACK_MS`] after `expected`.
fn near(value: u64, expected: u64) -> bool {
    value >= expected && value - expected <= SLACK_MS
}

#[test]
fn host_instant_arithmetic_matches_std_instant() {
    let base = StdInstant::now();
    let host = HostInstant::from(base);
    let d = Duration::from_millis(1_234);
    let later = base + d;

    assert_eq!(
        host.checked_add(d).map(StdInstant::from),
        base.checked_add(d)
    );
    assert_eq!(
        HostInstant::from(later)
            .checked_sub(d)
            .map(StdInstant::from),
        later.checked_sub(d)
    );
    assert_eq!(
        HostInstant::from(later).saturating_duration_since(host),
        later.saturating_duration_since(base)
    );
    assert_eq!(
        host.saturating_duration_since(HostInstant::from(later)),
        Duration::ZERO
    );
}

#[test]
fn a_wall_millis_stamp_is_its_stored_value() {
    assert_eq!(
        WallStamp::wall_ms(WallMillis(1_700_000_000_123)),
        1_700_000_000_123
    );
    assert_eq!(WallStamp::wall_ms(WallMillis(0)), 0);
}

#[test]
fn a_past_host_instant_stamps_as_the_wall_clock_less_its_age() {
    let past = HostClock
        .now()
        .checked_sub(Duration::from_secs(10))
        .expect("the host has been up ten seconds");
    let expected = wall_ms() - 10_000;
    let stamp = WallStamp::wall_ms(past);
    assert!(
        near(stamp, expected - SLACK_MS / 2),
        "stamp {stamp}, expected about {expected}"
    );
}

#[test]
fn a_future_host_instant_stamps_as_the_wall_clock_plus_the_time_until_it() {
    let future = HostClock
        .now()
        .checked_add(Duration::from_secs(10))
        .expect("ten seconds ahead fits");
    let expected = wall_ms() + 10_000;
    let stamp = WallStamp::wall_ms(future);
    assert!(
        near(stamp, expected - SLACK_MS / 2),
        "stamp {stamp}, expected about {expected}"
    );
}

#[test]
fn a_wall_clock_test_instant_is_the_literal_milliseconds() {
    assert_eq!(at::<WallClock>(0), WallMillis(0));
    assert_eq!(at::<WallClock>(34_999), WallMillis(34_999));
}

#[test]
fn host_clock_test_instants_are_spaced_by_their_milliseconds() {
    let a = at::<HostClock>(1_000);
    let b = at::<HostClock>(4_500);
    assert_eq!(b.saturating_duration_since(a), Duration::from_millis(3_500));
    assert_eq!(at::<HostClock>(0), HostClock::epoch());
    assert!(at::<HostClock>(0) < a);
}

#[test]
fn a_deadline_is_now_plus_the_delay_below_the_ceiling() {
    assert_eq!(
        deadline(WallMillis(5_000), Duration::from_millis(2_500)),
        WallMillis(7_500)
    );
    let now = HostClock.now();
    assert_eq!(
        deadline(now, Duration::from_secs(30)),
        now.checked_add(Duration::from_secs(30)).unwrap()
    );
}

#[test]
fn a_deadline_is_capped_at_the_ceiling() {
    let now = WallMillis(1_700_000_000_000);
    let capped = now.checked_add(CEILING).unwrap();
    assert_eq!(deadline(now, CEILING), capped);
    assert_eq!(deadline(now, CEILING + Duration::from_secs(1)), capped);
    assert_eq!(deadline(now, Duration::MAX), capped);

    let host = HostClock.now();
    assert_eq!(
        deadline(host, Duration::MAX),
        host.checked_add(CEILING).unwrap()
    );
}

#[test]
fn a_deadline_past_the_end_of_the_range_stops_short_of_it_without_panicking() {
    let now = WallMillis(u64::MAX - 10);
    let at = deadline(now, Duration::from_secs(60));
    assert!(at >= now);
    assert_eq!(
        deadline(WallMillis(u64::MAX), Duration::from_secs(60)),
        WallMillis(u64::MAX)
    );
}

#[test]
fn stepping_the_wall_clock_moves_wall_time_and_not_the_monotonic_clock() {
    let wall_before = wall_ms();
    let mono_before = HostClock.now();
    let ms_before = mono_ms();

    step_wall(3_600_000);
    assert!(near(wall_ms(), wall_before + 3_600_000));
    assert!(near(WallClock.now().0, wall_before + 3_600_000));
    assert!(
        HostClock.now().saturating_duration_since(mono_before) < Duration::from_millis(SLACK_MS)
    );
    assert!(near(mono_ms(), ms_before));

    step_wall(-7_200_000);
    assert!(near(wall_ms(), wall_before - 3_600_000));
}

#[test]
fn advancing_the_monotonic_clock_moves_it_and_not_wall_time() {
    let wall_before = wall_ms();
    let mono_before = HostClock.now();
    let ms_before = mono_ms();

    advance(Duration::from_secs(3_600));
    let moved = HostClock.now().saturating_duration_since(mono_before);
    assert!(moved >= Duration::from_secs(3_600));
    assert!(moved < Duration::from_secs(3_600) + Duration::from_millis(SLACK_MS));
    assert!(near(mono_ms(), ms_before + 3_600_000));
    assert!(near(wall_ms(), wall_before));
}

#[test]
fn wall_millis_differences_saturate_like_u64_saturating_sub() {
    for (a, b) in [(9_u64, 5_u64), (5, 9), (0, 0), (u64::MAX, 0), (0, u64::MAX)] {
        assert_eq!(
            WallMillis(a).saturating_duration_since(WallMillis(b)),
            Duration::from_millis(a.saturating_sub(b))
        );
    }
    assert_eq!(
        WallMillis(u64::MAX).checked_add(Duration::from_millis(1)),
        None
    );
    assert_eq!(WallMillis(0).checked_sub(Duration::from_millis(1)), None);
}
