# Benchmark Results

This document contains published Marrow benchmark results.

No reproducible performance results have been published yet.

## Current Status

| Benchmark | Status | Result |
| --- | --- | --- |
| X25519 key agreement | Not benchmarked | — |
| ML-KEM-768 key generation | Not benchmarked | — |
| ML-KEM-768 encapsulation | Not benchmarked | — |
| ML-KEM-768 decapsulation | Not benchmarked | — |
| Ed25519 signing | Not benchmarked | — |
| Ed25519 verification | Not benchmarked | — |
| XChaCha20-Poly1305 encryption | Not benchmarked | — |
| XChaCha20-Poly1305 decryption | Not benchmarked | — |
| Double Ratchet processing | Not benchmarked | — |
| Hybrid handshake | Not benchmarked | — |
| Encrypted storage write | Not benchmarked | — |
| Encrypted storage read | Not benchmarked | — |
| Tantivy indexing | Not benchmarked | — |
| Tantivy search | Not benchmarked | — |
| Message delivery latency | Not benchmarked | — |
| Relay throughput | Not benchmarked | — |
| Relay memory per peer | Not benchmarked | — |

## Publication Policy

A benchmark result should only be added after the measurement can be reproduced
using the methodology described in [`methodology.md`](methodology.md).

Each published result must identify:

- repository commit;
- benchmark implementation;
- hardware;
- operating system;
- Rust/compiler version;
- build configuration;
- benchmark parameters;
- sample size;
- measurement methodology.

## Future Results

When benchmark infrastructure is implemented, results will be grouped by:

- cryptographic primitives;
- session establishment;
- Double Ratchet processing;
- encrypted storage;
- local search;
- network transport;
- relay infrastructure;
- complete end-to-end message delivery.

Historical results should remain associated with their original commit and
benchmark environment so that performance regressions can be tracked over time.