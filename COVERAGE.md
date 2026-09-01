# Sv2 Protocol Coverage

This document formally describes how `interoperability-tests-sv2` covers the [Stratum V2 Protocol Specification](https://github.com/stratum-mining/sv2-spec).

§ indicates spec sections.

## Scenarios

A **Scenario** is a single testable Sv2 Protocol behavior — each describes a failure mode, its spec basis, and how a client detects it.

Scenarios are application-agnostic: each row describes a generic Sv2 Protocol behavior, which might potentially happen under different application contexts.

### How scenarios are organized

The catalog is organized **one table per spec message**, mirroring the spec's own structure. A **General** section covers cross-cutting invariants that have no single home message (framing, error codes, channel/job namespaces, liveness).

**Anchor rule:** each scenario is filed under the message whose spec section carries the normative requirement it tests. Scenarios frequently involve a longer session with other messages; the `Involves` column lists them so the scenario remains discoverable from every message it touches.

**Symmetric split**: scenarios that apply to both a standard and extended message pair are separate scenarios with separate IDs — they exercise different server code paths and are covered independently per suite.

**Scenario IDs** are `<PROTO>-<MSG>-<n>` where `PROTO` ∈ `GEN` (cross-cutting), `C` (common), `M` (mining), `JD` (job declaration), `TD` (template distribution). Extension scenarios use `X<extid>-<MSG>-<n>` (e.g. `X0001-REQS-1` for extension `0x0001`). IDs are unique, stable references: new scenarios append; existing IDs are never renumbered or reused. IDs prefixed `JD-`/`TD-` are reserved for future suite expansion.

## Suites

A **Suite** is an application context (e.g. Solo Pool, Pool, JDS, Proxy) that maps scenarios to real-world deployments. The [**Suite Coverage**](#suite-coverage) table below tracks which scenarios are implemented per suite.

## Compliance Tiers

**Compliance Tiers** classify scenarios by how severely a failure undermines Sv2 compliance.

- **Ω1 — non-compliant.** An application failing any Ω1 scenario is fundamentally broken and does not meet the Sv2 protocol specification.
- **Ω2 — interoperability-critical.** Meaningful edge cases that might hurt interoperability, but are not direct protocol violations.
- **Ω3 — robustness.** Miscellaneous non-critical corner cases.

This list might grow as the project matures.

## Testability Tiers

**Testability Tiers** classify scenarios by the type of assertion required.

- **Θ1 — Protocol conformance.** Pure message-level assertions: framing, negotiation, channel open, message ordering, invalid-share rejection, channel lifecycle.
- **Θ2 — Valid-share accounting.** The test client grinds real shares meeting the *channel* target and asserts acceptance / rejection and batch-accounting semantics.

This list might grow as the project matures.

---

## Scenario Catalog

### General ([§3.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#31-data-types-mapping)–[§3.5](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#35-error-codes), [§5.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#51-job)–[§5.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#52-channel), [§8](https://github.com/stratum-mining/sv2-spec/blob/main/08-Message-Types.md#8-message-types))

Cross-cutting invariants with no single home message.

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| GEN-1 | Ω1 | Θ1 | `channel_msg` bit wrong per message type (e.g. `NewMiningJob` 0x15 sent with bit unset, `OpenStandardMiningChannel.Success` 0x11 with bit set) | [§3.2.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#321-routing-frames-over-channels), [§8](https://github.com/stratum-mining/sv2-spec/blob/main/08-Message-Types.md#8-message-types) | Assert bit on every received frame | all messages |
| GEN-2 | Ω1 | Θ1 | Core messages sent with non-zero `extension_type` | [§3.4.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#341-extension-type-field-usage) | Assert `extension_type & 0x7FFF == 0x0000` (mask `channel_msg` bit first) | all messages |
| GEN-3 | Ω1 | Θ1 | Error codes containing control characters | [§3.5](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#35-error-codes) | Validate every `error_code` string | all `*.Error` messages |
| GEN-4 | Ω2 | Θ1 | Error codes containing non-printable non-control bytes | [§3.5](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#35-error-codes) | Validate every `error_code` string | all `*.Error` messages |
| GEN-5 | Ω1 | Θ1 | Server sends messages for a `channel_id` already closed on this connection | [§5.3.9](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#539-closechannel-client---server-server---client) "The server MUST stop sending messages for the channel" | Track closed channel set; assert silence | all channel messages; see M-CC-1 |
| GEN-6 | Ω2 | Θ1 | Server sends messages for a `channel_id` never opened on this connection | [§5.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#52-channel) (channel ID consistency) | Track open channel set; assert all inbound channel messages reference it | all channel messages |
| GEN-7 | Ω3 | Θ1 | Job starvation: mining server never sends fresh jobs after the first, stranding miners on an exhausted search space | [§5.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#51-job) (server must manage hashing space and "provide new and unique Jobs quickly enough") | Soft liveness/timing metric | `NewMiningJob`, `NewExtendedMiningJob` |
| GEN-8 | Ω2 | Θ1 | Message with unknown `extension_type` that terminates locally is not discarded and ignored (server errors, hangs up, or crashes) | [§3.4.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#341-extension-type-field-usage) "Messages with an unknown `extension_type` which are to be processed locally MUST be discarded and ignored" | Send a frame with an unallocated `extension_type`; assert the session continues unaffected | all messages |
| GEN-9 | Ω2 | Θ2 | Server chokes on unnegotiated/unknown TLV fields appended to a core message | [§3.4.3](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#343-stratum-v2-tlv-encoding-model) ("unknown fields can be safely ignored") | Append an unknown TLV to `SubmitSharesExtended`; assert the share is processed normally | `SubmitSharesExtended` |
| GEN-10 | Ω2 | Θ1 | Server-generated TLV fields violate placement/ordering rules (not at end of payload, not ordered by `extension_type`) | [§3.4.3](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#343-stratum-v2-tlv-encoding-model) | Parse TLV region on server messages; only applicable once the server negotiates a TLV extension | all TLV-extended messages |
| GEN-11 | Ω2 | Θ1 | TLV field exceeding its extension's maximum length is accepted | [§3.4.3](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#343-stratum-v2-tlv-encoding-model) "the recipient MUST reject the message" | Send over-length TLV (e.g. 33-byte `user_identity`); assert the message is rejected | `SubmitSharesExtended`, `SubmitShares.Error` |
| GEN-12 | Ω2 | Θ1 | Proxy drops or modifies an unknown-extension channel message instead of forwarding it unmodified to the channel endpoint | [§3.4.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#341-extension-type-field-usage) "MUST forward that message... MUST NOT be otherwise modified" | Proxy suites only: inject unknown-`extension_type` channel message; assert byte-identical forwarding | all channel messages |
| GEN-13 | Ω2 | Θ1 | Server sends extension-specific messages for an extension that was never negotiated | [§3.4.2](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#342-implementing-extensions-support) "Extensions MUST require negotiation... before sending" | Assert no extension-typed frames arrive without a prior `RequestExtensions.Success` | `RequestExtensions.Success` |
| GEN-14 | Ω2 | Θ1 | Server sends contradictory responses to a single request (e.g. both `.Success` and `.Error` for the same `request_id`) | Field-definition identity semantics | Assert exactly one response per `request_id` | `OpenMiningChannel.Error`, `RequestExtensions.Error`, `SetCustomMiningJob.Error` |

### Common Protocol Messages

#### `SetupConnection` ([§3.6.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#361-setupconnection-client---server), mining flags [§5.3.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#531-setupconnection-flags-for-mining-protocol))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| C-SC-1 | Ω1 | Θ1 | Server sends *any* protocol message before responding to `SetupConnection` | [§3.6.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#361-setupconnection-client---server) ("MUST be the first message") | Message-order assertion | `SetupConnection.Success`, `SetupConnection.Error` |
| C-SC-2 | Ω2 | Θ1 | Server rejects empty `device_id` or crashes on boundary device-info strings (255-byte or empty) | [§3.6.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#361-setupconnection-client---server), [§3.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#31-data-types-mapping) types | Send `SetupConnection` with empty `device_id` (explicitly allowed); assert no error. Send with 255-byte strings; assert no crash | — |
| C-SC-3 | Ω1 | Θ1 | Server sets `REQUIRES_FIXED_VERSION` in `.Success.flags` even though client set `REQUIRES_VERSION_ROLLING` | [§5.3.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#531-setupconnection-flags-for-mining-protocol) "MUST NOT be set" | Flag cross-check | `SetupConnection.Success` |
| C-SC-4 | Ω1 | Θ1 | Server rejects `REQUIRES_STANDARD_JOBS` (bit 0) — fails the most basic client class | [§5.2.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#522-extended-channel) "MUST also support Standard Channels" | Minimal-flags `SetupConnection` must succeed | `SetupConnection.Error` |
| C-SC-5 | Ω2 | Θ1 | Server sets `REQUIRES_EXTENDED_CHANNELS` but then accepts `OpenStandardMiningChannel` (or rejects it, contradicting a standard-only deployment) | [§5.3.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#531-setupconnection-flags-for-mining-protocol) bit 1 | Open standard channel; assert coherent behavior | `OpenStandardMiningChannel`, `OpenMiningChannel.Error` |
| C-SC-6 | Ω2 | Θ1 | Server sets `REQUIRES_FIXED_VERSION` but later sends `NewExtendedMiningJob` with `version_rolling_allowed = True` | [§5.3.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#531-setupconnection-flags-for-mining-protocol) | Cross-message consistency | `NewExtendedMiningJob` |
| C-SC-7 | Ω1 | Θ1 | Server accepts `SetupConnection` with an unsupported or invalid `protocol` value | [§3.6.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#361-setupconnection-client---server) | Send `SetupConnection` with `protocol=1` (JDC) or `protocol=3` (invalid); assert `SetupConnection.Error` | `SetupConnection.Error` |
| C-SC-8 | Ω1 | Θ1 | Server accepts `SetupConnection` with no supported version in the offered range (e.g. `(3,3)`, `(3,2)`, `(1,1)`) or closes without first sending `SetupConnection.Error` | [§3.6.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#361-setupconnection-client---server) | Send version ranges with no overlap with 2; assert framed `SetupConnection.Error` before TCP close | `SetupConnection.Error` |

#### `SetupConnection.Success` ([§3.6.2](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#362-setupconnectionsuccess-server---client))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| C-SCS-1 | Ω1 | Θ1 | Server fails to respond to `SetupConnection` with a properly framed `.Success` or `.Error` — silence, wrong message type, or TCP close instead of a protocol error | [§3.6.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#361-setupconnection-client---server) "MUST respond with either .Success or .Error" | Assert framed `.Success` or `.Error` within timeout; assert not bare TCP close | `SetupConnection.Error` |
| C-SCS-2 | Ω1 | Θ1 | `used_version` outside client's `[min_version, max_version]` | [§3.6.2](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#362-setupconnectionsuccess-server---client) | Send `min_version = max_version = 2`; assert `used_version == 2` | — |

#### `SetupConnection.Error` ([§3.6.3](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#363-setupconnectionerror-server---client))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| C-SCE-1 | Ω1 | Θ1 | Client sets all flags (feature probing); server errors with non-zero flags that are missing or mismatched | [§3.6.3](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#363-setupconnectionerror-server---client) "MUST provide the full set of flags which it does not support" | All-flags probe; if flags ≠ 0 assert they exactly equal the unsupported set; flags=0 means rejection for an unrelated reason | — |
| C-SCE-2 | Ω1 | Θ1 | Server reports different supported/unsupported flag sets across connections to the same hostname and port | [§3.6.3](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#363-setupconnectionerror-server---client) "MUST consistently support the same set of flags across all servers on the same hostname and port number" | Repeat identical all-flags probe across multiple fresh connections; compare the returned Error flags sets | — |

#### `ChannelEndpointChanged` ([§3.6.4](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#364-channelendpointchanged-server---client))

No scenarios yet — relevant to proxy contexts, not solo-pool deployments.

#### `Reconnect` ([§3.6.5](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#365-reconnect-server---client))

No scenarios yet.

### Mining Protocol Messages

#### `OpenStandardMiningChannel` ([§5.3.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#532-openstandardminingchannel-client---server))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| M-OSMC-1 | Ω1 | Θ1 | Returned `target` is *above* client's `max_target` and no `OpenMiningChannel.Error` sent | [§5.3.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#532-openstandardminingchannel-client---server) "Server MUST accept the target or respond by sending OpenMiningChannel.Error" | Assert `target <= max_target` | `OpenStandardMiningChannel.Success`, `OpenMiningChannel.Error` |
| M-OSMC-2 | Ω3 | Θ1 | Client completes `SetupConnection.Success` on a Mining Protocol connection, then opens no channels — server fails to close the idle connection within a reasonable period | [§5.3.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#532-openstandardminingchannel-client---server) (server SHOULD close idle connections) | Connect, handshake, wait; assert connection timeout + disconnect | — |
| M-OSMC-3 | Ω1 | Θ1 | A valid `OpenStandardMiningChannel` receives neither `.Success` nor `.Error` | [§5.3.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#532-openstandardminingchannel-client---server) "Server MUST accept the target or respond by sending OpenMiningChannel.Error" | Send a satisfiable request; assert a terminal `.Success` or `.Error` within timeout; silence violates the exhaustive accept-or-error MUST | — |

#### `OpenStandardMiningChannel.Success` ([§5.3.3](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#533-openstandardminingchannelsuccess-server---client))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| M-OSMCS-1 | Ω1 | Θ1 | `.Success.request_id` doesn't echo the request | [§5.3.3](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#533-openstandardminingchannelsuccess-server---client) | Random `request_id`s; assert pairing | — |
| M-OSMCS-2 | Ω1 | Θ1 | `channel_id` reused for a second channel, or collides with a `group_channel_id` | [§5.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#52-channel) "There MUST NOT be two Channels with the same ID"; [§5.2.3](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#523-group-channel) shared namespace | Open N channels; assert uniqueness | `SetGroupChannel` |
| M-OSMCS-3 | Ω1 | Θ1 | Same `extranonce_prefix` assigned to two standard channels → overlapping search space, duplicate work | [§5.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#51-job) "Mining servers MUST assign a unique subset of the search space to each mining device" | Open 2+ channels; compare prefixes | — |

#### `OpenExtendedMiningChannel` ([§5.3.4](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#534-openextendedminingchannel-client---server))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| M-OEMC-1 | Ω1 | Θ1 | Server doesn't support Extended channels at all | [§5.2.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#522-extended-channel) "upstream servers which accept connections and provide work MUST support Extended Channels" | `OpenExtendedMiningChannel` must not error | `OpenMiningChannel.Error` |
| M-OEMC-2 | Ω1 | Θ1 | Returned `target` is *above* client's `max_target` and no `OpenMiningChannel.Error` sent (extended open) | [§5.3.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#532-openstandardminingchannel-client---server) via [§5.3.4](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#534-openextendedminingchannel-client---server) | Assert `target <= max_target` | `OpenExtendedMiningChannel.Success`, `OpenMiningChannel.Error` |
| M-OEMC-3 | Ω1 | Θ1 | A valid `OpenExtendedMiningChannel` receives neither `.Success` nor `.Error` | [§5.3.4](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#534-openextendedminingchannel-client---server) | Send a satisfiable request; assert a terminal `.Success` or `.Error` within timeout; silence violates the exhaustive accept-or-error MUST | — |

#### `OpenExtendedMiningChannel.Success` ([§5.3.5](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#535-openextendedminingchannelsuccess-server---client))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| M-OEMCS-1 | Ω1 | Θ1 | `.Success.request_id` doesn't echo the request | [§5.3.5](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#535-openextendedminingchannelsuccess-server---client) | Random `request_id`s; assert pairing | — |
| M-OEMCS-2 | Ω1 | Θ1 | `channel_id` reused for a second channel, or collides with a `group_channel_id` | [§5.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#52-channel) "There MUST NOT be two Channels with the same ID"; [§5.2.3](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#523-group-channel) shared namespace | Open N channels; assert uniqueness | `SetGroupChannel` |
| M-OEMCS-3 | Ω1 | Θ1 | Extended channel: `extranonce_size` smaller than requested `min_extranonce_size`, without error | [§5.3.4](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#534-openextendedminingchannel-client---server) / [§5.3.5](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#535-openextendedminingchannelsuccess-server---client) | Assert `extranonce_size >= min_extranonce_size` (or explicit error) | `OpenMiningChannel.Error` |
| M-OEMCS-4 | Ω1 | Θ1 | Channels in the same group have different full extranonce sizes | [§5.2.3](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#523-group-channel), [§5.1.2.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5121-extended-extranonce) | Cross-check full extranonce sizes within group | `SetGroupChannel`, `NewExtendedMiningJob` |
| M-OEMCS-5 | Ω1 | Θ1 | Same extranonce space assigned to two extended channels → overlapping search space, duplicate work | [§5.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#51-job) | Open 2+ channels; compare prefixes | — |
| M-OEMCS-6 | Ω1 | Θ1 | Server negotiates `extranonce_size > 32`, making a valid `SubmitSharesExtended` impossible because `extranonce` is `B0_32` | [§5.3.4](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#534-openextendedminingchannel-client---server) / [§5.3.5](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#535-openextendedminingchannelsuccess-server---client) "extranonce_size MUST be at most 32 bytes" | Request `min_extranonce_size = 33`; assert `OpenMiningChannel.Error`. For successful opens, assert `extranonce_size <= 32` | `OpenMiningChannel.Error` |

#### `OpenMiningChannel.Error` ([§5.3.6](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#536-openminingchannelerror-server---client))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| M-OMCE-1 | Ω1 | Θ1 | `request_id` not echoed on error, or error referencing a `request_id` never sent → response cannot be paired with a request | [§5.3.6](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#536-openminingchannelerror-server---client) field-definition identity semantics | Force an open error; assert pairing; assert no unsolicited errors | — |

#### `UpdateChannel` ([§5.3.7](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#537-updatechannel-client---server))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| M-UC-1 | Ω1 | Θ1 | `UpdateChannel` with a *smaller* `max_target` ignored | [§5.3.7](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#537-updatechannel-client---server) "upstream node MUST reflect the client's request (and send appropriate SetTarget message)" | Send `UpdateChannel`; await matching `SetTarget` | `SetTarget` |

#### `UpdateChannel.Error` ([§5.3.8](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#538-updatechannelerror-server---client))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| M-UCE-1 | Ω2 | Θ1 | `UpdateChannel` for a never-opened or closed `channel_id` gets no `UpdateChannel.Error` (crash, silence, or acceptance instead) | [§5.3.8](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#538-updatechannelerror-server---client) "Sent only when `UpdateChannel` message is invalid" | Send `UpdateChannel` with bogus `channel_id`; assert `UpdateChannel.Error` | — |
| M-UCE-2 | Ω2 | Θ1 | `UpdateChannel.Error` sent for a valid `UpdateChannel` (false rejection) | [§5.3.8](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#538-updatechannelerror-server---client) "When it is accepted by the server, no response is sent back" | Send valid `UpdateChannel`; assert no error response | — |

#### `CloseChannel` ([§5.3.9](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#539-closechannel-client---server-server---client))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| M-CC-1 | Ω1 | Θ1 | Server keeps sending jobs/`SetTarget` for a channel after `CloseChannel` | [§5.3.9](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#539-closechannel-client---server-server---client) "The server MUST stop sending messages for the channel" | Close channel; assert silence | `NewMiningJob`, `NewExtendedMiningJob`, `SetTarget` |
| M-CC-2 | Ω3 | Θ1 | Server-initiated `CloseChannel` mid-session without `reason_code`; client cannot re-open cleanly | [§5.3.9](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#539-closechannel-client---server-server---client) | Resilience test | — |
| M-CC-3 | Ω1 | Θ1 | `CloseChannel` addressing a group channel doesn't close member channels (or closes unrelated ones) | [§5.3.9](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#539-closechannel-client---server-server---client) "all channels belonging to such group MUST be closed" | Group topology probe (only if pool uses groups) | `SetGroupChannel` |
| M-CC-4 | Ω1 | Θ1 | `CloseChannel` `reason_code` contains control characters | [§5.3.9](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#539-closechannel-client---server-server---client), [§3.5](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#35-error-codes) | Validate `reason_code` string; assert no control characters | — |
| M-CC-5 | Ω2 | Θ1 | `CloseChannel` `reason_code` contains non-printable non-control bytes | [§5.3.9](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#539-closechannel-client---server-server---client), [§3.5](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#35-error-codes) | Validate `reason_code` string; assert printable ASCII | — |
| M-CC-6 | Ω1 | Θ2 | Closing one channel (individual, not group) accidentally closes or rejects valid work on another channel on the same connection | [§5.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#52-channel) "All Channels are independent of each other" | Open channels A and B, close A, submit a valid share on B; assert `SubmitShares.Success` | `SubmitSharesStandard`, `SubmitSharesExtended` |

#### `SetExtranoncePrefix` ([§5.3.10](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5310-setextranonceprefix-server---client))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| M-SEP-1 | Ω1 | Θ2 | `SetExtranoncePrefix` applied retroactively to jobs sent before the change (validation mismatch) | [§5.3.10](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5310-setextranonceprefix-server---client) "applicable for all jobs sent after this message" | Submit share on a pre-change job with the old prefix; assert acceptance | `NewExtendedMiningJob`, `SubmitSharesExtended` |

#### `SubmitSharesStandard` ([§5.3.11](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5311-submitsharesstandard-client---server))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| M-SSS-1 | Ω1 | Θ2 | Valid share (meets channel target, correct job/ntime/extranonce) rejected — e.g. endianness bugs in U256 target comparison | [§5.3.21](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5321-settarget-server---client) converse: hashes at-or-below target must be accepted | Grind real share; assert `SubmitShares.Success` | `SubmitShares.Success` |
| M-SSS-2 | Ω1 | Θ2 | Invalid share accepted (hash above target) — breaks difficulty accounting; mining server reports hashrate that doesn't exist | [§5.3.21](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5321-settarget-server---client) "All submits leading to hashes higher than the specified target will be rejected" | Submit share just above target; assert `SubmitShares.Error` | `SubmitShares.Error` |
| M-SSS-3 | Ω1 | Θ1 | Out-of-bounds `ntime` accepted: `< min_ntime` of latest `SetNewPrevHash`, or too far past its receipt time | [§5.3.11](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5311-submitsharesstandard-client---server) MUST | Boundary `ntime` submissions | `SetNewPrevHash`, `SubmitShares.Error` |
| M-SSS-4 | Ω2 | Θ2 | Mining Server rejects shares with rolled `nTime` inside the protocol ntime window (from `SetNewPrevHash`) | [§5.3.11](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5311-submitsharesstandard-client---server) ntime bounds | Submit with `ntime = min_ntime + k`; assert acceptance | `SetNewPrevHash` |
| M-SSS-5 | Ω3 | Θ2 | Duplicate share counted twice — no dedup on `(job_id, nonce, ntime, version)` | Expected pool behavior (spec-permissive — not a spec compliance requirement) | Resubmit identical valid share; assert error or unchanged accounting | `SubmitShares.Success` |
| M-SSS-6 | Ω2 | Θ1 | Share for unknown/closed/never-opened `channel_id` or `job_id` causes crash, silence, or acceptance | [§5.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#52-channel) channel/job scoping | Probe with bogus IDs | `SubmitShares.Error` |
| M-SSS-7 | Ω1 | Θ1 | `SubmitSharesStandard` on an extended channel mishandled | [§5.3.11](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5311-submitsharesstandard-client---server), [§5.3.12](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5312-submitsharesextended-client---server) "Only relevant for extended channels" | Wrong-type probe | `SubmitSharesExtended` |
| M-SSS-8 | Ω1 | Θ2 | Server sets `REQUIRES_FIXED_VERSION` in `SetupConnection.Success.flags` but accepts a Standard share whose BIP323 general-purpose version bits differ from the job | [§5.3.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#531-setupconnection-flags-for-mining-protocol) "Upstream node will not accept any changes to the version field" | Grind a valid share after modifying only the BIP323 general-purpose version bits; assert `SubmitShares.Error` | `SubmitShares.Error` |

#### `SubmitSharesExtended` ([§5.3.12](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5312-submitsharesextended-client---server))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| M-SSE-1 | Ω1 | Θ2 | Valid share rejected (extended channel) | [§5.3.21](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5321-settarget-server---client) converse | Grind real share; assert `SubmitShares.Success` | `SubmitShares.Success` |
| M-SSE-2 | Ω1 | Θ2 | Invalid share accepted (extended channel) | [§5.3.21](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5321-settarget-server---client) | Submit share just above target; assert `SubmitShares.Error` | `SubmitShares.Error` |
| M-SSE-3 | Ω1 | Θ1 | Wrong `extranonce` length accepted (or correct length rejected) on extended channels | [§5.3.12](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5312-submitsharesextended-client---server) "MUST be equal to the negotiated extranonce size" | Off-by-one extranonce lengths | `SubmitShares.Error` |
| M-SSE-4 | Ω1 | Θ1 | Out-of-bounds `ntime` accepted (extended channel) | [§5.3.11](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5311-submitsharesstandard-client---server) MUST | Boundary `ntime` submissions | `SetNewPrevHash` |
| M-SSE-5 | Ω2 | Θ2 | Rolled `nTime` inside the protocol ntime window rejected (extended channel) | [§5.3.11](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5311-submitsharesstandard-client---server) ntime bounds | Submit with `ntime = min_ntime + k`; assert acceptance | `SetNewPrevHash` |
| M-SSE-6 | Ω3 | Θ2 | Duplicate share counted twice (extended channel) | Expected pool behavior (spec-permissive — not a spec compliance requirement) | Resubmit identical valid share; assert error or unchanged accounting | `SubmitShares.Success` |
| M-SSE-7 | Ω2 | Θ1 | Share for unknown/closed/never-opened `channel_id` or `job_id` causes crash, silence, or acceptance (extended channel) | [§5.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#52-channel) channel/job scoping | Probe with bogus IDs | `SubmitShares.Error` |
| M-SSE-8 | Ω1 | Θ1 | `SubmitSharesExtended` on a standard channel mishandled | [§5.3.12](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5312-submitsharesextended-client---server) "Only relevant for extended channels" | Wrong-type probe | `SubmitSharesStandard` |

#### `SubmitShares.Success` ([§5.3.13](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5313-submitsharessuccess-server---client))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| M-SSOK-1 | Ω1 | Θ2 | Batch semantics broken: `last_sequence_number` ≠ client's last accepted seq; `new_submits_accepted_count` ≠ batch size; `new_shares_sum` ≠ sum of batch share difficulties; counters not reset between batches | [§5.3.13](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5313-submitsharessuccess-server---client) "their respective counters MUST be reset when a new batch starts being processed" | Submit known-difficulty shares in controlled batches; verify all three fields arithmetically | `SubmitSharesStandard`, `SubmitSharesExtended` |
| M-SSOK-2 | Ω2 | Θ2 | `.Success` acknowledging shares never submitted (`last_sequence_number` ahead of what the client sent) | [§5.3.13](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5313-submitsharessuccess-server---client) | Sequence audit | — |

#### `SubmitShares.Error` ([§5.3.14](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5314-submitshareserror-server---client))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| M-SSERR-1 | Ω1 | Θ1 | Errors silently dropped: incorrect submit never gets a `SubmitShares.Error` | [§5.3.14](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5314-submitshareserror-server---client) "An error is immediately submitted for every incorrect submit attempt" | Timeout assertion on every bad share | — |
| M-SSERR-2 | Ω2 | Θ1 | `SubmitShares.Error` referencing the wrong `sequence_number` (cannot be paired with the offending submit) | [§5.3.14](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5314-submitshareserror-server---client) | Pairing assertion | — |

#### `NewMiningJob` ([§5.3.15](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5315-newminingjob-server---client))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| M-NMJ-1 | Ω1 | Θ1 | Server sends nothing (or another message) after standard channel open — the first message MUST be a `NewMiningJob` | [§5.3.15](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5315-newminingjob-server---client) "This MUST be the first message after the channel has been successfully opened" | Message-order assertion | `OpenStandardMiningChannel.Success`, `SetNewPrevHash` |
| M-NMJ-2 | Ω1 | Θ1 | First job arrives with `min_ntime` **set** (active job before any `SetNewPrevHash` exists) | [§5.3.15](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5315-newminingjob-server---client) "This first message will have min_ntime unset" | Assert first job is a future job | `SetNewPrevHash` |
| M-NMJ-3 | Ω1 | Θ1 | `NewMiningJob.channel_id` references an extended/group channel | [§5.3.15](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5315-newminingjob-server---client) "this must be a standard channel" | Type/channel-kind cross-check | — |
| M-NMJ-4 | Ω1 | Θ1 | `job_id` collisions across simultaneously valid jobs, or jobs referencing IDs never announced | [§5.3.15](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5315-newminingjob-server---client)–5.3.17 pairing semantics | Track `job_id` space | `SetNewPrevHash` |
| M-NMJ-5 | Ω1 | Θ2 | Mining Server rejects shares with rolled BIP323 general-purpose version bits despite advertising version rolling | [§5.3.15](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5315-newminingjob-server---client) (downstream may freely manipulate BIP323 bits; "MUST NOT rely on the upstream node to set the BIP323 bits") | Submit valid share with rolled version bits; assert acceptance when rolling allowed | `SubmitSharesStandard`, `SubmitShares.Error` |

#### `NewExtendedMiningJob` ([§5.3.16](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5316-newextendedminingjob-server---client))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| M-NEMJ-1 | Ω1 | Θ1 | Server sends nothing (or another message) after extended channel open — the first message MUST be a `NewExtendedMiningJob` | [§5.3.16](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5316-newextendedminingjob-server---client) | Message-order assertion | `OpenExtendedMiningChannel.Success`, `SetNewPrevHash` |
| M-NEMJ-2 | Ω1 | Θ1 | First extended job arrives with `min_ntime` **set** (active job before any `SetNewPrevHash` exists) | [§5.3.16](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5316-newextendedminingjob-server---client) | Assert first job is a future job | `SetNewPrevHash` |
| M-NEMJ-3 | Ω1 | Θ1 | `NewExtendedMiningJob` targets a standard channel | [§5.3.16](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5316-newextendedminingjob-server---client) (extended/group channels only) | Type/channel-kind cross-check | — |
| M-NEMJ-4 | Ω1 | Θ1 | `NewExtendedMiningJob` sent on a connection where client set `REQUIRES_STANDARD_JOBS` | [§5.3.16](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5316-newextendedminingjob-server---client), [§5.2.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#521-standard-channel) | Assert absence | `SetupConnection` |
| M-NEMJ-5 | Ω1 | Θ1 | BIP141 stripping violation: `coinbase_tx_prefix`/`coinbase_tx_suffix` still contain marker/flag/witness data → client-computed coinbase txid and merkle root are wrong → every share built on the job is garbage | [§5.3.16](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5316-newextendedminingjob-server---client) "MUST be stripped of BIP141 fields" | Reconstruct coinbase on extended channel; assert no witness fields present | — |
| M-NEMJ-6 | Ω1 | Θ1 | `job_id` collisions across simultaneously valid jobs (extended) | [§5.3.16](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5316-newextendedminingjob-server---client)–5.3.17 pairing semantics | Track `job_id` space | `SetNewPrevHash` |
| M-NEMJ-7 | Ω1 | Θ2 | Mining Server rejects shares with rolled BIP323 bits despite `version_rolling_allowed = True` | [§5.3.16](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5316-newextendedminingjob-server---client) (downstream may freely manipulate BIP323 bits; "MUST NOT rely on the upstream node to set the BIP323 bits") | Submit valid share with rolled version bits; assert acceptance when rolling allowed | `SubmitSharesExtended`, `SubmitShares.Error` |
| M-NEMJ-8 | Ω1 | Θ1 | Client sets `REQUIRES_VERSION_ROLLING` in `SetupConnection.flags`; server accepts but later sends `NewExtendedMiningJob` with `version_rolling_allowed = False` | [§5.3.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#531-setupconnection-flags-for-mining-protocol) "server MUST NOT send jobs which do not allow version rolling" | Assert `version_rolling_allowed` is always True when `REQUIRES_VERSION_ROLLING` was set | `NewExtendedMiningJob` |
| M-NEMJ-9 | Ω1 | Θ2 | Server sends `NewExtendedMiningJob` with `version_rolling_allowed = False` but accepts a share whose BIP323 general-purpose version bits differ from the job | [§5.3.16](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5316-newextendedminingjob-server---client) "If set to False, the downstream node MUST use version as it is defined by this message" | Grind a valid share after modifying only the BIP323 general-purpose version bits; assert `SubmitShares.Error` | `SubmitSharesExtended`, `SubmitShares.Error` |

#### `SetNewPrevHash` ([§5.3.17](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5317-setnewprevhash-server---client-broadcast))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| M-SNPH-1 | Ω1 | Θ1 | `SetNewPrevHash` activates a `job_id` never sent as a future job | [§5.3.15](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5315-newminingjob-server---client) / [§5.3.17](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5317-setnewprevhash-server---client-broadcast) | Cross-reference announced future jobs | `NewMiningJob`, `NewExtendedMiningJob` |
| M-SNPH-2 | Ω2 | Θ2 | Mining Server keeps accepting shares for jobs invalidated by a `SetNewPrevHash` (stale acceptance → wrong accounting, misleading hashrate) | [§5.3.17](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5317-setnewprevhash-server---client-broadcast) "only the job referenced by Job ID is valid" | Submit share for invalidated job; expect `SubmitShares.Error` | `SubmitShares.Error`, `SubmitShares.Success` |
| M-SNPH-3 | Ω1 | Θ1 | `min_ntime` in `SetNewPrevHash` wildly in the future (beyond `MAX_FUTURE_BLOCK_TIME`) → any block found would be invalid | [§5.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#51-job) | Sanity-bound assertion | — |

#### `SetCustomMiningJob` ([§5.3.18](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5318-setcustomminingjob-client---server))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| M-SCMJ-1 | Ω2 | Θ1 | `SetCustomMiningJob` accepted on a connection that never declared `REQUIRES_WORK_SELECTION` | [§5.3.18](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5318-setcustomminingjob-client---server) "SetupConnection.flags MUST contain REQUIRES_WORK_SELECTION" | Send without declaring; expect `SetCustomMiningJob.Error` | `SetupConnection`, `SetCustomMiningJob.Error` |
| M-SCMJ-2 | Ω1 | Θ1 | `SetCustomMiningJob` sent on a group channel that contains standard channels, and the custom job leaks into those standard channels | [§5.3.18](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5318-setcustomminingjob-client---server) "the server MUST ignore those" | Send on group channel containing standard channels; assert standard channels do not receive custom work | `SetGroupChannel` |

#### `SetCustomMiningJob.Success` ([§5.3.19](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5319-setcustomminingjobsuccess-server---client))

No scenarios yet — custom-job flows are exercised by JDC suites.

#### `SetCustomMiningJob.Error` ([§5.3.20](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5320-setcustomminingjoberror-server---client))

No scenarios yet.

#### `SetTarget` ([§5.3.21](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5321-settarget-server---client))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| M-ST-1 | Ω1 | Θ1 | `SetTarget.target` above the client's declared `max_target` | [§5.3.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#532-openstandardminingchannel-client---server) / [§5.3.7](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#537-updatechannel-client---server) (client's max is a hard bound) | Assert bound on every `SetTarget` | `OpenStandardMiningChannel`, `UpdateChannel` |
| M-ST-2 | Ω1 | Θ2 | Mining Server applies new target retroactively to active jobs that had `min_ntime` set | [§5.3.21](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5321-settarget-server---client) "not applicable for already received jobs with min_ntime=nTime" | Submit share valid under old target on a pre-`SetTarget` job; assert old target applies | `NewMiningJob`, `SubmitSharesStandard` |
| M-ST-3 | Ω1 | Θ2 | A `SetTarget` update for one channel (individual, not group) changes validation on another independent channel | [§5.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#52-channel) "All Channels are independent of each other" | Open channels A and B, reduce A's target via `UpdateChannel`; submit a share on B valid under B's unchanged target; assert `SubmitShares.Success` | `UpdateChannel`, `SubmitSharesStandard`, `SubmitSharesExtended` |

#### `SetGroupChannel` ([§5.3.22](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5322-setgroupchannel-server---client))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| M-SGC-1 | Ω1 | Θ1 | `SetGroupChannel` sent on a connection with `REQUIRES_STANDARD_JOBS` | [§5.3.22](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5322-setgroupchannel-server---client) "can be sent only to connections that don't have REQUIRES_STANDARD_JOBS" | Assert absence | `SetupConnection` |
| M-SGC-2 | Ω1 | Θ1 | `group_channel_id` collides with an existing mining `channel_id` | [§5.2.3](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#523-group-channel), [§5.3.22](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5322-setgroupchannel-server---client) "MUST ensure that a group channel has a unique channel ID" | Cross-check ID spaces | `OpenStandardMiningChannel.Success`, `OpenExtendedMiningChannel.Success` |

### Protocol Extensions ([§3.4](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#34-protocol-extensions))

One subsection per registered extension; future registrations append new subsections. Generic extension machinery (unknown `extension_type` handling, TLV invariants, negotiation-before-use) lives in the [General](#general-3135-5152) table as GEN-8…GEN-13.

**Applicability caveat:** a server that implements no extensions is compliant — it simply ignores `RequestExtensions`. Therefore *no response* to `RequestExtensions` is not a failure; the `X0001-*` scenarios below only apply once a server engages with the negotiation protocol.

#### Extension `0x0001` — [Extensions Negotiation](https://github.com/stratum-mining/sv2-spec/blob/main/extensions/0x0001-extensions-negotiation.md)

##### `RequestExtensions`

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| X0001-REQ-1 | Ω2 | Θ1 | Server requires extensions the client didn't request, but keeps the connection open after the client doesn't retry | ext `0x0001` §4 ("the server MUST disconnect the client") | Omit a server-required extension from the request; do not retry; assert disconnect | `RequestExtensions.Error` |

##### `RequestExtensions.Success`

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| X0001-REQS-1 | Ω1 | Θ1 | `request_id` doesn't echo the request | ext `0x0001` §2 (field-definition identity semantics) | Random `request_id`s; assert pairing | — |
| X0001-REQS-2 | Ω1 | Θ1 | Extension-defined message framed with the wrong `extension_type` (must be `0x0001`, not `0x0000`) | [§3.4.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#341-extension-type-field-usage), ext `0x0001` §3 | Assert `extension_type == 0x0001` (after masking `channel_msg` bit) on every negotiation frame | `RequestExtensions.Error` |
| X0001-REQS-3 | Ω2 | Θ1 | `RequestExtensions.Success.supported_extensions` contains extension IDs that were not in the client's request | ext `0x0001` §1 (derived — negotiation semantics: Success confirms which of the requested extensions are supported) | Request a known set of extensions; assert `supported_extensions` ⊆ requested set | — |

##### `RequestExtensions.Error`

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection | Involves |
|---|---|---|---|---|---|---|
| X0001-REQE-1 | Ω2 | Θ1 | None of the requested extensions are supported, but server doesn't respond with `RequestExtensions.Error` | ext `0x0001` §4.1 ("MUST respond with `RequestExtensions.Error` if none of the requested extensions are supported") | Request only unallocated extension IDs; assert `Error` | — |
| X0001-REQE-2 | Ω2 | Θ1 | `Error` contents wrong: `unsupported_extensions` ≠ the requested-but-unsupported set, or `required_extensions` omits extensions the server requires | ext `0x0001` §1.1, §4.1 | Request a mix of supported and bogus IDs; assert exact sets | — |
| X0001-REQE-3 | Ω1 | Θ1 | `request_id` doesn't echo the request | ext `0x0001` §2 (field-definition identity semantics) | Random `request_id`s; assert pairing | — |

### Job Declaration Protocol Messages ([§6](https://github.com/stratum-mining/sv2-spec/blob/main/06-Job-Declaration-Protocol.md))

Reserved for future suites (JDS/JDC contexts). No scenarios yet.

### Template Distribution Protocol Messages ([§7](https://github.com/stratum-mining/sv2-spec/blob/main/07-Template-Distribution-Protocol.md))

Reserved for future suites (TP contexts). No scenarios yet.

---

## Suite Coverage

✅ = covered; ❌ = not covered

| Scenario IDs | Solo Pool |
|---|---|
| GEN-1 — `channel_msg` bit correctness | ❌ |
| GEN-2 — `extension_type` on core messages | ❌ |
| GEN-3 — error code control characters | ❌ |
| GEN-4 — error code non-printable bytes | ❌ |
| GEN-5 — `channel_id` already closed | ❌ |
| GEN-6 — `channel_id` never opened | ❌ |
| GEN-7 — job starvation (liveness) | ❌ |
| GEN-8 — unknown `extension_type` discarded | ❌ |
| GEN-9 — unexpected TLV fields ignored | ❌ |
| GEN-10 — server TLV placement/ordering | ❌ |
| GEN-11 — over-length TLV rejected | ❌ |
| GEN-12 — proxy forwards unknown-extension frames unmodified | ❌ |
| GEN-13 — no extension messages without negotiation | ❌ |
| GEN-14 — contradictory double response | ❌ |
| C-SC-1 — no messages before `SetupConnection` response | ❌ |
| C-SC-2 — rejects empty `device_id` / crashes on boundary strings | ❌ |
| C-SC-3 — `REQUIRES_FIXED_VERSION` vs `REQUIRES_VERSION_ROLLING` | ❌ |
| C-SC-4 — `REQUIRES_STANDARD_JOBS` must be accepted | ❌ |
| C-SC-5 — `REQUIRES_EXTENDED_CHANNELS` coherence | ❌ |
| C-SC-6 — `REQUIRES_FIXED_VERSION` + `version_rolling_allowed` consistency | ❌ |
| C-SC-7 — unsupported or invalid `protocol` value | ❌ |
| C-SC-8 — version range with no supported version | ❌ |
| C-SCS-1 — `SetupConnection` response type / TCP close vs framed error | ❌ |
| C-SCS-2 — `used_version` within negotiated range | ❌ |
| C-SCE-1 — feature-probing missing/mismatched error flags | ❌ |
| C-SCE-2 — flag-set stability across connections | ❌ |
| M-OSMC-1 — target vs `max_target` on channel open | ❌ |
| M-OSMC-2 — inactivity timeout on idle mining connection | ❌ |
| M-OSMC-3 — no response to standard channel open | ❌ |
| M-OSMCS-1 — `request_id` echo in channel open success | ❌ |
| M-OSMCS-2 — `channel_id` uniqueness | ❌ |
| M-OSMCS-3 — `extranonce_prefix` uniqueness | ❌ |
| M-OEMC-1 — Extended channel support | ❌ |
| M-OEMC-2 — target vs `max_target` on extended open | ❌ |
| M-OEMC-3 — no response to extended channel open | ❌ |
| M-OEMCS-1 — `request_id` echo in extended open success | ❌ |
| M-OEMCS-2 — `channel_id` uniqueness (extended) | ❌ |
| M-OEMCS-3 — `extranonce_size` vs `min_extranonce_size` | ❌ |
| M-OEMCS-4 — same group, different full extranonce sizes | ❌ |
| M-OEMCS-5 — extranonce space uniqueness (extended) | ❌ |
| M-OEMCS-6 — `extranonce_size > 32` | ❌ |
| M-OMCE-1 — `request_id` echo on open error | ❌ |
| M-UC-1 — `UpdateChannel` target reduction honored | ❌ |
| M-UCE-1 — `UpdateChannel.Error` for invalid channel | ❌ |
| M-UCE-2 — no `UpdateChannel.Error` on valid update | ❌ |
| M-CC-1 — silence after `CloseChannel` | ❌ |
| M-CC-2 — server-initiated `CloseChannel` with reason_code | ❌ |
| M-CC-3 — group-channel `CloseChannel` correctness | ❌ |
| M-CC-4 — `reason_code` control characters | ❌ |
| M-CC-5 — `reason_code` non-printable non-control bytes | ❌ |
| M-CC-6 — closing one channel breaks another independent channel | ❌ |
| M-SEP-1 — `SetExtranoncePrefix` non-retroactivity | ❌ |
| M-SSS-1 — valid-share acceptance (standard) | ❌ |
| M-SSS-2 — invalid-share rejection (standard) | ❌ |
| M-SSS-3 — `ntime` bounds enforcement (standard) | ❌ |
| M-SSS-4 — `nTime`-rolling acceptance (standard) | ❌ |
| M-SSS-5 — duplicate-share rejection (standard) | ❌ |
| M-SSS-6 — bogus IDs in share submission (standard) | ❌ |
| M-SSS-7 — `SubmitSharesStandard` on extended channel | ❌ |
| M-SSS-8 — fixed-version share accepted (standard) | ❌ |
| M-SSE-1 — valid-share acceptance (extended) | ❌ |
| M-SSE-2 — invalid-share rejection (extended) | ❌ |
| M-SSE-3 — extranonce length validation | ❌ |
| M-SSE-4 — `ntime` bounds enforcement (extended) | ❌ |
| M-SSE-5 — `nTime`-rolling acceptance (extended) | ❌ |
| M-SSE-6 — duplicate-share rejection (extended) | ❌ |
| M-SSE-7 — bogus IDs in share submission (extended) | ❌ |
| M-SSE-8 — `SubmitSharesExtended` on standard channel | ❌ |
| M-SSOK-1 — `SubmitShares.Success` batch accounting | ❌ |
| M-SSOK-2 — `last_sequence_number` audit | ❌ |
| M-SSERR-1 — `SubmitShares.Error` always sent for bad submits | ❌ |
| M-SSERR-2 — `SubmitShares.Error` sequence_number pairing | ❌ |
| M-NMJ-1 — first message after standard open is a job | ❌ |
| M-NMJ-2 — first standard job `min_ntime` unset | ❌ |
| M-NMJ-3 — `NewMiningJob` channel-kind cross-check | ❌ |
| M-NMJ-4 — standard `job_id` uniqueness and pairing | ❌ |
| M-NMJ-5 — version-rolling share acceptance (standard) | ❌ |
| M-NEMJ-1 — first message after extended open is a job | ❌ |
| M-NEMJ-2 — first extended job `min_ntime` unset | ❌ |
| M-NEMJ-3 — `NewExtendedMiningJob` channel-kind cross-check | ❌ |
| M-NEMJ-4 — `NewExtendedMiningJob` vs `REQUIRES_STANDARD_JOBS` | ❌ |
| M-NEMJ-5 — BIP141 stripping in coinbase tx data | ❌ |
| M-NEMJ-6 — extended `job_id` uniqueness and pairing | ❌ |
| M-NEMJ-7 — version-rolling share acceptance (extended) | ❌ |
| M-NEMJ-8 — version-rolling loophole (`version_rolling_allowed = False` after `REQUIRES_VERSION_ROLLING`) | ❌ |
| M-NEMJ-9 — non-rolling share accepted (extended) | ❌ |
| M-SNPH-1 — `SetNewPrevHash` references known future `job_id` | ❌ |
| M-SNPH-2 — stale-share rejection | ❌ |
| M-SNPH-3 — `min_ntime` sanity bounds | ❌ |
| M-SCMJ-1 — `SetCustomMiningJob` requires declared work selection | ❌ |
| M-SCMJ-2 — custom-job leak into standard channels on group | ❌ |
| M-ST-1 — `SetTarget` bound vs client `max_target` | ❌ |
| M-ST-2 — `SetTarget` non-retroactivity | ❌ |
| M-ST-3 — target update leaks to another independent channel | ❌ |
| M-SGC-1 — `SetGroupChannel` vs `REQUIRES_STANDARD_JOBS` | ❌ |
| M-SGC-2 — group/channel ID namespace separation | ❌ |
| X0001-REQ-1 — disconnect when required extensions ignored | ❌ |
| X0001-REQS-1 — `request_id` echo on negotiation success | ❌ |
| X0001-REQS-2 — negotiation frames carry `extension_type = 0x0001` | ❌ |
| X0001-REQS-3 — `supported_extensions` not a subset of the request | ❌ |
| X0001-REQE-1 — `Error` when nothing requested is supported | ❌ |
| X0001-REQE-2 — `Error` unsupported/required sets exact | ❌ |
| X0001-REQE-3 — `request_id` echo on negotiation error | ❌ |


Note: more columns will be added as the project matures (Pool, JDS, TP, Proxy suites).