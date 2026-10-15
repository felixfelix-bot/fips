//! Test fixtures shared by the crate's unit tests.

use crate::identity::NodeAddr;

/// Build a `NodeAddr` from a single discriminating byte in position 0.
pub(crate) fn make_node_addr(val: u8) -> NodeAddr {
    let mut bytes = [0u8; 16];
    bytes[0] = val;
    NodeAddr::from_bytes(bytes)
}

/// A random secp256k1 keypair, built the way `Identity::generate` builds one.
pub(crate) fn make_keypair() -> secp256k1::Keypair {
    let mut secret = [0u8; 32];
    rand::Rng::fill_bytes(&mut rand::rng(), &mut secret);
    let key =
        secp256k1::SecretKey::from_slice(&secret).expect("32 random bytes is a valid secret key");
    secp256k1::Keypair::from_secret_key(crate::identity::secp(), &key)
}

/// An entropy source that fills every byte with `B`, so a test can tell
/// which source a value was drawn from.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FixedEntropy<const B: u8>;

impl<const B: u8> crate::entropy::EntropySource for FixedEntropy<B> {
    fn fill(dest: &mut [u8]) {
        dest.fill(B);
    }
}

/// The random source for tests: `rand`'s thread-local generator.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct TestEntropy;

impl crate::entropy::EntropySource for TestEntropy {
    fn fill(dest: &mut [u8]) {
        rand::Rng::fill_bytes(&mut rand::rng(), dest);
    }
}

/// An instant of [`TestClock`]: whole milliseconds from 0.
///
/// Durations convert to whole milliseconds; a fraction of a millisecond is
/// dropped.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct TestInstant(pub(crate) u64);

impl crate::time::Instant for TestInstant {
    fn saturating_duration_since(self, earlier: Self) -> crate::time::Duration {
        crate::time::Duration::from_millis(self.0.saturating_sub(earlier.0))
    }

    fn checked_add(self, d: crate::time::Duration) -> Option<Self> {
        let ms = u64::try_from(d.as_millis()).ok()?;
        self.0.checked_add(ms).map(Self)
    }

    fn checked_sub(self, d: crate::time::Duration) -> Option<Self> {
        let ms = u64::try_from(d.as_millis()).ok()?;
        self.0.checked_sub(ms).map(Self)
    }
}

/// The clock for tests: it reads what the test last set, starting at 0, and
/// moves only when the test advances it.
#[derive(Debug, Default)]
pub(crate) struct TestClock {
    now: core::cell::Cell<TestInstant>,
}

impl TestClock {
    /// Move the clock forward by `ms` milliseconds.
    pub(crate) fn advance(&self, ms: u64) {
        self.now.set(TestInstant(self.now.get().0 + ms));
    }
}

impl crate::time::Clock for TestClock {
    type Instant = TestInstant;

    fn now(&self) -> TestInstant {
        self.now.get()
    }
}
