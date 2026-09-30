# Tauri IPC Bridge

## Purpose

The Tauri IPC layer connects the Preact frontend with the Rust application core.

The IPC boundary is security-sensitive because frontend-controlled input can reach filesystem, identity, storage and networking operations.

## Architectural model

```text
Preact / TypeScript
        │
        │ Tauri IPC
        ▼
Rust application state
        │
        ├── identity
        ├── network
        ├── storage
        └── cryptographic/session state
```

The frontend should not directly own cryptographic secrets or networking primitives that belong to the Rust core.

## IPC design principles

### Explicit commands

Each IPC command should have:

- a narrow purpose;
- typed input;
- typed output;
- documented failure behavior;
- explicit authorization assumptions.

Avoid generic commands that expose arbitrary filesystem or process functionality.

### Validate at the Rust boundary

Frontend validation is not a security boundary.

Every security-sensitive IPC argument must be validated in Rust.

Examples include:

- filesystem paths;
- contact identifiers;
- message lengths;
- identity references;
- configuration values.

## Filesystem operations

The current application contains identity file operations that accept filesystem paths.

This creates an important trust boundary.

Production-quality handling should prefer a native file picker or an explicitly constrained path model rather than trusting arbitrary frontend-supplied paths.

For file import:

```text
frontend request
      ↓
Rust validation
      ↓
path resolution / canonicalization
      ↓
type / size / format validation
      ↓
read
      ↓
cryptographic validation
```

Do not let a frontend-controlled path become an unrestricted filesystem primitive.

## Identity unlock

Identity unlock should:

1. validate the selected identity file;
2. derive/unlock required key material;
3. verify the identity;
4. initialize required runtime state;
5. avoid exposing private keys to the frontend;
6. clear sensitive temporary material after use.

Errors should not reveal passphrase correctness details beyond what the local user needs.

## Network commands

Network lifecycle belongs to Rust.

The frontend may request:

- connect;
- disconnect;
- send;
- start/stop runtime;
- query public status.

The frontend should not construct or manipulate raw cryptographic session state.

## Event flow

Application events should use explicit, versionable event names.

Conceptually:

```text
Rust core
   │
   ├── message received
   ├── connection changed
   ├── handshake state changed
   └── error
   │
   ▼
Tauri event
   │
   ▼
Frontend state
```

Event payloads should not contain private key material, raw session secrets or unnecessary sensitive metadata.

## Error handling

IPC errors should be typed or mapped to stable application-level error categories.

Do not serialize arbitrary Rust debug strings containing internal state.

Prefer:

```text
StorageError
AuthenticationFailed
InvalidInput
NetworkUnavailable
IdentityLocked
```

over raw implementation exceptions.

## Application state

Shared Rust state should have a clearly defined ownership model.

Particular attention is required for:

- identity lifecycle;
- network runtime;
- pending handshakes;
- crypto sessions;
- storage encryption keys.

Logout must clear all relevant state rather than only updating the frontend.

## Startup consistency

Identity creation and identity unlock must converge on the same network/runtime initialization path.

A newly created identity must not enter an apparently connected application state while the Rust network command/runtime remains uninitialized.

## Capabilities and CSP

Tauri capabilities should expose only the permissions required by the application.

The project currently has a permissive CSP configuration. A security-focused desktop application should define a restrictive CSP appropriate to the actual frontend and avoid unnecessary remote content.

Capability and CSP changes should be included in security review.

## IPC review checklist

- [ ] Commands have typed inputs/outputs.
- [ ] Frontend input is validated in Rust.
- [ ] Filesystem paths are constrained.
- [ ] Secrets never cross into frontend state unnecessarily.
- [ ] IPC errors do not leak secrets.
- [ ] Logout clears backend state.
- [ ] Identity creation and unlock share runtime initialization.
- [ ] Tauri capabilities are minimal.
- [ ] CSP is restrictive.
- [ ] Security-sensitive commands have tests.
