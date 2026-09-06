# Feature status — 0.2.0 native preview

Real system networking is implemented. The release remains a native preview
because physical device pairs, ordinary-user privilege dialogs, minimum OS
versions and broader network/security acceptance are not fully verified.

| Capability | Implemented behavior | Evidence / limit |
|---|---|---|
| Saved networks and identities | Labels, device nicknames, create/join/invite/forget/replacement | Domain, UI and earlier native vault checks; names are not verified identities |
| Protected credentials | OS Keychain / Windows Credential Manager, secret-free metadata and diagnostics | Native roundtrips passed; cross-store crash recovery is best effort |
| Connect/disconnect | Bundled elevated helper, one active engine, real adapter/IP, shutdown acknowledgement | Native Windows, ARM/Intel Mac acceptance; current package checks tracked in TEST_MATRIX |
| Helper security | Both-end kernel PID authentication, private socket/pipe, fixed commands, bounded framing, hash verification and substitution defenses | Native IPC tests; no stock TCP management RPC |
| Permissions and repair | Per-connection OS elevation, denial error, bundled helper replacement by reinstall, no persistent service | Ordinary-user OS dialog and upgrade matrix remain unverified |
| Peer state | Live virtual IP, direct/relayed/unreachable path and measured direct latency | Actual Instance APIs; absent metrics remain absent |
| Reachability | One explicit bounded TCP port probe to a current peer | Real listener/refusal unit tests and UI workflows; UDP apps need application tests |
| No public assistance | Explicit IP endpoints, no implicit STUN/public endpoint/external-IP probe/DNS fallback | Patched core and isolated stacks with no public route |
| P2P preferred, relay permitted | Custom operator-approved node and consent, actual relay state | Forced-relay TCP/UDP underlay tests pass |
| Direct-only application traffic | Final-send and inbound enforcement, no relay data fallback | Both-direction TCP/UDP pass directly, then fail when only the relay remains |
| Route restrictions | Fixed private /24, startup overlap checks, hostile advertised subnet rejection and packet scope | Domain/packet tests and real owned-adapter cleanup; later VPN route changes remain unverified |
| Forwarding restrictions | No foreign or in-group application transit, exit node, subnet proxy, default route, DNS change or broad firewall rule | Source enforcement and isolated/native checks; full hostile-peer audit outstanding |
| Local setup | User-selected OS Wi-Fi/Ethernet endpoint in create flow | UI test preserves choice; remote NAT discovery is not automatic |
| Failure recovery | Controller EOF/heartbeat cleanup, restart, wrong-secret rejection, stale observation suppression | Native and isolated integration tests |
| Interface | English/Chinese, light/dark, keyboard, responsive layout, preferences/help/about | Five browser workflows; native GUI inspection is separately recorded |
| Packaging | ARM/Intel DMG and Windows x64 EXE with engine and notices | Native CI; ad-hoc Mac integrity only, unsigned Windows, no notarization |
| Source and maintenance | Public source, locked/patched engine, SBOM/notices, corresponding-source recipe, scans | No independent audit, reproducible-build or performance claim |

There is no built-in public service. An invitation needs a shared reachable
endpoint. Compatible user-operated nodes are optional; arbitrary remote NATed
peers may fail. No account, subscriptions, chat, file sharing, telemetry,
automatic updater, startup auto-connect, tray background mode or LAN broadcast
emulation is included. These are scope decisions, not hidden paid features.

Controlled scale tested is two application peers plus a relay, not unlimited
practical capacity. Physical Windows/Mac combinations and realistic home/restrictive
NAT, IPv6, sleep/wake, MTU, Wi-Fi changes and concurrent VPN changes have executable
follow-up procedures in TEST_MATRIX but are not all verified.
