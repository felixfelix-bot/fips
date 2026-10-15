# core/src/proto/lookup

Mesh lookups: how a node finds a destination's tree coordinates by
asking its tree peers, and how it forwards, answers and rate-limits the
requests it receives. The root crate's lookup handlers decode requests,
call the planners here through the `RoutingView` trait, and send what
they return.

## Files

- `mod.rs`: declares the files and re-exports their public items.
- `core.rs`: the `RoutingView` trait through which the root crate shows
  its peers and tree, the `plan_forward` and `plan_initiate` planners,
  `plan_transit_forward` (which runs `plan_forward` through the forward
  rate limiter and records which link peers the request went to), their
  `LookupAction` and `ForwardOutcome` results, the dedup cache capacity
  policy, and `classify_response`, which relays a transit response only
  from a peer the request went to, naming the request's target, and only
  once per proof digest (`ResponseRoute`, `Ineligibility`).
- `state.rs`: `Lookup`, which groups the recent-request dedup cache,
  the lookups in flight, the originator's backoff and the transit
  forward rate limiter; `RecentRequest`, one cached request with its
  target, the peers it was forwarded to and the proof digests of the
  responses relayed for it.
- `limits.rs`: `LookupBackoff`, the forward rate limiter (keyed on the
  link peer a request came from and its target), and the caps on the
  dedup cache, the backoff table and the forward limiter.
- `wire.rs`: `LookupRequest` and `LookupResponse` and their codecs.
- `tests/`: the unit tests; its README describes it.

## Invariants

- **Tables a remote party fills are bounded.** The dedup cache, the
  backoff table, the forward rate limiter and each cached request's
  memory of relayed responses have fixed caps
  (`MAX_RECENT_LOOKUP_REQUESTS`, `MAX_BACKOFF_ENTRIES`,
  `MAX_FORWARD_ENTRIES`, `MAX_RELAYED_DIGESTS`) that requests cannot
  raise.
- **The request and response formats are wire format** and do not
  change on the deployed lines.
