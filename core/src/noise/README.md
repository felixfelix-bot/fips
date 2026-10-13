# core/src/noise

The fixed sizes of Noise handshake messages and their parts. The
link-layer wire codec in `proto/fmp/wire.rs` checks frame lengths
against them.

The handshake itself, the sessions, the replay window and the
encryption stay in the root crate's `noise` module for now, which
re-exports these constants, so `fips::noise::TAG_SIZE` and the rest are
unchanged.

## Files

- `mod.rs`: seven `usize` constants. `TAG_SIZE` (the AEAD tag),
  `PUBKEY_SIZE` (a compressed secp256k1 key) and `EPOCH_SIZE` (the
  startup epoch) are the parts; `EPOCH_ENCRYPTED_SIZE` and the three XX
  handshake message sizes, `HANDSHAKE_MSG1_SIZE`, `HANDSHAKE_MSG2_SIZE`
  and `HANDSHAKE_MSG3_SIZE`, are built from them.

## Invariants

- **The sizes are wire format.** Every node must agree on them. This
  line uses the XX sizes; `maint` and `master` use the IK sizes under the
  same names, and a merge into this line keeps the XX values.
- **Only sizes are here.** No cryptography, keys or session state.
