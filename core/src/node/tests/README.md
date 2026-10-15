# core/src/node/tests

Unit tests for the files one level up. They run on `testutil::TestClock`
instants, so no test reads a clock or sleeps.

- `rate_limit.rs`: tests for `rate_limit.rs`, attached from there with
  `#[cfg(test)] #[path = "tests/rate_limit.rs"] mod tests;` (names
  `node::rate_limit::tests::...`). They cover the token bucket's drain,
  refill and reset, the msg1 limiter's two buckets and its pending
  limb, the established-bucket derivation, and the setup limiter's
  per-link buckets and idle pruning.
- `rate_limit_sign.rs`: tests for the lookup signing budget in
  `rate_limit.rs`, attached as its second test module with
  `#[cfg(test)] #[path = "tests/rate_limit_sign.rs"] mod
  sign_limiter_tests;`, so they keep their names
  (`node::rate_limit::sign_limiter_tests::...`). A file of their own,
  because nesting them in `rate_limit.rs` would rename them.
- `stats_history.rs`: tests for `stats_history.rs`, attached with
  `#[cfg(test)] #[path = "tests/stats_history.rs"] mod tests;` (names
  `node::stats_history::tests::...`). They cover the rings, the three
  downsample aggregations, per-peer back-fill and NaN gaps, eviction at
  the window's edge, and NaN serializing as JSON `null`.
