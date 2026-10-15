//! Noise message sizes.
//!
//! The fixed sizes of the AEAD tag, a public key, the startup epoch and the
//! IK handshake messages, which the link-layer wire codec checks frames
//! against. They are wire format. The handshake and the sessions that use
//! them stay in the root crate's `noise` module, which re-exports these.

pub mod aead;
mod cipher;
mod handshake;
mod replay;
mod session;

#[cfg(any(test, feature = "test-hooks"))]
pub use cipher::open;
pub use cipher::{
    CipherState, HandshakeProgress, HandshakeRole, MAX_MESSAGE_SIZE, NoiseError, NoisePattern,
    REPLAY_WINDOW_SIZE, XK_HANDSHAKE_MSG1_SIZE, XK_HANDSHAKE_MSG2_SIZE, XK_HANDSHAKE_MSG3_SIZE,
};
pub(crate) use cipher::{PROTOCOL_NAME_IK, PROTOCOL_NAME_XK};
#[doc(hidden)]
pub use cipher::{clear_slot, take_cleared};
pub use handshake::HandshakeState;
pub use replay::ReplayWindow;
pub use session::NoiseSession;

#[cfg(test)]
mod tests;

/// Size of the AEAD tag.
pub const TAG_SIZE: usize = 16;

/// Size of a public key (compressed secp256k1).
pub const PUBKEY_SIZE: usize = 33;

/// Size of the startup epoch (random bytes for restart detection).
pub const EPOCH_SIZE: usize = 8;

/// Size of encrypted epoch (epoch + AEAD tag).
pub const EPOCH_ENCRYPTED_SIZE: usize = EPOCH_SIZE + TAG_SIZE;

/// Size of IK handshake message 1: ephemeral (33) + encrypted static (33 + 16 tag) + encrypted epoch (8 + 16 tag).
pub const HANDSHAKE_MSG1_SIZE: usize = PUBKEY_SIZE + PUBKEY_SIZE + TAG_SIZE + EPOCH_ENCRYPTED_SIZE;

/// Size of IK handshake message 2: ephemeral (33) + encrypted epoch (8 + 16 tag).
pub const HANDSHAKE_MSG2_SIZE: usize = PUBKEY_SIZE + EPOCH_ENCRYPTED_SIZE;
