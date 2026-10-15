# core/src/noise/tests

Unit tests for `core/src/noise`.

- `aead.rs`: the tests for `../aead.rs`, attached from it with
  `#[cfg(test)] #[path = "tests/aead.rs"] mod tests;`, so they are a
  child module of `aead` and see its private items. The RFC 8439
  section 2.8.2 vector and the rejection of altered or short input run
  on each backend; `ring_and_rustcrypto_interoperate_both_ways` seals
  with one backend and opens with the other at every length up to 2048
  bytes and at 9000, under the AAD sizes the transport uses; and the
  nonce layout is checked against a fixed value. The `ring` tests are
  behind `cfg(feature = "ring")`, so `cargo test -p fips-core`, which
  has no `ring`, runs only the RustCrypto ones.
