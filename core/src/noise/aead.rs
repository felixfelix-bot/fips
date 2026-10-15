//! The AEAD under Noise: ChaCha20-Poly1305 (RFC 8439).
//!
//! Two backends implement [`Aead`], and one is chosen at compile time as
//! [`AeadKey`]: `ring`'s `LessSafeKey` when this crate's `ring` feature is
//! on, which hosted builds turn on for its assembly; RustCrypto's
//! `ChaCha20Poly1305` otherwise, which builds for any target. Both compute
//! the same function, so a node using one interoperates with a node using
//! the other. The choice is fixed per build, so calls through [`AeadKey`]
//! compile to direct calls into the chosen backend.

use super::TAG_SIZE;

/// Size of a ChaCha20-Poly1305 key.
pub const KEY_SIZE: usize = 32;

/// Size of a ChaCha20-Poly1305 nonce.
pub const NONCE_SIZE: usize = 12;

/// An AEAD operation failed: the tag did not verify, or the buffer was too
/// short to hold one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AeadError;

/// The nonce for message number `counter`: four zero bytes, then the counter
/// as a little-endian `u64`, as Noise and WireGuard lay it out. Part of the
/// wire format.
#[inline]
pub fn nonce(counter: u64) -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    nonce[4..].copy_from_slice(&counter.to_le_bytes());
    nonce
}

/// ChaCha20-Poly1305 under one key, expanded once and reused.
///
/// A value is a keyed cipher. It is cloned for each worker that seals or
/// opens off the session's own task, so it is `Clone` and `Send`.
pub trait Aead: Clone + Send + Sync + Sized + 'static {
    /// Key a cipher. Returns `None` only if the backend rejects the key,
    /// which neither backend does for a 32-byte key.
    fn with_key(key: &[u8; KEY_SIZE]) -> Option<Self>;

    /// Encrypt `in_out` in place and return the tag separately; the caller
    /// places it after the ciphertext.
    fn seal(
        &self,
        nonce: &[u8; NONCE_SIZE],
        aad: &[u8],
        in_out: &mut [u8],
    ) -> Result<[u8; TAG_SIZE], AeadError>;

    /// Decrypt `in_out` in place, where it holds the ciphertext followed by
    /// its tag, and return the plaintext's length. On failure the buffer's
    /// contents are unspecified.
    fn open(
        &self,
        nonce: &[u8; NONCE_SIZE],
        aad: &[u8],
        in_out: &mut [u8],
    ) -> Result<usize, AeadError>;
}

/// The backend this build uses.
#[cfg(feature = "ring")]
pub type AeadKey = ring::aead::LessSafeKey;

/// The backend this build uses.
#[cfg(not(feature = "ring"))]
pub type AeadKey = chacha20poly1305::ChaCha20Poly1305;

#[cfg(feature = "ring")]
impl Aead for ring::aead::LessSafeKey {
    #[inline]
    fn with_key(key: &[u8; KEY_SIZE]) -> Option<Self> {
        ring::aead::UnboundKey::new(&ring::aead::CHACHA20_POLY1305, key)
            .ok()
            .map(ring::aead::LessSafeKey::new)
    }

    #[inline]
    fn seal(
        &self,
        nonce: &[u8; NONCE_SIZE],
        aad: &[u8],
        in_out: &mut [u8],
    ) -> Result<[u8; TAG_SIZE], AeadError> {
        let tag = self
            .seal_in_place_separate_tag(
                ring::aead::Nonce::assume_unique_for_key(*nonce),
                ring::aead::Aad::from(aad),
                in_out,
            )
            .map_err(|_| AeadError)?;
        let mut out = [0u8; TAG_SIZE];
        out.copy_from_slice(tag.as_ref());
        Ok(out)
    }

    #[inline]
    fn open(
        &self,
        nonce: &[u8; NONCE_SIZE],
        aad: &[u8],
        in_out: &mut [u8],
    ) -> Result<usize, AeadError> {
        ring::aead::LessSafeKey::open_in_place(
            self,
            ring::aead::Nonce::assume_unique_for_key(*nonce),
            ring::aead::Aad::from(aad),
            in_out,
        )
        .map(|plaintext| plaintext.len())
        .map_err(|_| AeadError)
    }
}

impl Aead for chacha20poly1305::ChaCha20Poly1305 {
    #[inline]
    fn with_key(key: &[u8; KEY_SIZE]) -> Option<Self> {
        use chacha20poly1305::KeyInit;
        Some(<Self as KeyInit>::new(key.into()))
    }

    #[inline]
    fn seal(
        &self,
        nonce: &[u8; NONCE_SIZE],
        aad: &[u8],
        in_out: &mut [u8],
    ) -> Result<[u8; TAG_SIZE], AeadError> {
        use chacha20poly1305::AeadInPlace;
        self.encrypt_in_place_detached(nonce.into(), aad, in_out)
            .map(Into::into)
            .map_err(|_| AeadError)
    }

    #[inline]
    fn open(
        &self,
        nonce: &[u8; NONCE_SIZE],
        aad: &[u8],
        in_out: &mut [u8],
    ) -> Result<usize, AeadError> {
        use chacha20poly1305::AeadInPlace;
        let len = in_out.len().checked_sub(TAG_SIZE).ok_or(AeadError)?;
        let (ciphertext, tag) = in_out.split_at_mut(len);
        let tag: &[u8; TAG_SIZE] = (&*tag).try_into().map_err(|_| AeadError)?;
        self.decrypt_in_place_detached(nonce.into(), aad, ciphertext, tag.into())
            .map(|()| len)
            .map_err(|_| AeadError)
    }
}

#[cfg(test)]
#[path = "tests/aead.rs"]
mod tests;
