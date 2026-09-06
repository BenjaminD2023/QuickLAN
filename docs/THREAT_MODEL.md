# Threat model

Assets: bearer network credentials, OS routing/DNS/firewall integrity, the user's listening applications, executable/helper integrity, private device/network metadata and diagnostic contents.

| Adversary | Risk | Implemented boundary | Remaining gate |
|---|---|---|---|
| Outsider without invitation | Guess network/access application traffic | 128-bit random identifier, 256-bit random credential; real wrong-secret lab exclusion | Exact core authentication/downgrade/replay audit and hostile network testing |
| Leaked invitation holder | Joins or shares the group | Honest bearer warning, OS vault, local forget, new-network migration | No per-device revocation, expiration, owner approval or single-use promise |
| Untrusted assisting node | Learns metadata, blocks traffic, attempts downgrade | No built-in nodes, explicit custom operator/consent; no default analytics | End-to-end relay encryption properties and public policy approval; system connection gated |
| Malicious group member | Spoofs name/address, advertises routes, probes services | Names/IDs never marked verified, strict private subnet proposal, no exit/subnet configuration in wrapper | Core-side malicious route/IP/identity enforcement, scoped firewall tests |
| Other local user or hostile same-user process | Controls elevated engine or reads secrets | UI unprivileged; no elevated helper launched; OS vault; command allowlist; no shell/remote content | Signed/audit-token-authorized OS IPC and authenticated core management. Loopback IP alone is insufficient |
| Malformed invitation | Command/config injection, excessive allocation, unwanted fetch | Exact schema, byte/field limits, endpoint allowlist, no raw flags/config URLs, preview bound to acceptance, property tests | Fuzzer runtime budgets and corpus expansion |
| Compromised update/dependency | Malicious executable or secret exfiltration | Lockfiles, pinned artifact digests, manual updates only; no updater endpoint | Signed releases, provenance verification, reproducibility and corresponding-source/license review |
| Crash/partial storage operation | Invisible engine, stale routes or unavailable secret | No background engine in this build, lifecycle generations, atomic metadata, best-effort rollback | Real resource ownership journal, platform crash recovery and full storage reconciliation |

Inherited properties must be described only after verifying the exact upstream version. The legacy core KDF, secure-mode legacy compatibility and relayed session handling have not received a complete audit. Secure-mode protocol strings in source are not a certification of system security. QuickLAN does not implement its own crypto, kernel driver or network handshake.

Diagnostics use a whitelist of fixed fields/codes rather than a broad “redact secrets” regex. Unknown native errors map to generic text. Raw core config/logs never enter diagnostic exports. The clipboard is a separate bearer-credential exposure; only explicit copy actions access it, and the UI warns about other apps/history. No automatic clipboard clearing or monitoring is performed.

The engineering app may be used to inspect local invitation workflows. **It must not be marketed as a working VPN or public beta.**
