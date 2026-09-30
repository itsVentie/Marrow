# Protocol Errors

## Principles

Protocol errors should be deterministic, version-aware, safe to expose remotely, useful for local diagnostics and free of secret material.

Remote peers must not receive raw cryptographic exceptions or stack traces.

## Categories

### Version

- `unsupported_version`
- `unsupported_feature`
- `invalid_version_field`

### Serialization

- `malformed_frame`
- `invalid_encoding`
- `unexpected_eof`
- `oversized_frame`

### Handshake

- `invalid_handshake`
- `authentication_failed`
- `invalid_key_material`
- `unsupported_suite`
- `handshake_timeout`
- `handshake_replay`
- `handshake_state_conflict`

### Session

- `session_not_found`
- `invalid_ratchet_state`
- `ratchet_header_invalid`
- `message_key_unavailable`
- `message_authentication_failed`
- `replay_rejected`

### Routing

- `recipient_not_found`
- `invalid_recipient`
- `route_unavailable`
- `relay_queue_full`

### Local state

- `storage_error`
- `storage_corruption`
- `identity_locked`
- `identity_invalid`
- `key_lifecycle_error`

## Authentication failure

Remote errors should be coarse, for example:

```text
authentication_failed
```

Local diagnostics may be more detailed only when they contain no secret material.

## Oversized input

Reject oversized frames before unbounded allocation.

The network codec limit and protocol maximum must be consistent.

## State errors

Do not automatically reset cryptographic state after malformed or unauthenticated input. Recovery must be an explicit operation.

## Wire stability

If errors are serialized on the wire, their identifiers become part of the protocol. Human-readable strings should not be treated as stable protocol identifiers.

## Logging

Never log root keys, message keys, private keys, mnemonics or plaintext.
