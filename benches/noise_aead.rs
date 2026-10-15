//! Seal and open on the Noise transport paths.
//!
//! Three shapes, the first two at a 64-byte and a 1400-byte payload behind a
//! 16-byte header used as associated data:
//!
//! - `session/*`: `NoiseSession::encrypt_with_aad` and
//!   `decrypt_with_replay_check_and_aad`, the calls the node makes when it
//!   seals or opens on its own task.
//! - `worker/*`: what the Unix encrypt and decrypt workers do per packet: a
//!   key handed out by the session (`send_cipher_clone`) and a counter it
//!   reserves, a seal in place on the wire buffer with the tag appended; or a
//!   replay check, an open in place and an accept on a worker-owned window.
//! - `handout/send_cipher_clone`: the session's key hand-out alone, which the
//!   encrypt path calls per packet.
//!
//! Only API that exists before and after the AEAD moved into `fips-core` is
//! used, so the same file runs against both trees for comparison.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use fips::Identity;
use fips::noise::{HandshakeState, NoiseSession};
use ring::aead::{Aad, Nonce};

const HEADER: usize = 16;
const SIZES: [usize; 2] = [64, 1400];
/// Packets sealed ahead for the open benchmarks; the replay window is reset
/// each time the ring wraps.
const RING: usize = 1024;

/// A completed initiator and responder session pair.
fn session_pair() -> (NoiseSession, NoiseSession) {
    let a = Identity::generate();
    let b = Identity::generate();
    let mut i = HandshakeState::new_initiator(a.keypair());
    let mut r = HandshakeState::new_responder(b.keypair());
    i.set_local_epoch([1; 8]);
    r.set_local_epoch([2; 8]);
    let m1 = i.write_message_1().unwrap();
    r.read_message_1(&m1).unwrap();
    let m2 = r.write_message_2().unwrap();
    i.read_message_2(&m2).unwrap();
    let m3 = i.write_message_3().unwrap();
    r.read_message_3(&m3).unwrap();
    (i.into_session().unwrap(), r.into_session().unwrap())
}

fn nonce(counter: u64) -> Nonce {
    let mut n = [0u8; 12];
    n[4..].copy_from_slice(&counter.to_le_bytes());
    Nonce::assume_unique_for_key(n)
}

fn session_paths(c: &mut Criterion) {
    let mut g = c.benchmark_group("session");
    for size in SIZES {
        g.throughput(Throughput::Bytes(size as u64));
        let header = [0x5au8; HEADER];
        let payload = vec![0xa5u8; size];

        let (mut tx, _) = session_pair();
        g.bench_with_input(BenchmarkId::new("seal", size), &size, |bch, _| {
            bch.iter(|| black_box(tx.encrypt_with_aad(black_box(&payload), &header).unwrap()))
        });

        let (mut tx, mut rx) = session_pair();
        let sealed: Vec<Vec<u8>> = (0..RING)
            .map(|_| tx.encrypt_with_aad(&payload, &header).unwrap())
            .collect();
        g.bench_with_input(BenchmarkId::new("open", size), &size, |bch, _| {
            rx.reset_replay_window();
            let mut k = 0usize;
            bch.iter(|| {
                if k == RING {
                    rx.reset_replay_window();
                    k = 0;
                }
                let out = rx
                    .decrypt_with_replay_check_and_aad(&sealed[k], k as u64, &header)
                    .unwrap();
                k += 1;
                black_box(out)
            })
        });
    }
    g.finish();
}

fn worker_paths(c: &mut Criterion) {
    let mut g = c.benchmark_group("worker");
    for size in SIZES {
        g.throughput(Throughput::Bytes(size as u64));

        let (mut tx, _) = session_pair();
        let mut wire = Vec::with_capacity(HEADER + size + 16);
        g.bench_with_input(BenchmarkId::new("seal", size), &size, |bch, _| {
            bch.iter(|| {
                let key = tx.send_cipher_clone().unwrap();
                let counter = tx.take_send_counter().unwrap();
                wire.clear();
                wire.resize(HEADER + size, 0xa5);
                let (aad, body) = wire.split_at_mut(HEADER);
                let tag = key
                    .seal_in_place_separate_tag(nonce(counter), Aad::from(&*aad), body)
                    .unwrap();
                wire.extend_from_slice(tag.as_ref());
                black_box(&wire);
            })
        });

        let (tx, rx) = session_pair();
        let key = tx.send_cipher_clone().unwrap();
        let header = [0x5au8; HEADER];
        let sealed: Vec<Vec<u8>> = (0..RING as u64)
            .map(|counter| {
                let mut body = vec![0xa5u8; size];
                let tag = key
                    .seal_in_place_separate_tag(nonce(counter), Aad::from(&header), &mut body)
                    .unwrap();
                body.extend_from_slice(tag.as_ref());
                body
            })
            .collect();
        let rkey = rx.recv_cipher_clone().unwrap();
        let mut window = rx.recv_replay_snapshot_owned();
        let mut buf = Vec::with_capacity(size + 16);
        g.bench_with_input(BenchmarkId::new("open", size), &size, |bch, _| {
            window.reset();
            let mut k = 0usize;
            bch.iter(|| {
                if k == RING {
                    window.reset();
                    k = 0;
                }
                let counter = k as u64;
                assert!(window.check(counter));
                buf.clear();
                buf.extend_from_slice(&sealed[k]);
                let plain = rkey
                    .open_in_place(nonce(counter), Aad::from(&header), &mut buf)
                    .unwrap();
                black_box(plain);
                window.accept(counter);
                k += 1;
            })
        });
    }
    g.finish();

    let (tx, _) = session_pair();
    c.bench_function("handout/send_cipher_clone", |bch| {
        bch.iter(|| black_box(tx.send_cipher_clone()))
    });
}

criterion_group!(benches, session_paths, worker_paths);
criterion_main!(benches);
