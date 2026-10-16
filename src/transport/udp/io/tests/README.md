# UDP socket I/O tests

Unit tests for files in `src/transport/udp/io/`. Each file here is the test
module of the source file of the same name one level up, declared from that
file as `#[cfg(test)] #[path = "tests/<file>.rs"] mod tests;`, so it stays a
child module of its source file and can reach that file's private items.

- `bind_device.rs`: tests for `bind_device.rs`, binding a UDP instance's
  listen socket and its per-peer connected sockets to one interface
  (`udp.interface`). On Linux they read the binding back from the kernel
  with `SO_BINDTODEVICE`; they need the `lo` interface and kernel 5.7 or
  later (or `CAP_NET_RAW` on an older kernel).

`mod.rs`, `unix.rs` and the files under `connected/` still carry inline
`mod tests` blocks. Those blocks predate the rule that unit tests live in
this directory, and this directory's tests do not move them; a test written
for one of those files once its block has moved belongs here.
