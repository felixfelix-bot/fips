use core::hash::BuildHasher;

use super::KeyedHasher;
use crate::testutil::FixedEntropy;

#[test]
fn keyed_hasher_takes_its_key_from_the_entropy_source() {
    let key = [0x5au8; 16];
    let one = KeyedHasher::new::<FixedEntropy<1>>();
    let one_again = KeyedHasher::new::<FixedEntropy<1>>();
    let two = KeyedHasher::new::<FixedEntropy<2>>();

    assert_eq!(one.hash_one(key), one_again.hash_one(key));
    assert_ne!(one.hash_one(key), two.hash_one(key));
}

#[test]
fn keyed_hasher_draws_both_key_halves_from_the_entropy_source() {
    let hasher = KeyedHasher::new::<FixedEntropy<1>>();
    let half = u64::from_le_bytes([1; 8]);
    assert_eq!(hasher.k0, half);
    assert_eq!(hasher.k1, half);
}
