# Threat model

Assets: network bearer credentials; routing, DNS and firewall integrity; listening
applications; helper integrity; private metadata; and diagnostic contents.

| Adversary | Implemented boundary | Limits / remaining evidence |
|---|---|---|
| Outsider without invitation | 128-bit random identifier, 256-bit credential; upstream Secure Mode, legacy handshake rejected | Wrong-secret joins fail in real direct/relay tests; no independent protocol/replay audit |
| Credential holder | Explicit bearer semantics, vault storage, replacement-network workflow | Can share credential and probe permitted services; no per-device revocation/expiry/owner approval |
| Assisting operator | Explicit node/consent, no built-ins, encrypted core transport | Operator sees IP/timing/volume and can deny service; operator key and full end-to-end relay properties are not independently audited |
| Malicious group member | No identity badges; fixed private /24; packet source/destination scope and proxy-advertisement rejection | Does not enforce honest peer names or prevent all within-subnet spoofing; keep application authentication/firewalls |
| Hostile local process | Unprivileged GUI, kernel peer PID checks, private IPC, fixed commands, bounded frames, Mac root staging and Windows held file/directory handles | OS vault does not protect against a fully compromised same-user session/admin; publisher identity is not established by an embedded hash |
| Malformed invitation/IPC | Strict schema, size/field/endpoint limits, no raw flags, preview-bound acceptance, timeouts | Unit/property and native tests do not prove all parsers/boundaries bug-free |
| Compromised dependency/update | Pinned locks/source hashes/patch, vulnerability scans, no automatic updater | Checksums are not trusted publisher signatures; independent audit and signed release credentials absent |
| Crash/controller loss | Engine supervision, heartbeat/EOF termination, OS-owned ephemeral adapter lifecycle, shutdown acknowledgement | Hard-killing both Mac guardian and engine can leave inert root-owned staging files; power-loss and every recovery topology unverified |
| Storage interruption | Private atomic metadata and best-effort vault rollback | Separate stores are not transactional; an orphan secret/unavailable saved entry may need re-import |

QuickLAN reuses EasyTier's Noise_XX_25519_ChaChaPoly_SHA256 secure transport and
HMAC-SHA256 credential proof; it does not implement custom crypto or a new driver.
Exact secure-mode key setup is performed through the pinned core API and the
patch rejects legacy handshakes. This is not a certification of forward secrecy,
replay resistance, peer identity or end-to-end properties in every relay topology.

The packet boundary prevents out-of-scope IPv4, IPv6 and broadcast delivery.
Direct-only rejects relayed application packets at the final send decision and
receive boundary, rather than reacting to UI path state. Controlled migration
checks passed with relay control still available. Ordinary peers cannot act as
application transit relays through QuickLAN's configured data plane.

No stock EasyTier management TCP server, IP proxy, DNS service, default route,
exit node, UPnP mapping or broad firewall exception starts. Existing overlapping
routes fail before adapter creation. Subsequent unrelated VPN route mutations
remain an unverified scenario; disconnect before changing those configurations.

Raw core logs are discarded at their source. Diagnostics use fixed fields/codes
and a bounded memory ring. Only explicit copy actions access the clipboard;
clipboard history or other applications may retain credentials. This is a native
preview with real networking, not a production-ready or independently audited VPN.
