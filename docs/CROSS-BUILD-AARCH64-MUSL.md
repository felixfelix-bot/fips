# Cross-building FIPS for the MT6000 (aarch64 / musl)

Working recipe, 2026-10-20 (Rust 1.94.1 per rust-toolchain.toml):

1. rust-std for the target on the PINNED toolchain:
   rustup target add aarch64-unknown-linux-musl --toolchain 1.94.1
2. C cross-toolchain for ring (cached at ~/musl-cross.tgz; upstream musl.cc):
   tar xzf ~/musl-cross.tgz -C /tmp/muslcc/
3. Build with the cross linker EXPLICIT:
   export PATH=/tmp/muslcc/aarch64-linux-musl-cross/bin:$PATH
   export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-musl-gcc
   export CC=aarch64-linux-musl-gcc AR=aarch64-linux-musl-ar
   cargo build --release --target aarch64-unknown-linux-musl

Pitfalls (each cost a build cycle):
- rust-toolchain.toml pins 1.94.1; plain `rustup target add` installs into the
  DEFAULT toolchain and the build still cannot find core.
- ring needs a real C cross-compiler; without CC its build script dies.
- Without CARGO_TARGET_..._LINKER, cargo calls host x86 /usr/bin/ld -> EM: 183.

Output verified: ELF 64-bit LSB, ARM aarch64, statically linked, ~21 MB.
