# core/src/noise

The fixed sizes of Noise handshake messages and their parts, and the
AEAD that Noise seals and opens with. The link-layer wire codec in
`proto/fmp/wire.rs` checks frame lengths against the sizes.

The handshake itself, the sessions, the replay window and the cipher
state stay in the root crate's `noise` module for now, which re-exports
these constants, so `fips::noise::TAG_SIZE` and the rest are unchanged.
The root crate's XK message sizes are built from them there, and its
cipher state seals and opens through `aead`.

## Files

- `mod.rs`: six `usize` constants. `TAG_SIZE` (the AEAD tag),
  `PUBKEY_SIZE` (a compressed secp256k1 key) and `EPOCH_SIZE` (the
  startup epoch) are the parts; `EPOCH_ENCRYPTED_SIZE`,
  `HANDSHAKE_MSG1_SIZE` and `HANDSHAKE_MSG2_SIZE` are built from them.
- `aead.rs`: ChaCha20-Poly1305 (RFC 8439) behind the `Aead` trait, with
  two backends. `AeadKey` is the one this build uses: `ring`'s
  `LessSafeKey` when the crate's `ring` feature is on, RustCrypto's
  `ChaCha20Poly1305` otherwise. `nonce(counter)` builds the transport's
  nonce: four zero bytes, then the counter little-endian.
- `tests/`: the AEAD's known-answer and cross-backend tests. Its own
  README describes it.

## Choosing the AEAD backend

The backend is chosen by feature, not by target, since this crate has
no target `cfg`. The root crate's `fips-core` line enables `ring`, so
every hosted build seals and opens with `ring`'s assembly. A build of
this crate alone (`cargo build -p fips-core`, the bare-metal build,
firmware) has no `ring` and uses RustCrypto, which builds for any
target; `ring` does not build for the bare-metal targets. RustCrypto is
always a dependency, so the cross-backend tests can run in any build
that has `ring` on. When both are present `ring` wins, which keeps the
features additive.

## Invariants

- **The sizes and the nonce layout are wire format.** Every node must
  agree on them, and they do not change on the deployed lines. A new
  handshake message adds a constant rather than changing one.
- **The two backends compute the same function.** A node on one
  interoperates with a node on the other; the cross-backend tests in
  `tests/aead.rs` hold them to it.
- **The trait's methods are `with_key`, `seal` and `open`.** `ring`'s
  `LessSafeKey` has inherent `new` and `open_in_place`, and an inherent
  method wins over a trait method of the same name, so the trait must
  not reuse those names.
- **No keys or session state here.** `AeadKey` is a keyed cipher that a
  caller builds and holds; this module keeps none.
