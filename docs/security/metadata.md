# Metadata and Traffic Analysis

**Status:** Experimental / limitations documented

Marrow provides end-to-end cryptographic protection for message content as the design target, but cryptographic confidentiality does not imply metadata confidentiality.

## 1. Metadata categories

Potentially observable or locally retained metadata includes:

- source and destination network information;
- relay routing information;
- connection establishment and termination;
- message timing;
- packet/frame frequency;
- packet/frame size;
- queue state;
- online/offline transitions;
- local contact and conversation metadata;
- search-index metadata;
- application logs and diagnostics.

The exact visibility of each field depends on the transport and protocol version.

## 2. Relay-visible information

The relay may need to process:

- peer identifiers or routing identifiers;
- recipient routing information;
- connection state;
- frame delivery state;
- queue state;
- timestamps associated with transport activity.

The relay should not receive message plaintext or ratchet secret material.

This does not make the relay metadata-blind.

## 3. Message-size leakage

Padding can reduce direct plaintext-length leakage.

Current fixed-block padding should be treated as a mitigation rather than a proof of traffic-flow confidentiality.

Residual information can include:

- number of messages;
- approximate timing;
- directionality;
- bursts;
- session duration;
- packet-level behavior.

## 4. Timing leakage

Timing randomization can make simple timing inference harder.

It does not defeat a sufficiently capable observer that can correlate both sides of a communication channel.

Marrow therefore does not claim protection against global traffic correlation.

## 5. Dummy traffic

A `Dummy` frame type and related experimental mechanisms exist, but a complete production-grade cover-traffic scheduler is not established.

A future implementation should define:

- scheduling distribution;
- traffic budget;
- idle behavior;
- burst behavior;
- bandwidth limits;
- battery/CPU impact;
- interaction with real messages;
- observable failure modes.

A claim of Poisson or statistically modeled cover traffic should not be made until the scheduler and measurements exist.

## 6. Anonymity boundary

Marrow is not an anonymity network.

It does not currently provide:

- onion routing;
- mixnet routing;
- global-observer resistance;
- sender anonymity;
- receiver anonymity;
- unlinkability against all network observers.

Anonymity must therefore be treated as a separate future research area rather than a consequence of E2EE.

## 7. Local metadata

Even when database records are encrypted, local metadata can remain exposed through:

- filesystem characteristics;
- database/index sizes;
- file timestamps;
- process state;
- logs;
- crash artifacts;
- backups;
- search indexes;
- OS-level telemetry.

Protecting local metadata requires controls beyond database encryption.

## 8. Security documentation rule

Documentation should explicitly distinguish:

**Content confidentiality**
from
**metadata confidentiality**
from
**traffic-analysis resistance**
from
**anonymity**.

These are separate properties and should not be conflated.
