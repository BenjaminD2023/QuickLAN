# Pinned upstream capabilities — stock baseline and QuickLAN patch


## Current QuickLAN 0.2.0 integration

The table below records the original stock-core findings. The release engine now
builds the exact pinned source with `upstream/quicklan.patch`, using a separate
GPL-3.0-only process linked to the still-LGPL EasyTier library. It never starts the
stock TCP RPC server. Native Unix sockets / Windows named pipes replace management
RPC; the wrapper embeds the staged helper digest. No stock executable is elevated.

The patch fixes DHCP to the invitation prefix, rejects legacy secure-mode
handshakes, scopes packets and advertisements, disables implicit STUN/DNS fallback/
external-IP probes, prevents broad Windows firewall/profile edits and excludes
unneeded proxies/Windows capture dependencies. Direct-only is enforced on final
send and inbound relay paths. Windows uses official Wintun only; Npcap/WinDivert
are not required. The selected dependency graph is locked separately in
`engine/Cargo.lock`, with zero vulnerability-class findings in the recorded scan.

Native adapter/IPC/cleanup checks pass on Windows and both Mac architectures.
QuickLAN direct and forced-relay TCP/UDP tests, wrong credential, restart and
direct-only path-loss checks pass in isolated Linux stacks (run 34041033461).
These replace the original implementation gaps below; the older evidence is kept
as a review trail and is not the current feature-status table. Physical-device,
ordinary-user elevation and internet traversal evidence remains incomplete.
See NATIVE_ENGINE.md, FEATURES.md and TEST_MATRIX.md. Each binary release supplies
exact corresponding source and original notices; no upstream relicensing occurs.

## Original stock-core inspection (historical)

Inspected 2026-09-06. This is an evidence ledger, not a security audit or beta approval.

- Project: [EasyTier](https://github.com/EasyTier/EasyTier), unaffiliated with QuickLAN.
- Release: [v2.6.4](https://github.com/EasyTier/EasyTier/releases/tag/v2.6.4), 2026-05-12.
- Commit: `8428a89d2dabc94c97d370ec607c6ca142473626` (tag resolved with `git ls-remote`).
- Native ARM64 execution: `easytier-core 2.6.4-8428a89d`.
- Immutable pin manifest: [easytier.lock.json](../upstream/easytier.lock.json). ZIP digests came from the GitHub release API; ARM64 ZIP and extracted core/CLI digests were independently computed locally. Intel Mac and Windows x64 ZIPs and extracted binaries were also verified locally; those binaries have not been executed. Source tar digest was computed locally.
- Main source license is **LGPL-3.0**, not the documentation website's Apache footer. Exact text is retained in `licenses/EasyTier-LGPL-3.0.txt`, together with GPL-3.0. Original QuickLAN code is Apache-2.0. No upstream code is linked into the wrapper. A separate process is the intended integration.
- Shipping core binaries requires a corresponding-source distribution, dependency license inventory, notices and installation/rebuild instructions. A link alone is not our release compliance strategy. Public core redistribution is gated until these are checked. No upstream changes have been shipped.
- Upstream artifacts exist for macOS aarch64, macOS x86_64 and Windows x86_64 (and additional architectures outside launch scope). ARM64 Mach-O reports minimum macOS 11.0. QuickLAN targets macOS 13+ for modern service integration and Windows 11 x64; these are **candidate baselines**, not claims of verified support. Only this macOS 26.4.1 ARM64 host is available.

Source paths below are relative to the exact commit, accessible under [the source tree](https://github.com/EasyTier/EasyTier/tree/8428a89d2dabc94c97d370ec607c6ca142473626).

| Capability | Exact evidence | Decision / remaining verification |
|---|---|---|
| Configuration | `easytier/src/common/config.rs` private `Config`, `core.rs` CLI merge; TOML, `--config-file`, `--check-config`, `--disable-env-parsing` | Generate a narrow schema in Rust; never accept raw TOML/flags from UI/invites. Core info logs print entire config: discard upstream stdout/stderr, never persist them. |
| Management | `rpc_service/api.rs`, `instance/instance.rs::InstanceRpcServerHook`, `rpc_service/instance_manage.rs`; `easytier-cli -o json node/peer/route` | Loopback TCP RPC allows configuration/lifecycle mutation and authenticates by **IP whitelist only**. No verified protected IPC/token for stock core. **Do not launch stock core elevated.** A protected helper around an exposed core RPC would not solve this. |
| Virtual interface | `instance/virtual_nic.rs`, pinned `tun-easytier` dependency | OS TUN/utun/Wintun, no custom driver. Requires privilege/integration checks. No local TUN proof yet. |
| No-TUN integration | `gateway/tcp_proxy.rs`, `gateway/udp_proxy.rs`, port forwards in `common/config.rs` | Real userspace virtual-address traffic can be exercised without host routes; this is a developer test, not the game-by-IP product. |
| Address allocation | `common/config.rs`, `instance/virtual_nic.rs` DHCP; CLI warns of address changes on conflict | RFC1918 canonical /24 only in v1 invitation proposal. Every member must retain the same prefix. Check collisions before starting. DHCP/malicious announcements need controlled tests. |
| IPv4 routes | `instance/proxy_cidrs_monitor.rs::diff_proxy_cidrs` | Explicit `routes=[]` overrides learned subnet routes for OS route installation. Not proof that malicious advertisements are rejected in the whole data plane. IPv6/overlay-IP spoofing still require audit/tests. |
| DNS, exit/subnet routing | default flags plus explicit `accept_dns=false`, `enable_exit_node=false`, `proxy_forward_by_system=false`, `exit_nodes=[]`, no proxy networks/VPN portal | No DNS/default-route/firewall modification is permitted by the wrapper. Confirm before/after privileged runs. |
| Unrelated forwarding | `peers/foreign_network_manager.rs`; `relay_network_whitelist=""`; `disable_relay_data=true` receive path | Disables data transit, while some control forwarding remains. Applies to in-group transit too. Test against malicious foreign peers before enablement. |
| Own traffic through relays | `peers/peer_manager.rs::check_p2p_only_before_send` / `send_msg_internal` | **Different from forwarding.** `p2p_only` checks direct presence then has asynchronous routing/fallback paths; inbound relayed payload rejection and migration race not proven. Direct-only mode disabled at QuickLAN validation layer. |
| Private mode | `peers/peer_manager.rs:769`, upstream `peers/tests.rs` | Rejects untrusted foreign network servers too; cannot combine blindly with public bootstrap. Manual profile may set it; assisted profile must not assume it works with arbitrary public nodes. |
| Traversal | `connector/direct.rs`, `hole_punch/*`, `common/stun.rs` | TCP/UDP and IPv6 support exists. Same-machine tests prove none of internet NAT traversal. No universal connectivity claim. |
| Public discovery | `common/global_ctx.rs:290`, `common/stun.rs:1096-1117` | **Release gap:** `stun_servers=[]` overrides UDP only; default TCP STUN list remains. Locally observed nonempty public IP with both configured lists empty. Stock release cannot support the promised no-public profile merely by these flags. Lab uses OS outbound confinement. Production connections remain gated. |
| Encryption | `peers/encrypt/mod.rs`, `common/global_ctx.rs::get_128_key/get_256_key` | Legacy shared-key derivation uses `DefaultHasher`; do not make modern KDF/forward secrecy claims. Never select XOR or disable encryption. |
| Secure mode | `peers/peer_conn.rs` uses `Noise_XX_25519_ChaChaPoly_SHA256`; HMAC-SHA256 secret proof in `common/global_ctx.rs` | `--secure-mode true` generates/processes upstream keys. Merely writing `[secure_mode] enabled=true` without keys fails at runtime. Server can accept legacy handshakes; downgrade/replay/end-to-end relay properties are not fully established. No independently verified device identity claim. |
| Metrics | `easytier-cli.rs::handle_peer_list`, node and route JSON | Table-derived JSON uses strings and `-` for unavailable metrics. Parse exact schema, explicit unknown, no unavailable-to-zero conversion. Table JSON `cost="p2p"` indicates an observed direct route; no fabricated peers/latency. |
| Broadcast | v2.6.4 release notes and `enable_udp_broadcast_relay` | Disabled. No universal LAN-game discovery, Ethernet bridge, multicast or mDNS support claim. |

## Primary documentation checked

- [Quick networking](https://easytier.cn/en/guide/network/quick-networking.html): a shared bootstrap choice is required; do not copy its suggestion to disable firewalls.
- [P2P optimization](https://easytier.cn/en/guide/network/p2p-optimize.html), [shared nodes](https://easytier.cn/en/guide/network/host-public-server.html).
- [Tauri sidecars](https://v2.tauri.app/develop/sidecar/) and [security](https://v2.tauri.app/security/): bundling is not privilege elevation.
- [Tauri macOS signing](https://v2.tauri.app/distribute/sign/macos/), [Windows signing](https://v2.tauri.app/distribute/sign/windows/), [Apple SMAppService](https://developer.apple.com/documentation/servicemanagement/smappservice).

A checksum establishes equality to a recorded artifact, not publisher identity, reproducibility or absence of vulnerabilities. Stable version strings alone do not authenticate executables. All public beta gates remain subject to TEST_MATRIX.md.
