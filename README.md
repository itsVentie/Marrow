# Marrow

<img width="100%" height="100%" alt="photo_2026-07-31_13-27-56" src="https://github.com/user-attachments/assets/eef7f428-8e28-4b4c-a4c6-4da80a4d7523" />

> [!WARNING]
> **This project is under active development.**
> The current version may contain bugs, incomplete or non-functional features, and breaking changes without prior notice. Use at your own risk.

> High-performance, zero-trust, post-quantum resilient desktop communication suite built with Rust, Tauri v2, and Preact.

---

## Security & Architectural Principles

Marrow follows a zero-trust security model centered around local
cryptographic identities and end-to-end encrypted communication.

### 1. Identity & Cryptography

- **Local Identity:** Ed25519-based identities are generated locally and
  do not require email addresses, phone numbers, or a centralized account
  provider.

- **Hybrid Key Exchange:** Session establishment uses X25519 together with
  ML-KEM-768 to provide hybrid classical/post-quantum key agreement.

- **Forward Secrecy:** One-to-one sessions use a Double Ratchet design to
  continuously evolve message encryption keys.

- **Authenticated Encryption:** Application payloads are protected using
  modern AEAD primitives.

- **Key Hygiene:** Sensitive cryptographic material is explicitly
  zeroized where supported. OS-level memory locking is planned and is
  not currently assumed as a universal security guarantee.

### 2. Networking

- **Peer-to-Peer Networking:** The client uses a libp2p-based networking
  stack with encrypted transports and NAT traversal components.

- **Relay Infrastructure:** Marrow includes an experimental in-memory
  relay for peer routing and bounded offline delivery.

- **Traffic Metadata:** Fixed-size padding and timing randomization are
  being developed to reduce metadata leakage. These mechanisms do not
  currently provide complete protection against traffic analysis.

### 3. Local Storage

- **Encrypted Persistence:** Persistent application records are encrypted
  before being stored in the local database.

- **Local Search:** Tantivy provides local full-text search. Protecting
  search indexes at rest is an active security-hardening task.

- **Key Lifecycle:** Storage keys and session state are intended to be
  destroyed when the application is securely locked or logged out.

### 4. Threat Model

Marrow is designed to reduce exposure to:

- passive network interception;
- compromised relay infrastructure;
- local database disclosure;
- future cryptanalytic attacks against classical key exchange;
- unauthorized modification of network messages.

Marrow does **not yet claim protection against**:

- a compromised endpoint;
- a malicious operating system;
- a fully compromised client runtime;
- traffic-analysis attacks;
- a malicious or compromised identity authority;
- all metadata leakage;
- physical compromise of an unlocked device.

---

## Tech Stack

| Layer | Technology | Status |
| --- | --- | --- |
| Frontend | Preact + TypeScript + Vite | Active |
| Desktop Runtime | Tauri v2 | Active |
| Core Engine | Rust | Active |
| P2P Networking | libp2p | Active |
| QUIC Transport | libp2p QUIC | Active |
| Relay Server | Quinn | Experimental |
| Transport Security | libp2p Noise / QUIC TLS | Active |
| Serialization | Serde + Bincode | Active |
| Local Storage | redb + XChaCha20-Poly1305 | Active |
| Local Search | Tantivy | Active, at-rest encryption pending |
| Identity | Ed25519 | Active |
| Key Exchange | X25519 + ML-KEM-768 | Active |
| Session Security | Double Ratchet | Active, integration hardening pending |
| Audio/Video | WebRTC + Opus | Planned |

---

## Roadmap

> Roadmap status reflects the actual implementation state of the repository.
> Features are marked as completed only when they are implemented and
> sufficiently tested. Experimental and research features are explicitly
> separated from the stable 1-to-1 messaging core.

### Legend

- [x] Implemented and verified
- [~] Partially implemented / requires hardening or integration work
- [ ] Planned
- [!] Blocked by a prerequisite

---

<details>
<summary><b>Phase 0: Project Foundation & Engineering Infrastructure</b></summary>

- [x] Cargo workspace and modular crate architecture.
- [x] Rust core engine and Tauri v2 desktop shell.
- [x] Preact + TypeScript + Vite frontend.
- [x] Basic Rust ↔ Tauri IPC layer.
- [x] Versioned Git hooks.
- [x] Workspace formatting and linting configuration.
- [x] Automated Rust test workflow.
- [~] Fix all current `cargo fmt --check` failures.
- [ ] Add frontend dependency installation to CI.
- [ ] Add frontend type checking to CI.
- [ ] Add frontend production build to CI.
- [ ] Add release artifact verification.
- [ ] Add dependency auditing (`cargo audit`, `cargo deny`).
- [ ] Add SBOM generation for release artifacts.
- [ ] Pin and document supported toolchain versions.
- [ ] Define semantic versioning and protocol compatibility policy.

</details>

<details>
<summary><b>Phase 1: Identity, Key Management & Local Cryptographic Foundation</b></summary>

- [x] Ed25519 identity generation.
- [x] Deterministic identity derivation from BIP-39 seed material.
- [x] Argon2id-based local identity protection.
- [x] X25519 key agreement primitives.
- [x] ML-KEM-768 key encapsulation primitives.
- [x] XChaCha20-Poly1305 authenticated encryption.
- [x] Cryptographic key zeroization in implemented sensitive structures.
- [~] Complete zeroization audit across all crates.
- [ ] Define explicit key hierarchy for identity, storage, sessions,
      and application data.
- [ ] Add key-version identifiers to persistent cryptographic material.
- [ ] Implement secure key rotation.
- [ ] Implement identity backup/export using BIP-39.
- [ ] Implement identity restore/import using BIP-39.
- [ ] Add backup integrity verification and checksum validation.
- [ ] Add secure identity deletion.
- [ ] Add password/passphrase change with safe key migration.
- [ ] Document identity recovery and compromise semantics.

</details>

<details>
<summary><b>Phase 2: Authenticated Hybrid Session Establishment</b></summary>

- [x] X25519 + ML-KEM-768 hybrid key exchange primitives.
- [x] Hybrid handshake state structures.
- [x] Initial Double Ratchet state construction.
- [~] Complete initiator/responder integration.
- [ ] Fix responder ephemeral X25519 keypair mismatch between handshake
      and Double Ratchet initialization.
- [ ] Authenticate the handshake using Ed25519 identity signatures.
- [ ] Bind the authenticated identity to the ephemeral X25519 key.
- [ ] Bind ML-KEM material to the authenticated transcript.
- [ ] Define canonical transcript serialization.
- [ ] Include protocol version and cryptographic suite identifiers
      in the transcript.
- [ ] Derive session secrets from the complete authenticated transcript.
- [ ] Bind the application identity to the transport identity / PeerId.
- [ ] Reject malformed, replayed, or inconsistent handshake states.
- [ ] Add explicit handshake timeout and cancellation handling.
- [ ] Add handshake state cleanup after failure.
- [ ] Add deterministic handshake test vectors.
- [ ] Add negative tests for MITM, key substitution, replay,
      malformed ciphertext, and transcript modification.

</details>

<details>
<summary><b>Phase 3: Reliable 1-to-1 End-to-End Messaging</b></summary>

- [x] Double Ratchet root/send/receive chain implementation.
- [x] DH ratchet implementation.
- [x] Message counters and previous-chain-length handling.
- [x] Skipped-message key storage.
- [x] Maximum skipped-message limits.
- [x] AEAD-protected message payloads.
- [~] Complete application-level session integration.
- [ ] Pass a complete Alice → Bob encrypted message.
- [ ] Pass a complete Bob → Alice encrypted response.
- [ ] Support out-of-order message delivery.
- [ ] Support delayed messages using skipped keys.
- [ ] Support dropped packets and retransmission.
- [ ] Support concurrent message sends.
- [ ] Queue messages while a handshake is in progress.
- [ ] Securely persist ratchet state.
- [ ] Recover ratchet state after application restart.
- [ ] Detect and reject stale/replayed messages.
- [ ] Add message-level authentication failure handling.
- [ ] Remove duplicated ratchet header fields from the wire protocol.
- [ ] Define a single canonical encrypted-message representation.
- [ ] Add protocol versioning to encrypted message frames.

</details>

<details>
<summary><b>Phase 4: Secure Local Persistence & Recovery</b></summary>

- [x] Embedded `redb` storage.
- [x] Encrypted persistent records using XChaCha20-Poly1305.
- [x] Contact persistence.
- [x] Session persistence.
- [x] Message persistence.
- [x] Tantivy-based local search integration.
- [~] Protect search index contents at rest.
- [ ] Encrypt or otherwise protect Tantivy index files.
- [ ] Implement persistent search writer/worker instead of
      opening and committing a writer for every message.
- [ ] Define storage encryption key lifecycle.
- [ ] Zeroize storage keys on logout.
- [ ] Clear storage keys from application state after lock.
- [ ] Securely destroy in-memory session and ratchet state on logout.
- [ ] Implement crash-safe storage recovery.
- [ ] Implement schema/version migrations.
- [ ] Implement encrypted database backup/restore.
- [ ] Test corrupted database recovery.
- [ ] Test interrupted writes and process crashes.
- [ ] Document what metadata remains observable locally.

</details>

<details>
<summary><b>Phase 5: Network Transport & Blind Relay Hardening</b></summary>

- [x] libp2p-based networking layer.
- [x] TCP transport.
- [x] libp2p QUIC transport.
- [x] Noise-secured libp2p transport.
- [x] Yamux stream multiplexing.
- [x] Kademlia integration.
- [x] Identify integration.
- [x] Ping integration.
- [x] AutoNAT integration.
- [x] DCUtR integration.
- [x] Relay client components.
- [x] Quinn-based relay server prototype.
- [x] In-memory active peer routing.
- [x] Bounded offline message queues.
- [x] TTL-based queue eviction.
- [~] Unify the libp2p client relay architecture with the Quinn relay.
- [ ] Define a single production transport architecture.
- [ ] Implement authenticated relay registration.
- [ ] Add relay connection lifecycle identifiers.
- [ ] Fix duplicate-connection cleanup races.
- [ ] Add relay rate limiting.
- [ ] Add per-peer memory limits.
- [ ] Add global queue limits.
- [ ] Add connection limits.
- [ ] Add backpressure.
- [ ] Add relay abuse / flooding protection.
- [ ] Add persistent production TLS certificates.
- [ ] Add proper relay certificate validation.
- [ ] Add secure relay deployment configuration.
- [ ] Add recipient routing information to initial handshake frames.
- [ ] Define an outer routing envelope independent from E2EE payloads.
- [ ] Align network codec and protocol frame size limits.
- [ ] Add malformed-frame and oversized-frame tests.
- [ ] Add network partition and reconnection tests.

</details>

<details>
<summary><b>Phase 6: Traffic Analysis Resistance</b></summary>

- [x] Fixed-size message padding primitives.
- [x] Binary frame padding.
- [~] Timing jitter.
- [ ] Implement a real dummy-traffic scheduler.
- [ ] Implement configurable cover traffic policies.
- [ ] Define traffic generation limits and battery/CPU constraints.
- [ ] Evaluate metadata leakage under realistic traffic analysis.
- [ ] Benchmark padding overhead.
- [ ] Document exactly which metadata remains visible to relays.
- [ ] Avoid claiming protection against traffic analysis that has not
      been empirically evaluated.

</details>

<details>
<summary><b>Phase 7: Desktop Client Completion</b></summary>

- [x] Basic dashboard.
- [x] Contact management.
- [x] Conversation state management.
- [x] Basic chat screen.
- [x] Tauri event-based message delivery.
- [x] Logout flow.
- [~] Complete chat component hierarchy.
- [ ] Implement message list virtualization.
- [ ] Implement message composer.
- [ ] Implement chat header.
- [ ] Implement conversation list/sidebar.
- [ ] Implement profile UI.
- [ ] Implement settings UI.
- [ ] Implement connection status UI.
- [ ] Implement handshake/session status UI.
- [ ] Implement backup/restore UI.
- [ ] Implement identity management UI.
- [ ] Implement secure lock/unlock state.
- [ ] Add application auto-lock.
- [ ] Add secure clipboard handling.
- [ ] Add accessibility and keyboard navigation.
- [ ] Add localization infrastructure.
- [ ] Add desktop notifications without leaking message contents.
- [ ] Add OS-specific secure storage integrations where appropriate.

</details>

<details>
<summary><b>Phase 8: Security Hardening & Verification</b></summary>

- [ ] Build a complete in-process Alice/Bob end-to-end test.
- [ ] Test identity creation → handshake → encryption → delivery →
      decryption → storage → restart → continued messaging.
- [ ] Test out-of-order messages.
- [ ] Test duplicate messages.
- [ ] Test replay attacks.
- [ ] Test malformed protocol frames.
- [ ] Test invalid cryptographic keys.
- [ ] Test failed authentication.
- [ ] Test network interruption and recovery.
- [ ] Test relay restart.
- [ ] Test client restart.
- [ ] Test storage corruption.
- [ ] Add `cargo-fuzz` targets for protocol parsing.
- [ ] Add fuzzing for handshake state transitions.
- [ ] Add fuzzing for Double Ratchet state transitions.
- [ ] Add fuzzing for storage deserialization.
- [ ] Run Miri where applicable.
- [ ] Run AddressSanitizer / relevant sanitizers where applicable.
- [ ] Run dependency vulnerability audits.
- [ ] Perform cryptographic API misuse audit.
- [ ] Perform Tauri IPC security audit.
- [ ] Define and enforce a strict Content Security Policy.
- [ ] Minimize Tauri capabilities and filesystem access.
- [ ] Audit all filesystem paths exposed through IPC.
- [ ] Add rate limiting to expensive cryptographic operations.
- [ ] Add resource exhaustion tests.
- [ ] Publish reproducible security test results.
- [ ] Perform an independent cryptographic/security review before
      production security claims.

</details>

<details>
<summary><b>Phase 9: Memory Protection & Local Privacy</b></summary>

- [ ] Implement OS memory locking (`mlock` / `VirtualLock`) where supported.
- [ ] Define portable fallback behavior when memory locking is unavailable.
- [ ] Complete zeroization audit for ratchet keys and skipped keys.
- [ ] Zeroize storage encryption keys on lock/logout.
- [ ] Minimize plaintext lifetime in memory.
- [ ] Implement secure application lock.
- [ ] Implement panic mode.
- [ ] Define secure local data destruction semantics.
- [ ] Add optional encrypted temporary-file handling.
- [ ] Audit crash dumps and logging for sensitive data.
- [ ] Ensure production logging never contains plaintext message contents
      or cryptographic secrets.

</details>

<details>
<summary><b>Phase 10: Group Messaging & Multi-Party Sessions</b></summary>

- [ ] Evaluate IETF Messaging Layer Security (MLS).
- [ ] Design group identity and membership model.
- [ ] Implement group session state machine.
- [ ] Implement authenticated group membership changes.
- [ ] Implement member addition/removal.
- [ ] Implement group key rotation.
- [ ] Implement group state persistence.
- [ ] Implement group state recovery after restart.
- [ ] Design blind group relay routing.
- [ ] Define group metadata minimization.
- [ ] Add group replay and state-conflict handling.
- [ ] Add group synchronization and recovery.
- [ ] Add comprehensive multi-party integration tests.

> Group messaging will not be considered production-ready until the
> 1-to-1 authenticated session and ratchet implementation is stable.

</details>

<details>
<summary><b>Phase 11: Voice, Video & Real-Time Media</b></summary>

- [ ] Design E2EE media session architecture.
- [ ] Implement encrypted signaling over the Marrow protocol.
- [ ] Implement WebRTC connectivity.
- [ ] Implement DTLS-SRTP media transport.
- [ ] Integrate Opus audio.
- [ ] Implement adaptive bitrate control.
- [ ] Implement voice activity detection.
- [ ] Implement echo cancellation / noise suppression where supported.
- [ ] Implement call lifecycle state machine.
- [ ] Implement call recovery after network changes.
- [ ] Add media permission handling.
- [ ] Add call UI and background call state.
- [ ] Evaluate post-quantum protection for media session establishment.
- [ ] Benchmark CPU, latency, bandwidth, and battery consumption.

</details>

<details>
<summary><b>Phase 12: Anonymity & Advanced Privacy</b></summary>

- [ ] Define Marrow's anonymity threat model.
- [ ] Document what IP and metadata are visible to peers and relays.
- [ ] Implement proxy support.
- [ ] Implement SOCKS5 support.
- [ ] Evaluate Tor integration.
- [ ] Evaluate multi-hop relay routing.
- [ ] Implement onion-style routing only after the threat model
      and routing protocol are formally specified.
- [ ] Evaluate traffic-analysis resistance experimentally.
- [ ] Add relay rotation policies.
- [ ] Add relay trust minimization.
- [ ] Add privacy-preserving relay discovery.
- [ ] Document limitations of anonymity guarantees.

</details>

<details>
<summary><b>Phase 13: Cryptographic Agility & Hardware Security</b></summary>

- [ ] Define versioned cryptographic suite identifiers.
- [ ] Implement explicit protocol capability negotiation.
- [ ] Implement safe cryptographic suite migration.
- [ ] Add ML-DSA support where justified by the threat model.
- [ ] Evaluate hybrid Ed25519 + ML-DSA identity signatures.
- [ ] Add hardware-backed key storage support.
- [ ] Evaluate FIDO2 / WebAuthn integration.
- [ ] Evaluate PKCS#11 support.
- [ ] Support optional hardware-backed identity authorization.
- [ ] Define migration procedures for compromised or deprecated keys.

</details>

<details>
<summary><b>Phase 14: Relay Infrastructure & Research</b></summary>

- [ ] Design authenticated relay discovery.
- [ ] Implement decentralized relay discovery.
- [ ] Implement cryptographically signed relay descriptors.
- [ ] Add relay health checks.
- [ ] Add relay capacity advertisement.
- [ ] Add relay load balancing.
- [ ] Add privacy-preserving relay telemetry.
- [ ] Evaluate eBPF-based observability for Linux relay deployments.
- [ ] Add optional operator metrics without collecting message contents.
- [ ] Build relay fleet deployment tooling.
- [ ] Build reproducible relay container images.
- [ ] Add automated relay integration environments.

> Research infrastructure must not become a dependency of the secure
> 1-to-1 messaging core.

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
