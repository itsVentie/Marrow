# Marrow

<img width="100%" height="100%" alt="photo_2026-07-31_13-27-56" src="https://github.com/user-attachments/assets/eef7f428-8e28-4b4c-a4c6-4da80a4d7523" />

> [!WARNING]
> **This project is under active development.**
> The current version may contain bugs, incomplete or non-functional features, and breaking changes without prior notice. Use at your own risk.

> High-performance, zero-trust, post-quantum resilient desktop communication suite built with Rust, Tauri v2, and Preact.

---

## Security & Architectural Principles

`marrow` is designed with a strict zero-trust philosophy. It operates on isolated cryptographic identities, zero PII requirement, and a memory-safe execution engine.

### 1. Identity & Cryptography

* **Identity Management:** Ed25519 asymmetric signature scheme (`ed25519-dalek`). Accounts require no email, phone number, or centralized authority; identities are bound strictly to a local 32-byte seed (`identity.key`).
* **Post-Quantum Key Exchange:** Hybrid **X25519 + ML-KEM-768** (Kyber) key exchange to protect session initialization against "Harvest Now, Decrypt Later" quantum adversary scenarios.
* **Forward Secrecy:** Full **Double Ratchet Algorithm** implementation. Session keys mutate on every message exchange, invalidating past and future ciphertexts if a single key is compromised.
* **Authenticated Encryption:** **ChaCha20-Poly1305** AEAD for all payload encryptions, offering superior resistance against side-channel timing attacks without reliance on native AES hardware instructions.
* **Memory Protection & Zeroization:** Strict zeroization (`zeroize`) on drop for all ephemeral keys, seed states, and raw byte buffers. Critical memory regions containing identity keys and master keys are locked in RAM via OS memory pinning (`mlock` / `VirtualLock`) to prevent page swapping to disk.

### 2. Networking Layer

* **Transport & Resilience:** Native **QUIC** protocol (`quinn`) over UDP, offering 0-RTT session resumption, multi-path connection migration, and built-in TLS 1.3 encryption. Includes an automatic **TCP/TLS** (`tokio-rustls`) fallback transport layer to bypass strict corporate firewalls and aggressive UDP-blocking NAT environments.
* **Serialization:** Ultra-compact, zero-copy binary framing protocol using `postcard` for low-overhead Rust-to-Rust IPC and network payload serialization.
* **Blind Relay Architecture:** The server operates as an untrusted, stateless relay. It holds no databases, tracks zero logs, and transiently forwards binary QUIC packets by public key routing. Un-routable messages remain in volatile RAM with a short TTL before absolute eviction.
* **Traffic Obfuscation:** Active padding to fixed-size binary frames and dummy traffic generation (Poisson distribution) to mitigate Deep Packet Inspection (DPI) and metadata/size analysis.

### 3. Local Storage Architecture

* **Embedded Storage:** Zero-dependency embedded KV store (`redb`) operating with ACID guarantees and zero-copy read paths.
* **Data-At-Rest Protection:** All local database pages are encrypted via **ChaCha20-Poly1305**. Key derivation utilizes **Argon2id** with high memory cost parameters derived from user master authentication.
* **Local Search Engine:** Embedded full-text search index (`tantivy`) operating over encrypted local stores for sub-millisecond query performance without unencrypting entire message histories to RAM.

---

## Tech Stack

| Layer | Technology | Key Characteristics |
| --- | --- | --- |
| **Frontend UI** | Preact + TypeScript + Vite | ~4KB core footprint, signal-based reactivity, strict typings |
| **GUI Framework** | Tauri v2 | OS-native WebView wrapper, sandboxed IPC, low RAM overhead (~20-30MB) |
| **Core Engine** | Rust (2021 Edition) | Memory safety without Garbage Collection, explicit zero-allocation targets |
| **Networking** | Quinn (QUIC/UDP) + TCP/TLS Fallback | DPI-resistant, multiplexed transport stream with automatic fallback |
| **Serialization** | `postcard` / `serde` | Compact, no-std capable zero-copy binary framing |
| **Local Storage** | `redb` + Argon2id | Embedded, fully encrypted, single-file local persistence |
| **Local Search** | `tantivy` | High-performance embedded full-text indexing engine |
| **Media & Audio** | Opus (`opus-rs`) / WebRTC (`webrtc-rs`) | Low-latency audio encoding and direct P2P media channels |

---

## Roadmap

<details>
<summary><b>Phase 1: Core Primitives & Identity (Completed)</b></summary>

* [x] Cargo Workspace setup and modular crate design.
* [x] Tauri v2 + Preact + TypeScript frontend pipeline initialization.
* [x] Implement Ed25519 keypair generation and Argon2id local file encryption in `crates/crypto`.
* [x] Build encrypted local KV abstraction over `redb` in `crates/storage`.

</details>

<details>
<summary><b>Phase 2: Transport & Protocol Layer (Completed)</b></summary>

* [x] Implement binary framing (`postcard`) and serde layer in `crates/protocol`.
* [x] Implement QUIC client transport (`quinn`) with automated TCP/TLS fallback (`tokio-rustls`).
* [x] Implement hybrid X25519 + ML-KEM-768 post-quantum handshake (`crates/crypto` / `crates/protocol`).
* [x] Build Double Ratchet session state machine for 1-on-1 sessions.
* [x] Implement stateless relay routing layer with ephemeral token validation in `apps/relay`.
* [x] Design adaptive padding and timing jitter algorithms to obfuscate traffic size/metadata against DPI analysis.

</details>

<details>
<summary><b>Phase 3: Desktop Client & User Interface</b></summary>

* [x] Design UI in Preact using CSS Modules.
* [x] Integrate IPC invocations between Preact and Rust core via `@tauri-apps/api`.
* [ ] Implement background daemon/tray process for persistent network listening without UI overhead.
* [ ] Integrate `tantivy` for instant local encrypted message search.
* [x] Implement secure local storage for contact lists and conversation histories.
* [ ] Build key exporting/importing mechanisms with physical key backup features (BIP-39 mnemonic phrase).
* [x] Build UI state logic for direct messaging, active contacts, and peer connection statuses.

</details>

<details>
<summary><b>Phase 4: Secure Group Messaging & Public Channels</b></summary>

* [ ] Implement TreeKEM / IETF MLS (Messaging Layer Security) protocol for dynamic encrypted group chats (`crates/mls`).
* [ ] Design Blind Group Relay engine for server-side distribution of group frames without decrypting metadata.
* [ ] Implement channel architecture (Broadcast channels with asymmetric signed state updates).
* [ ] Build local group session state persistence with automatic group key rotation on member removal.

</details>

<details>
<summary><b>Phase 5: Voice, Video & E2EE Real-Time Transport</b></summary>

* [ ] Build native WebRTC / DTLS-SRTP audio engine with custom Post-Quantum key exchange extension (`webrtc-rs`).
* [ ] Implement Opus audio codec pipeline (`opus-rs`) with adaptive bitrates, noise suppression, and voice activation.
* [ ] Implement encrypted P2P signaling frame routing over `apps/relay` for NAT traversal (ICE/STUN/TURN).
* [ ] Build background peer-to-peer audio calling state machine and UI overlay.

</details>

<details>
<summary><b>Phase 6: Memory Security, Anonymity & Anti-Analysis</b></summary>

* [ ] Memory pinning via `mlock` / `VirtualLock` to lock critical cryptographic key regions in physical RAM.
* [ ] Implement Onion Routing layer over relay nodes (Multi-hop routing for full sender/receiver IP masking).
* [ ] Integrate built-in proxy support (SOCKS5 / TOR / Shadow-like obfuscated QUIC transport).
* [ ] Complete zeroization audits for sensitive memory regions (using `zeroize` crate across all crates).
* [ ] Implement Panic Mode (Instant zeroization of in-memory keys and local storage drop via master passphrase).

</details>

<details>
<summary><b>Phase 7: Hardening, Auditing & Resilience</b></summary>

* [ ] Automated integration tests for network partitions, packet drops, and state recovery.
* [ ] Stress testing `apps/relay` throughput and memory footprint under thousands of simultaneous streams.
* [ ] Static analysis, Miri checks, clippy lint enforcement, and memory safety security audit.
* [ ] Production release pipelines (Cross-platform builds for Windows, macOS, Linux).

</details>

<details>
<summary><b>Phase 8: Architecture Hardening & I/O Isolation (Planned)</b></summary>

* [ ] **Sans-I/O Architecture Refactoring:** Decouple protocol framing, state machines, and cryptographic verification from direct operating system I/O (sockets and filesystem). Implement pure state-transition functions taking byte slices (`&[u8]`) to enable deterministic mock testing without network or disk overhead.
* [ ] **Zero-Copy Serialization Audit:** Optimize all payload parsing paths using `zerocopy` / `postcard` traits, eliminating intermediate heap allocations and runtime memory fragmentation during QUIC packet processing.
* [ ] **Deterministic Fuzz Testing:** Implement structured fuzzing targets using `cargo-fuzz` for wire protocols, frame bounds checking, and state-machine transitions under malformed inputs.
* [ ] **Encrypted Storage Migration Tools:** Build automated database schema migration pipelines with backward-compatible key derivation upgrades inside `redb`.

</details>

<details>
<summary><b>Phase 9: Post-Quantum Migration & Crypto Agility</b></summary>

* [ ] **ML-DSA (Dilithium) Signature Integration:** Add support for ML-DSA-65 / ML-DSA-87 digital signatures alongside Ed25519 for identity verification and long-term public keys.
* [ ] **Crypto Agility Versioning:** Implement dynamic cryptographic suite negotiation in protocol handshake headers to enable seamless algorithm upgrades without breaking backwards compatibility.
* [ ] **Hardware Security Integration:** Build optional PKCS#11 / FIDO2 YubiKey integration for hardware-backed master key derivation and session approval.

</details>

<details>
<summary><b>Phase 10: eBPF Telemetry & Autonomous Node Fleet Administration</b></summary>

* [ ] **eBPF-Based Kernel Telemetry:** Deploy lightweight eBPF probes in `apps/relay` nodes for real-time kernel-level packet inspection, drop tracking, and anti-DDoS traffic filtering without context switches.
* [ ] **Zero-Knowledge Node Monitoring:** Implement anonymized health and performance reporting metrics from relay nodes using blinded aggregation primitives.
* [ ] **Automated Relay Peer Discovery:** Build a decentralized peer-discovery protocol over distributed hash tables (DHT) with cryptographically verified node descriptors.

</details>

---

## Testing & Quality Assurance

`marrow` features a comprehensive automated test suite across all workspace crates, verifying core cryptography, binary framing, network protocols, and ACID storage persistence, alongside strict static analysis and code formatting rules.

### Local Git Hooks Setup

To automatically run pre-commit checks (`cargo fmt`, `cargo clippy`, and `cargo test`) before every commit, enable the repository's versioned hooks:

```bash
git config core.hooksPath .githooks

```

### Code Quality & Linting

Before submitting changes, ensure your code passes all formatting and linter checks:

```bash
# Check code formatting across the entire workspace
cargo fmt --all -- --check

# Run clippy static analysis and lints
cargo clippy --workspace -- -D warnings

```

### Running Tests

Execute the full workspace test suite:

```bash
cargo test --workspace

```

---

## Building Locally

### Prerequisites

* **Rust**: `1.78.0` or newer
* **Node.js**: `v20+` & `pnpm`
* **Tauri CLI**: `v2.x`

### Quick Start

1. Install frontend dependencies:

```bash
cd apps/marrow
pnpm install

```

2. Run application in dev mode:

```bash
pnpm tauri dev

```

3. Build production release:

```bash
pnpm tauri build

```

---

## Contributing & Security

* For development guidelines, branch strategies, and PR expectations, read [CONTRIBUTING.md](https://github.com/itsVentie/Marrow/blob/main/CONTRIBUTING.md).
* To report a security vulnerability or cryptographic flaw, review our [SECURITY.md](https://github.com/itsVentie/Marrow/blob/main/SECURITY.md).

---

## License

Licensed under [GPLv3](https://github.com/itsVentie/Marrow/blob/main/LICENSE).
