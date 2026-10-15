# src/peer/tests

Unit tests for the modules in `src/peer/`, kept out of the source files.

- `machine.rs`: tests for `PeerMachine` (`src/peer/machine.rs`), declared from
  that file as `#[cfg(test)] #[path = "tests/machine.rs"] mod msg2_tests;`, so
  they are a child module of `machine` and can reach its private items. They
  cover an initial (XX) dial receiving a msg2 that does not complete it: one
  that does not read, one whose negotiation payload does not decrypt, and one
  that reads under a static other than the one dialed. The machine refuses each
  with the handshake kept and the carrier untouched, so the genuine msg2 still
  completes the session.

`machine.rs`, `active.rs` and `mod.rs` also carry older inline `mod tests`
blocks. New tests go here rather than into those blocks; the inline blocks are
not converted in passing.
