# core/src/proto/fmp

The FIPS Mesh Protocol link layer: the lifecycle of a connection to a
direct peer, from the Noise XX handshake through rekeying to teardown,
and the framing of every frame on a link. The root crate's peer and
node handlers own the Noise sessions and sockets; they ask this module
what to do on each tick and each handshake message, and carry it out.
Handshake message bytes are opaque here.

## Files

- `mod.rs`: declares the files and re-exports their public items.
- `core.rs`: the `LifecycleView` trait through which the root crate's
  `Node` shows its connections, the `ConnAction` vocabulary, the
  snapshot types, the `poll_*` decisions (handshake timeout, msg1
  resend, rekey), the classification of an inbound msg3 and of an
  outbound msg2, the cross-connection tie-break, and the negotiation
  payload's version and profile checks.
- `state.rs`: `ConnectionState`, the bookkeeping of a connection in its
  handshake phase, and `Fmp`, the stateless lifecycle anchor `Node`
  holds.
- `silent.rs`: `SilentSessions`, the record of link sessions that ended
  without an authenticated frame, and the handshake refusal it drives,
  applied at msg3, where the XX handshake first names the initiator.
- `limits.rs`: the connection-retry backoff arithmetic.
- `wire.rs`: the link framing codec: the common prefix, the encrypted,
  msg1, msg2 and msg3 headers, the handshake message types, the
  negotiation payload, disconnect reasons and the disconnect message.
- `tests/`: the unit tests; its README describes it.

## Invariants

- **The silent-session records are an ordered map on purpose.** They are
  keyed by peer identity, which a remote party chooses. A B-tree needs
  no hash seed, so a target without the standard library's random seed
  cannot be made to collide, and its worst case stays logarithmic. The
  map is never iterated, so its order is not observable.
- **Handshake sizes come from `crate::noise`.** `wire.rs` checks msg1,
  msg2 and msg3 lengths against the constants there; it does not restate
  them.
- **The framing is wire format** and does not change on the deployed
  lines.
- **No encryption here.** Encryption, decryption and key derivation
  stay with the Noise code in the root crate.
