//! Tests for the clock traits: the arithmetic contract on the millisecond
//! test instant and on a tick-count instant shaped like an embedded timer's.

use super::{Clock, Duration, Instant};
use crate::testutil::{TestClock, TestInstant};

/// Ticks per second of [`TickInstant`]: a 32 768 Hz crystal, the rate of a
/// typical microcontroller real-time clock.
const TICK_HZ: u64 = 32_768;

/// An instant counted in ticks of a 32 768 Hz timer, as an embedded host's
/// clock counts them. It exists to show that the trait admits such a clock
/// with no allocation and no `std`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TickInstant(u64);

/// Whole ticks in `d`, rounded down, or `None` if they do not fit in a `u64`.
fn ticks_of(d: Duration) -> Option<u64> {
    u64::try_from(d.as_nanos() * u128::from(TICK_HZ) / 1_000_000_000).ok()
}

impl Instant for TickInstant {
    fn saturating_duration_since(self, earlier: Self) -> Duration {
        let ticks = self.0.saturating_sub(earlier.0);
        let nanos = (ticks % TICK_HZ) * 1_000_000_000 / TICK_HZ;
        Duration::new(ticks / TICK_HZ, nanos as u32)
    }

    fn checked_add(self, d: Duration) -> Option<Self> {
        self.0.checked_add(ticks_of(d)?).map(Self)
    }

    fn checked_sub(self, d: Duration) -> Option<Self> {
        self.0.checked_sub(ticks_of(d)?).map(Self)
    }
}

/// The contract every instant type keeps, checked from `base`, which must be
/// at least one second from either end of its representation.
fn check_contract<I: Instant>(base: I) {
    let second = Duration::from_secs(1);
    let later = base
        .checked_add(second)
        .expect("one second after base fits");
    let earlier = base
        .checked_sub(second)
        .expect("one second before base fits");

    assert!(earlier < base && base < later);
    assert_eq!(later.saturating_duration_since(base), second);
    assert_eq!(base.saturating_duration_since(earlier), second);
    assert_eq!(later.saturating_duration_since(earlier), 2 * second);
    assert_eq!(base.saturating_duration_since(later), Duration::ZERO);
    assert_eq!(base.saturating_duration_since(base), Duration::ZERO);
    assert_eq!(later.checked_sub(second), Some(base));
    assert_eq!(base.checked_add(Duration::ZERO), Some(base));
}

#[test]
fn the_millisecond_test_instant_keeps_the_instant_contract() {
    check_contract(TestInstant(10_000));
}

#[test]
fn a_tick_count_instant_keeps_the_instant_contract() {
    check_contract(TickInstant(10 * TICK_HZ));
}

#[test]
fn the_millisecond_test_instant_saturates_the_difference_at_zero() {
    assert_eq!(
        TestInstant(5).saturating_duration_since(TestInstant(9)),
        Duration::ZERO
    );
    assert_eq!(
        TestInstant(9).saturating_duration_since(TestInstant(5)),
        Duration::from_millis(4)
    );
}

#[test]
fn the_millisecond_test_instant_reports_overflow_and_underflow_as_none() {
    assert_eq!(
        TestInstant(u64::MAX).checked_add(Duration::from_millis(1)),
        None
    );
    assert_eq!(
        TestInstant(u64::MAX - 1).checked_add(Duration::from_millis(1)),
        Some(TestInstant(u64::MAX))
    );
    assert_eq!(TestInstant(0).checked_sub(Duration::from_millis(1)), None);
    assert_eq!(
        TestInstant(1).checked_sub(Duration::from_millis(1)),
        Some(TestInstant(0))
    );
    assert_eq!(TestInstant(0).checked_add(Duration::MAX), None);
}

#[test]
fn a_tick_count_instant_saturates_the_difference_at_zero() {
    let a = TickInstant(TICK_HZ);
    let b = TickInstant(3 * TICK_HZ);
    assert_eq!(a.saturating_duration_since(b), Duration::ZERO);
    assert_eq!(b.saturating_duration_since(a), Duration::from_secs(2));
}

#[test]
fn a_tick_count_instant_reports_overflow_at_the_last_tick_and_underflow_below_zero() {
    let one_tick = Duration::from_nanos(1_000_000_000 / TICK_HZ + 1);
    assert_eq!(TickInstant(u64::MAX).checked_add(one_tick), None);
    assert_eq!(
        TickInstant(u64::MAX - 1).checked_add(one_tick),
        Some(TickInstant(u64::MAX))
    );
    assert_eq!(TickInstant(0).checked_sub(one_tick), None);
    assert_eq!(TickInstant(0).checked_add(Duration::MAX), None);
}

#[test]
fn a_tick_count_instant_measures_fractions_of_a_second() {
    let half = TickInstant(TICK_HZ / 2);
    assert_eq!(
        half.saturating_duration_since(TickInstant(0)),
        Duration::from_millis(500)
    );
    assert_eq!(
        TickInstant(0).checked_add(Duration::from_millis(250)),
        Some(TickInstant(TICK_HZ / 4))
    );
}

#[test]
fn the_test_clock_reads_what_the_test_advanced_it_to() {
    let clock = TestClock::default();
    assert_eq!(clock.now(), TestInstant(0));
    clock.advance(1_500);
    assert_eq!(clock.now(), TestInstant(1_500));
    clock.advance(500);
    assert_eq!(clock.now(), TestInstant(2_000));
}
