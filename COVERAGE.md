# Sv2 Protocol Coverage

This document formally describes how `interoperability-tests-sv2` covers the [Stratum V2 Protocol Specification](https://github.com/stratum-mining/sv2-spec).

§ indicates spec sections.

## Scenarios

A **Scenario** is a single testable Sv2 Protocol behavior — each describes a failure mode, its spec basis, and how a client detects it.

Scenarios are application-agnostic: each row describes a generic Sv2 Protocol behavior, which might potentially happen under different application contexts.

**Scenario IDs** are unique identifiers and stable references. New scenarios append; existing IDs are never renumbered.

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

## Scenario Catalog

### A. Framing & common-message rules ([§3.2](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#32-framing), [§3.4](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#34-protocol-extensions), [§3.5](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#35-error-codes), [§8](https://github.com/stratum-mining/sv2-spec/blob/main/08-Message-Types.md#8-message-types))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection |
|--------------|---------------|----------------|---|---|---|
| A1 | Ω1 | Θ1 | `channel_msg` bit wrong per message type (e.g. `NewMiningJob` 0x15 sent with bit unset, `OpenStandardMiningChannel.Success` 0x11 with bit set) | [§3.2.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#321-routing-frames-over-channels), [§8](https://github.com/stratum-mining/sv2-spec/blob/main/08-Message-Types.md#8-message-types) table | Assert bit on every received frame |
| A2 | Ω1 | Θ1 | Core messages sent with non-zero `extension_type` | [§3.4.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#341-extension-type-field-usage) | Assert `extension_type & 0x7FFF == 0x0000` (mask `channel_msg` bit first) |
| A3 | Ω1 | Θ1 | Error codes containing control characters | [§3.5](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#35-error-codes) | Validate every `error_code` string |
| A4 | Ω2 | Θ1 | Error codes containing non-printable non-control bytes | [§3.5](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#35-error-codes) | Validate every `error_code` string |
| A5 | Ω1 | Θ1 | Server sends *any* protocol message before responding to `SetupConnection` | [§3.6.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#361-setupconnection-client---server) ("MUST be the first message") | Message-order assertion |
| A6 | Ω1 | Θ1 | Server sends messages for a `channel_id` already closed on this connection | [§5.3.9](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#539-closechannel-client---server-server---client) "The server MUST stop sending messages for the channel" | Track closed channel set; assert silence |
| A7 | Ω2 | Θ1 | Server sends messages for a `channel_id` never opened on this connection | [§5.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#52-channel) (channel ID consistency) | Track open channel set; assert all inbound channel messages reference it |

### B. `SetupConnection` negotiation ([§3.6](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#36-common-protocol-messages), [§5.3.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#531-setupconnection-flags-for-mining-protocol))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection |
|--------------|---------------|----------------|---|---|---|
| B1 | Ω1 | Θ1 | Server fails to respond to `SetupConnection` with a properly framed `.Success` or `.Error` — silence, wrong message type, or TCP close instead of a protocol error | [§3.6.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#361-setupconnection-client---server) "MUST respond with either .Success or .Error" | Assert framed `.Success` or `.Error` within timeout; assert not bare TCP close |
| B2 | Ω1 | Θ1 | Server accepts `SetupConnection` with an unsupported or invalid `protocol` value | [§3.6.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#361-setupconnection-client---server) | Send `SetupConnection` with `protocol=1` (JDC) or `protocol=3` (invalid); assert `SetupConnection.Error` |
| B3 | Ω1 | Θ1 | `used_version` outside client's `[min_version, max_version]` | [§3.6.2](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#362-setupconnectionsuccess-server---client) | Send `min_version = max_version = 2`; assert `used_version == 2` |
| B4 | Ω1 | Θ1 | Client sets all flags (feature probing); server errors with non-zero flags that are missing or mismatched | [§3.6.3](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#363-setupconnectionerror-server---client) "MUST provide the full set of flags which it does not support" | All-flags probe; if flags ≠ 0 assert they exactly equal the unsupported set; flags=0 means rejection for an unrelated reason |
| B5 | Ω1 | Θ1 | Server sets `REQUIRES_FIXED_VERSION` in `.Success.flags` even though client set `REQUIRES_VERSION_ROLLING` | [§5.3.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#531-setupconnection-flags-for-mining-protocol) "MUST NOT be set" | Flag cross-check |
| B6 | Ω2 | Θ1 | Server sets `REQUIRES_FIXED_VERSION` but later sends `NewExtendedMiningJob` with `version_rolling_allowed = True` | [§5.3.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#531-setupconnection-flags-for-mining-protocol) | Cross-message consistency |
| B7 | Ω2 | Θ1 | Server sets `REQUIRES_EXTENDED_CHANNELS` but then accepts `OpenStandardMiningChannel` (or rejects it, contradicting a standard-only deployment) | [§5.3.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#531-setupconnection-flags-for-mining-protocol) bit 1 | Open standard channel; assert coherent behavior |
| B8 | Ω1 | Θ1 | Server rejects `REQUIRES_STANDARD_JOBS` (bit 0) — fails the most basic client class | [§5.2.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#522-extended-channel) "MUST also support Standard Channels" | Minimal-flags `SetupConnection` must succeed |
| B9 | Ω1 | Θ1 | Server doesn't support Extended channels at all | [§5.2.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#522-extended-channel) "upstream servers which accept connections and provide work MUST support Extended Channels" | `OpenExtendedMiningChannel` must not error |
| B10 | Ω2 | Θ1 | Server rejects empty `device_id` or crashes on boundary device-info strings (255-byte or empty) | [§3.6.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#361-setupconnection-client---server), [§3.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#31-data-types-mapping) types | Send `SetupConnection` with empty `device_id` (explicitly allowed); assert no error. Send with 255-byte strings; assert no crash |


### C. Channel opening ([§5.3.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#532-openstandardminingchannel-client---server)–[§5.3.6](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#536-openminingchannelerror-server---client)

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection |
|--------------|---------------|----------------|---|---|---|
| C1 | Ω1 | Θ1 | `.Success.request_id` doesn't echo the request | [§5.3.3](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#533-openstandardminingchannelsuccess-server---client) / [§5.3.5](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#535-openextendedminingchannelsuccess-server---client) | Random `request_id`s; assert pairing |
| C2 | Ω1 | Θ1 | Returned `target` is *above* client's `max_target` and no `OpenMiningChannel.Error` sent | [§5.3.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#532-openstandardminingchannel-client---server) "Server MUST accept the target or respond by sending OpenMiningChannel.Error" | Assert `target <= max_target` |
| C3 | Ω1 | Θ1 | `channel_id` reused for a second channel, or collides with a `group_channel_id` | [§5.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#52-channel) "There MUST NOT be two Channels with the same ID"; [§5.2.3](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#523-group-channel) shared namespace | Open N channels; assert uniqueness |
| C4 | Ω1 | Θ1 | Same `extranonce_prefix` assigned to two standard channels → overlapping search space, duplicate work | [§5.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#51-job) "Mining servers MUST assign a unique subset of the search space to each mining device" | Open 2+ channels; compare prefixes |
| C5 | Ω1 | Θ1 | Extended channel: `extranonce_size` smaller than requested `min_extranonce_size`, without error | [§5.3.4](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#534-openextendedminingchannel-client---server) / [§5.3.5](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#535-openextendedminingchannelsuccess-server---client) | Assert `extranonce_size >= min_extranonce_size` (or explicit error) |
| C6 | Ω1 | Θ1 | Channels in the same group have different full extranonce sizes | [§5.2.3](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#523-group-channel), [§5.1.2.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5121-extended-extranonce) | Cross-check full extranonce sizes within group |
| C7 | Ω1 | Θ1 | Server sends nothing (or another message) after channel open — the first message MUST be a `NewMiningJob`/`NewExtendedMiningJob` | [§5.3.15](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5315-newminingjob-server---client) "This MUST be the first message after the channel has been successfully opened" | Message-order assertion |
| C8 | Ω1 | Θ1 | First job arrives with `min_ntime` **set** (active job before any `SetNewPrevHash` exists) | [§5.3.15](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5315-newminingjob-server---client) "This first message will have min_ntime unset" | Assert first job is a future job |

### D. Job distribution ([§5.3.15](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5315-newminingjob-server---client)–[§5.3.16](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5316-newextendedminingjob-server---client)

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection |
|--------------|---------------|----------------|---|---|---|
| D1 | Ω1 | Θ1 | `NewMiningJob.channel_id` references an extended/group channel, or `NewExtendedMiningJob` targets a standard channel | [§5.3.15](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5315-newminingjob-server---client) "this must be a standard channel"; [§5.3.16](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5316-newextendedminingjob-server---client) | Type/channel-kind cross-check |
| D2 | Ω1 | Θ1 | `job_id` collisions across simultaneously valid jobs, or jobs referencing IDs never announced | [§5.3.15](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5315-newminingjob-server---client)–5.3.17 pairing semantics | Track `job_id` space |
| D3 | Ω1 | Θ1 | `NewExtendedMiningJob` sent on a connection where client set `REQUIRES_STANDARD_JOBS` | [§5.3.16](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5316-newextendedminingjob-server---client), [§5.2.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#521-standard-channel) | Assert absence |
| D4 | Ω1 | Θ1 | BIP141 stripping violation: `coinbase_tx_prefix`/`coinbase_tx_suffix` still contain marker/flag/witness data → client-computed coinbase txid and merkle root are wrong → every share built on the job is garbage | [§5.3.16](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5316-newextendedminingjob-server---client) "MUST be stripped of BIP141 fields" | Reconstruct coinbase on extended channel; assert no witness fields present |
| D5 | Ω1 | Θ2 | Mining Server rejects shares with rolled BIP323 general-purpose version bits despite advertising version rolling | [§5.3.15](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5315-newminingjob-server---client)/[§5.3.16](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5316-newextendedminingjob-server---client)(downstream may freely manipulate BIP323 bits; "MUST NOT rely on the upstream node to set the BIP323 bits") | Submit valid share with rolled version bits; assert acceptance when rolling allowed |
| D6 | Ω2 | Θ2 | Mining Server rejects shares with rolled `nTime` inside the protocol ntime window (from `SetNewPrevHash`) | [§5.3.11](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5311-submitsharesstandard-client---server) ntime bounds | Submit with `ntime = min_ntime + k`; assert acceptance |
| D7 | Ω3 | Θ1 | Job starvation: mining server never sends fresh jobs after the first, stranding miners on an exhausted search space | [§5.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#51-job) (server must manage hashing space and "provide new and unique Jobs quickly enough") | Soft liveness/timing metric |

### E. `SetNewPrevHash` & future-job lifecycle ([§5.1.3](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#513-future-job), [§5.3.17](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5317-setnewprevhash-server---client-broadcast))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection |
|--------------|---------------|----------------|---|---|---|
| E1 | Ω1 | Θ1 | `SetNewPrevHash` activates a `job_id` never sent as a future job | [§5.3.15](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5315-newminingjob-server---client) / [§5.3.17](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5317-setnewprevhash-server---client-broadcast) | Cross-reference announced future jobs |
| E2 | Ω1 | Θ2 | Mining Server keeps accepting shares for jobs invalidated by a `SetNewPrevHash` (stale acceptance → wrong accounting, misleading hashrate) | [§5.3.17](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5317-setnewprevhash-server---client-broadcast) "only the job referenced by Job ID is valid" | Submit share for invalidated job; expect `SubmitShares.Error` |
| E3 | Ω1 | Θ1 | `min_ntime` in `SetNewPrevHash` wildly in the future (beyond `MAX_FUTURE_BLOCK_TIME`) → any block found would be invalid | [§5.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#51-job) | Sanity-bound assertion |

### F. `SetTarget` / `UpdateChannel` ([§5.3.7](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#537-updatechannel-client---server), [§5.3.21](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5321-settarget-server---client))

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection |
|--------------|---------------|----------------|---|---|---|
| F1 | Ω1 | Θ1 | `SetTarget.maximum_target` above the client's declared `max_target` | [§5.3.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#532-openstandardminingchannel-client---server) / [§5.3.7](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#537-updatechannel-client---server) (client's max is a hard bound) | Assert bound on every `SetTarget` |
| F2 | Ω1 | Θ2 | Mining Server applies new target retroactively to active jobs that had `min_ntime` set | [§5.3.21](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5321-settarget-server---client) "not applicable for already received jobs with min_ntime=nTime" | Submit share valid under old target on a pre-`SetTarget` job; assert old target applies |
| F3 | Ω1 | Θ1 | `UpdateChannel` with a *smaller* `maximum_target` ignored | [§5.3.7](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#537-updatechannel-client---server) "upstream node MUST reflect the client's request (and send appropriate SetTarget message)" | Send `UpdateChannel`; await matching `SetTarget` |

### G. Share submission & accounting ([§5.3.11](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5311-submitsharesstandard-client---server)–[§5.3.14](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5314-submitshareserror-server---client)

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection |
|--------------|---------------|----------------|---|---|---|
| G1 | Ω1 | Θ2 | Valid share (meets channel target, correct job/ntime/extranonce) rejected — e.g. endianness bugs in U256 target comparison | [§5.3.21](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5321-settarget-server---client) converse: hashes at-or-below target must be accepted | Grind real share; assert `SubmitShares.Success` |
| G2 | Ω1 | Θ2 | Invalid share accepted (hash above target) — breaks difficulty accounting; mining server reports hashrate that doesn't exist | [§5.3.21](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5321-settarget-server---client) "All submits leading to hashes higher than the specified target will be rejected" | Submit share just above target; assert `SubmitShares.Error` |
| G3 | Ω1 | Θ1 | Wrong `extranonce` length accepted (or correct length rejected) on extended channels | [§5.3.12](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5312-submitsharesextended-client---server) "MUST be equal to the negotiated extranonce size" | Off-by-one extranonce lengths |
| G4 | Ω1 | Θ1 | Out-of-bounds `ntime` accepted: `< min_ntime` of latest `SetNewPrevHash`, or too far past its receipt time | [§5.3.11](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5311-submitsharesstandard-client---server) MUST | Boundary `ntime` submissions |
| G5 | Ω3 | Θ2 | Duplicate share counted twice — no dedup on `(job_id, nonce, ntime, version, extranonce)` | Expected pool behavior (spec-permissive; see caveats) | Resubmit identical valid share; assert error or unchanged accounting |
| G6 | Ω2 | Θ1 | Share for unknown/closed/never-opened `channel_id` or `job_id` causes crash, silence, or acceptance | [§5.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#52-channel) channel/job scoping | Probe with bogus IDs |
| G7 | Ω1 | Θ2 | `SubmitShares.Success` batch semantics broken: `last_sequence_number` ≠ client's last accepted seq; `new_submits_accepted_count` ≠ batch size; `new_shares_sum` ≠ sum of batch share difficulties; counters not reset between batches | [§5.3.13](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5313-submitsharessuccess-server---client) "their respective counters MUST be reset when a new batch starts being processed" | Submit known-difficulty shares in controlled batches; verify all three fields arithmetically |
| G8 | Ω2 | Θ2 | `.Success` acknowledging shares never submitted (`last_sequence_number` ahead of what the client sent) | [§5.3.13](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5313-submitsharessuccess-server---client) | Sequence audit |
| G9 | Ω1 | Θ1 | Errors silently dropped: incorrect submit never gets a `SubmitShares.Error` | [§5.3.14](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5314-submitshareserror-server---client) "An error is immediately submitted for every incorrect submit attempt" | Timeout assertion on every bad share |
| G10 | Ω2 | Θ1 | `SubmitShares.Error` referencing the wrong `sequence_number` (cannot be paired with the offending submit) | [§5.3.14](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5314-submitshareserror-server---client) | Pairing assertion |
| G11 | Ω1 | Θ1 | `SubmitSharesExtended` on a standard channel (or `SubmitSharesStandard` on an extended channel) mishandled | [§5.3.12](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5312-submitsharesextended-client---server) "Only relevant for extended channels" | Wrong-type probe |

### H. Channel lifecycle ([§5.3.9](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#539-closechannel-client---server-server---client)–[§5.3.10](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5310-setextranonceprefix-server---client)

| Scenario IDs | Compliance tier | Testability tier | Failure mode | Spec basis | Detection |
|--------------|---------------|----------------|---|---|---|
| H1 | Ω1 | Θ1 | Server keeps sending jobs/`SetTarget` for a channel after `CloseChannel` | [§5.3.9](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#539-closechannel-client---server-server---client) "The server MUST stop sending messages for the channel" | Close channel; assert silence |
| H2 | Ω3 | Θ1 | Server-initiated `CloseChannel` mid-session without `reason_code`; client cannot re-open cleanly | [§5.3.9](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#539-closechannel-client---server-server---client) | Resilience test |
| H3 | Ω1 | Θ2 | `SetExtranoncePrefix` applied retroactively to jobs sent before the change (validation mismatch) | [§5.3.10](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5310-setextranonceprefix-server---client) "applicable for all jobs sent after this message" | Submit share on a pre-change job with the old prefix; assert acceptance |
| H4 | Ω3 | Θ1 | `CloseChannel` addressing a group channel doesn't close member channels (or closes unrelated ones) | [§5.3.9](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#539-closechannel-client---server-server---client) | Group topology probe (only if pool uses groups) |

---

## Caveats

- **G5 (duplicate-share rejection)** is an area where the spec is deliberately permissive; the test
  asserts *expected pool behavior*, not a spec MUST. The suite should label it as such to avoid
  false "non-compliant" verdicts.
- **E2 (stale-share rejection)** has accounting rather than consensus consequences, but stale
  acceptance silently inflates reported hashrate, so it is ranked Ω1.

---

## Suite Coverage

✅ = covered; ❌ = not covered

| Scenario IDs | Solo Pool |
|---|---|
| A1 — `channel_msg` bit correctness | ❌ |
| A2 — `extension_type` on core messages | ❌ |
| A3 — error code control characters | ❌ |
| A4 — error code non-printable non-control bytes | ❌ |
| A5 — no messages before `SetupConnection` response | ❌ |
| A6 — `channel_id` already closed | ❌ |
| A7 — `channel_id` never opened | ❌ |
| B1 — `SetupConnection` response type / TCP close vs framed error | ❌ |
| B2 — unsupported or invalid `protocol` value | ❌ |
| B3 — `used_version` within negotiated range | ❌ |
| B4 — feature-flag probing missing/mismatched error flags | ❌ |
| B5 — `REQUIRES_FIXED_VERSION` vs `REQUIRES_VERSION_ROLLING` cross-check | ❌ |
| B6 — `REQUIRES_FIXED_VERSION` + `version_rolling_allowed` consistency | ❌ |
| B7 — `REQUIRES_EXTENDED_CHANNELS` flag coherence | ❌ |
| B8 — `REQUIRES_STANDARD_JOBS` must be accepted | ❌ |
| B9 — Extended channel support | ❌ |
| B10 — rejects empty `device_id` / crashes on boundary strings | ❌ |
| C1 — `request_id` echo in channel open success | ❌ |
| C2 — target vs `max_target` on channel open | ❌ |
| C3 — `channel_id` uniqueness | ❌ |
| C4 — `extranonce_prefix` uniqueness | ❌ |
| C5 — `extranonce_size` vs `min_extranonce_size` | ❌ |
| C6 — same group, different full extranonce sizes | ❌ |
| C7 — first message after channel open is a job | ❌ |
| C8 — first job `min_ntime` unset | ❌ |
| D1 — channel-kind vs message-type cross-check | ❌ |
| D2 — `job_id` uniqueness and pairing | ❌ |
| D3 — `NewExtendedMiningJob` vs `REQUIRES_STANDARD_JOBS` | ❌ |
| D4 — BIP141 stripping in coinbase tx data | ❌ |
| D5 — version-rolling share acceptance (BIP323 bits) | ❌ |
| D6 — `nTime`-rolling share acceptance (protocol window) | ❌ |
| D7 — job starvation (liveness) | ❌ |
| E1 — `SetNewPrevHash` references known future `job_id` | ❌ |
| E2 — stale-share rejection | ❌ |
| E3 — `min_ntime` sanity bounds | ❌ |
| F1 — `SetTarget` bound vs client `max_target` | ❌ |
| F2 — `SetTarget` non-retroactivity | ❌ |
| F3 — `UpdateChannel` target reduction honored | ❌ |
| G1 — valid-share acceptance | ❌ |
| G2 — invalid-share rejection | ❌ |
| G3 — extranonce length validation | ❌ |
| G4 — `ntime` bounds enforcement | ❌ |
| G5 — duplicate-share rejection | ❌ |
| G6 — bogus `channel_id`/`job_id` in share submission | ❌ |
| G7 — `SubmitShares.Success` batch accounting | ❌ |
| G8 — `last_sequence_number` audit | ❌ |
| G9 — `SubmitShares.Error` always sent for bad submits | ❌ |
| G10 — `SubmitShares.Error` sequence_number pairing | ❌ |
| G11 — wrong share-submit message type per channel kind | ❌ |
| H1 — silence after `CloseChannel` | ❌ |
| H2 — server-initiated `CloseChannel` with reason_code | ❌ |
| H3 — `SetExtranoncePrefix` non-retroactivity | ❌ |
| H4 — group-channel `CloseChannel` correctness | ❌ |

Note: more columns will be added as the project matures.
