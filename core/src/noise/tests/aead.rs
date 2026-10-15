//! Known-answer and cross-backend tests for the AEAD backends.

use super::*;

/// RFC 8439 section 2.8.2: key, nonce, AAD, plaintext, ciphertext and tag.
const RFC_KEY: [u8; 32] = [
    0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f,
    0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f,
];
const RFC_NONCE: [u8; 12] = [
    0x07, 0x00, 0x00, 0x00, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47,
];
const RFC_AAD: [u8; 12] = [
    0x50, 0x51, 0x52, 0x53, 0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7,
];
const RFC_PLAINTEXT: &[u8] = b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.";
const RFC_CIPHERTEXT: [u8; 114] = [
    0xd3, 0x1a, 0x8d, 0x34, 0x64, 0x8e, 0x60, 0xdb, 0x7b, 0x86, 0xaf, 0xbc, 0x53, 0xef, 0x7e, 0xc2,
    0xa4, 0xad, 0xed, 0x51, 0x29, 0x6e, 0x08, 0xfe, 0xa9, 0xe2, 0xb5, 0xa7, 0x36, 0xee, 0x62, 0xd6,
    0x3d, 0xbe, 0xa4, 0x5e, 0x8c, 0xa9, 0x67, 0x12, 0x82, 0xfa, 0xfb, 0x69, 0xda, 0x92, 0x72, 0x8b,
    0x1a, 0x71, 0xde, 0x0a, 0x9e, 0x06, 0x0b, 0x29, 0x05, 0xd6, 0xa5, 0xb6, 0x7e, 0xcd, 0x3b, 0x36,
    0x92, 0xdd, 0xbd, 0x7f, 0x2d, 0x77, 0x8b, 0x8c, 0x98, 0x03, 0xae, 0xe3, 0x28, 0x09, 0x1b, 0x58,
    0xfa, 0xb3, 0x24, 0xe4, 0xfa, 0xd6, 0x75, 0x94, 0x55, 0x85, 0x80, 0x8b, 0x48, 0x31, 0xd7, 0xbc,
    0x3f, 0xf4, 0xde, 0xf0, 0x8e, 0x4b, 0x7a, 0x9d, 0xe5, 0x76, 0xd2, 0x65, 0x86, 0xce, 0xc6, 0x4b,
    0x61, 0x16,
];
const RFC_TAG: [u8; 16] = [
    0x1a, 0xe1, 0x0b, 0x59, 0x4f, 0x09, 0xe2, 0x6a, 0x7e, 0x90, 0x2e, 0xcb, 0xd0, 0x60, 0x06, 0x91,
];

/// Seal the RFC plaintext and check the published ciphertext and tag, then
/// open the published ciphertext and tag back to the plaintext.
fn check_rfc_vector<A: Aead>() {
    let key = A::with_key(&RFC_KEY).expect("32-byte key");
    let mut buf = RFC_PLAINTEXT.to_vec();
    let tag = key.seal(&RFC_NONCE, &RFC_AAD, &mut buf).expect("seal");
    assert_eq!(buf, RFC_CIPHERTEXT);
    assert_eq!(tag, RFC_TAG);

    let mut sealed = RFC_CIPHERTEXT.to_vec();
    sealed.extend_from_slice(&RFC_TAG);
    let len = key.open(&RFC_NONCE, &RFC_AAD, &mut sealed).expect("open");
    assert_eq!(&sealed[..len], RFC_PLAINTEXT);
}

/// A flipped tag bit, a flipped ciphertext bit, other AAD and a buffer
/// shorter than a tag all fail to open, and none panics.
fn check_rejects<A: Aead>() {
    let key = A::with_key(&RFC_KEY).expect("32-byte key");
    let mut sealed = RFC_CIPHERTEXT.to_vec();
    sealed.extend_from_slice(&RFC_TAG);

    let mut bad_tag = sealed.clone();
    *bad_tag.last_mut().unwrap() ^= 1;
    assert_eq!(key.open(&RFC_NONCE, &RFC_AAD, &mut bad_tag), Err(AeadError));

    let mut bad_ct = sealed.clone();
    bad_ct[0] ^= 1;
    assert_eq!(key.open(&RFC_NONCE, &RFC_AAD, &mut bad_ct), Err(AeadError));

    let mut other_aad = sealed.clone();
    assert_eq!(key.open(&RFC_NONCE, &[], &mut other_aad), Err(AeadError));

    for len in 0..TAG_SIZE {
        let mut short = sealed[..len].to_vec();
        assert_eq!(key.open(&RFC_NONCE, &RFC_AAD, &mut short), Err(AeadError));
    }
}

#[test]
fn rfc8439_vector_rustcrypto() {
    check_rfc_vector::<chacha20poly1305::ChaCha20Poly1305>();
}

#[test]
fn rustcrypto_rejects_altered_or_short_input() {
    check_rejects::<chacha20poly1305::ChaCha20Poly1305>();
}

#[cfg(feature = "ring")]
#[test]
fn rfc8439_vector_ring() {
    check_rfc_vector::<ring::aead::LessSafeKey>();
}

#[cfg(feature = "ring")]
#[test]
fn ring_rejects_altered_or_short_input() {
    check_rejects::<ring::aead::LessSafeKey>();
}

/// Sealed by one backend, opened by the other, in both directions, at every
/// payload length from empty to 2048 bytes and at 9000, under no AAD and
/// under the 12- and 16-byte headers the session and link layers bind, with
/// the transport's nonce layout. The key, plaintext and AAD are fixed, so a
/// failure reproduces. The ciphertexts and tags must be byte-identical, since
/// both backends compute RFC 8439.
#[cfg(feature = "ring")]
#[test]
fn ring_and_rustcrypto_interoperate_both_ways() {
    let key_bytes: [u8; KEY_SIZE] = core::array::from_fn(|i| (i as u8).wrapping_mul(29) ^ 0x6b);
    let ring_key = <ring::aead::LessSafeKey as Aead>::with_key(&key_bytes).unwrap();
    let rc_key = <chacha20poly1305::ChaCha20Poly1305 as Aead>::with_key(&key_bytes).unwrap();

    let mut counter = 0u64;
    for aad_len in [0usize, 12, 16] {
        let aad: alloc::vec::Vec<u8> = (0..aad_len).map(|i| (i as u8) ^ 0xa5).collect();
        for len in (0..=2048).chain([9000]) {
            let n = nonce(counter);
            counter = counter.wrapping_add(0x0101_0101_0101);
            let plaintext: alloc::vec::Vec<u8> = (0..len)
                .map(|i| (i as u8).wrapping_mul(31) ^ ((i >> 8) as u8))
                .collect();

            let mut by_ring = plaintext.clone();
            let ring_tag = ring_key.seal(&n, &aad, &mut by_ring).unwrap();
            let mut by_rc = plaintext.clone();
            let rc_tag = rc_key.seal(&n, &aad, &mut by_rc).unwrap();
            assert_eq!(
                by_ring, by_rc,
                "ciphertexts differ: length {len}, aad {aad_len}"
            );
            assert_eq!(ring_tag, rc_tag, "tags differ: length {len}, aad {aad_len}");

            let mut to_rc = by_ring.clone();
            to_rc.extend_from_slice(&ring_tag);
            let got = rc_key.open(&n, &aad, &mut to_rc).unwrap();
            assert_eq!(&to_rc[..got], &plaintext[..], "length {len}, aad {aad_len}");

            let mut to_ring = by_rc.clone();
            to_ring.extend_from_slice(&rc_tag);
            let got = ring_key.open(&n, &aad, &mut to_ring).unwrap();
            assert_eq!(
                &to_ring[..got],
                &plaintext[..],
                "length {len}, aad {aad_len}"
            );
        }
    }
}

#[test]
fn nonce_is_four_zero_bytes_then_the_counter_little_endian() {
    assert_eq!(
        nonce(0x0102_0304_0506_0708),
        [0, 0, 0, 0, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]
    );
    assert_eq!(nonce(0), [0u8; 12]);
}
