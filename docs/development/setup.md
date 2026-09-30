# Development Setup

## Scope

This document describes the local development environment for Marrow.

Marrow is a Rust + Tauri v2 desktop application with a Preact/TypeScript/Vite frontend.

## Requirements

The documented development stack is:

- Rust toolchain
- Cargo
- Node.js 20+
- pnpm
- Tauri v2 prerequisites for the host operating system
- Git

The repository's supported Rust toolchain should be pinned and documented separately once the project adopts a release-level toolchain policy.

## Repository structure

The project is organized as a Cargo workspace plus the desktop application:

```text
Marrow/
├── apps/
│   └── Marrow/
│       ├── src/
│       └── src-tauri/
├── crates/
│   ├── crypto/
│   ├── network/
│   ├── protocol/
│   └── storage/
├── docs/
└── Cargo.toml
```

Use the actual repository structure as the source of truth if modules are moved.

## Frontend dependencies

From the desktop application directory:

```powershell
cd apps/Marrow
pnpm install
```

The frontend uses Preact, TypeScript and Vite.

## Development build

```powershell
cd apps/Marrow
pnpm tauri dev
```

This starts the Tauri development application and the frontend development workflow.

## Production build

```powershell
cd apps/Marrow
pnpm tauri build
```

Release builds must be treated as security-sensitive artifacts and should be verified in CI before distribution.

## Rust checks

From the repository root:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

A clean development environment should be able to reproduce the same checks used by CI.

## Frontend checks

The frontend should have explicit CI coverage for:

```powershell
cd apps/Marrow

pnpm exec tsc --noEmit
pnpm build
```

These checks are required because Rust-only CI does not validate the TypeScript/Vite application.

## Development principles

### Do not treat warnings as harmless

Warnings in cryptographic, protocol, storage or networking code can represent correctness or security issues.

### Keep protocol changes explicit

Changes to:

- serialized structs;
- frame types;
- cryptographic suites;
- message headers;
- version fields;
- error identifiers

must be accompanied by protocol documentation and compatibility considerations.

### Keep security boundaries visible

Avoid hiding cryptographic or network state transitions behind convenience APIs when doing so makes ownership, lifetime or error behavior unclear.

## Local configuration

Development configuration should be separate from production configuration.

Do not copy development TLS certificates, debug settings or permissive resource limits into production deployments.

## Secrets

Never commit:

- private keys;
- mnemonics;
- session secrets;
- relay private keys;
- local database credentials;
- test credentials intended for production;
- generated release signing material.

Use dedicated test identities for local development.

## Recommended workflow

```text
create branch
    ↓
make focused change
    ↓
run Rust formatting/lint/tests
    ↓
run frontend typecheck/build
    ↓
run relevant integration/security tests
    ↓
update documentation
    ↓
review diff
    ↓
commit
```

## Clean-room verification

Before considering a change complete, verify that it works without relying on stale generated artifacts or locally cached application state.

For protocol and storage changes, test with a fresh temporary data directory.

## Current limitations

The repository is still under active development. Complete application-level Alice-to-Bob E2E coverage, full frontend CI, release verification and several security hardening checks remain roadmap work.
