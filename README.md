# Marrow

> High-performance, zero-trust, post-quantum resilient desktop communication suite built with Rust, Tauri v2, and Preact.

---

## Security & Architectural Principles

marrow is designed with a strict zero-trust philosophy. It operates on isolated cryptographic identities, zero PII requirement, and a memory-safe execution engine.

### 1. Identity & Cryptography
* **Identity Management:** Ed25519 asymmetric signature scheme (`ed25519-dalek`). Accounts require no email, phone number, or centralized authority; identities are bound strictly to a local 32-byte seed (`identity.key`).
* **Post-Quantum Key Exchange:** Hybrid **X25519 + ML-KEM-768** (Kyber) key exchange to protect session initialization against "Harvest Now, Decrypt Later" quantum adversary scenarios.
* **Forward Secrecy:** Full **Double Ratchet Algorithm** implementation. Session keys mutate on every message exchange, invalidating past and future ciphertexts if a single key is compromised.
* **Authenticated Encryption:** **ChaCha20-Poly1305** AEAD for all payload encryptions, offering superior resistance against side-channel timing attacks without reliance on native AES hardware instructions.

### 2. Networking Layer
* **Transport:** Native **QUIC** protocol (`quinn`) over UDP, offering 0-RTT session resumption, multi-path connection migration, and built-in TLS 1.3 encryption.
* **Blind Relay Architecture:** The server operates as an untrusted, stateless relay. It holds no databases, tracks zero logs, and transiently forwards binary QUIC packets by public key routing. Un-routable messages remain in volatile RAM with a short TTL before absolute eviction.

### 3. Local Storage Architecture
* **Embedded Storage:** Zero-dependency embedded KV store (`redb`) operating with ACID guarantees and zero-copy read paths.
* **Data-At-Rest Protection:** All local database pages are encrypted via **ChaCha20-Poly1305**. Key derivation utilizes **Argon2id** with high memory cost parameters derived from user master authentication.

---

## Tech Stack

| Layer | Technology | Key Characteristics |
| :--- | :--- | :--- |
| **Frontend UI** | Preact + TypeScript + Vite | ~4KB core footprint, signal-based reactivity, strict typings |
| **GUI Framework** | Tauri v2 | OS-native WebView wrapper, sandboxed IPC, low RAM overhead (~20-30MB) |
| **Core Engine** | Rust (2021 Edition) | Memory safety without Garbage Collection, explicit zero-allocation targets |
| **Networking** | Quinn (QUIC/UDP) | DPI-resistant, multiplexed transport stream |
| **Local Storage** | `redb` + Argon2id | Embedded, fully encrypted, single-file local persistence |

---

## Roadmap

### Phase 1: Core Primitives & Identity (Current)

* [x] Cargo Workspace setup and modular crate design.
* [x] Tauri v2 + Preact + TypeScript frontend pipeline initialization.
* [x] Implement Ed25519 keypair generation and Argon2id local file encryption in `crates/crypto`.
* [x] Build encrypted local KV abstraction over `redb` in `crates/storage`.

### Phase 2: Transport & Handshake Protocol

* [ ] Implement QUIC client framing and protocol buffers in `crates/protocol`.
* [ ] Implement hybrid X25519 + ML-KEM-768 post-quantum handshake.
* [ ] Build Double Ratchet session state machine.
* [ ] Deploy prototype `apps/relay` stateless forwarding node.

### Phase 3: Desktop Client & User Interface

* [ ] Design "Dark Glassmorphism" UI in Preact using CSS Modules / SCSS.
* [ ] Integrate IPC invocations between Preact and Rust core via `@tauri-apps/api`.
* [ ] Implement secure local storage for contact lists and conversation histories.
* [ ] Build key exporting/importing mechanisms with physical key backup features.

### Phase 4: Hardening & Auditing

* [ ] Zeroization audits for sensitive memory regions (using `zeroize` crate).
* [ ] Anti-analysis & DPI obfuscation layers on the QUIC transport level.
* [ ] Automated integration tests for network partitions and state recovery.

---

## Building locally

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

## License

Licensed under GPLv3.

