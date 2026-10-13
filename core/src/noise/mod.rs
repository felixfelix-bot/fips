//! Noise message sizes.
//!
//! The fixed sizes of the AEAD tag, a public key, the startup epoch and the
//! XX handshake messages, which the link-layer wire codec checks frames
//! against. They are wire format. The handshake and the sessions that use
//! them stay in the root crate's `noise` module, which re-exports these.

/// Size of the AEAD tag.
pub const TAG_SIZE: usize = 16;

/// Size of a public key (compressed secp256k1).
pub const PUBKEY_SIZE: usize = 33;

/// Size of the startup epoch (random bytes for restart detection).
pub const EPOCH_SIZE: usize = 8;

/// Size of encrypted epoch (epoch + AEAD tag).
pub const EPOCH_ENCRYPTED_SIZE: usize = EPOCH_SIZE + TAG_SIZE;

/// Handshake msg1: ephemeral only (33 bytes). No DH, no encryption.
pub const HANDSHAKE_MSG1_SIZE: usize = PUBKEY_SIZE;

/// Handshake msg2: ephemeral (33) + encrypted static (33 + 16 tag) + encrypted epoch (8 + 16 tag) = 106 bytes.
pub const HANDSHAKE_MSG2_SIZE: usize = PUBKEY_SIZE + PUBKEY_SIZE + TAG_SIZE + EPOCH_ENCRYPTED_SIZE;

/// Handshake msg3: encrypted static (33 + 16 tag) + encrypted epoch (8 + 16 tag) = 73 bytes.
pub const HANDSHAKE_MSG3_SIZE: usize = PUBKEY_SIZE + TAG_SIZE + EPOCH_ENCRYPTED_SIZE;
