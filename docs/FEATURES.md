# Feature status

This is an engineering preview. **QuickLAN does not yet connect users into a system virtual network.** Passing packaging and core feasibility tests does not complete the missing application/helper integration. The original product brief is not fully implemented.

| Requested capability | Current implementation | Remaining work |
|---|---|---|
| Saved networks, labels, device nickname | Implemented; native Mac workflows and domain/UI tests pass | Broader Windows native workflow acceptance |
| Create, preview/join, copy invitation, forget, replacement credentials | Implemented with validated versioned tokens and explicit preview consent | Real network join; replacement does not revoke the old group |
| Secret storage | OS Keychain / Windows Credential Manager; real roundtrips pass | Cross-store crash recovery and additional Windows ACL/recovery checks |
| Diagnostics, preferences, help, about/licenses | Implemented; sanitized projection and local preferences | Live core health and actual application-port reachability integration |
| Light/dark, keyboard navigation, responsive layout | Implemented with UI automation and native Mac inspection | Full accessibility audit; Chinese translation remains partial |
| Connect/disconnect and one active network | Lifecycle/serialization model implemented; Connect truthfully fails for missing helper | Authenticated helper, actual engine lifecycle and service-state reconciliation |
| Virtual IP, peer path and measured latency | Version-pinned adapter/parser and UI prepared | Live authenticated core state; no production peers or metrics are fabricated |
| Windows/macOS helper installation, permission refusal, repair, removal | Typed request validation only | Actual OS services, peer authentication, protected IPC, verified executables and installers |
| Route/address safety | Private-subnet, saved-network collision and forbidden-route validators | Live route inspection, data-plane enforcement, coordinated migration and owned-resource recovery |
| P2P preferred with relay consent | Saved policy and custom endpoints | Production enforcement, truthful mixed paths and controlled relay tests |
| Direct-only application traffic | Disabled with explicit gap | Packet-level enforcement through migration/reconnect, both directions |
| No public assistance | Production core launch disabled | Eliminate implicit upstream public TCP STUN and verify no public egress without lab confinement |
| No transit/exit/subnet forwarding | Restricted adapter configuration designed and source reviewed | Hostile-peer and OS data-plane proof; no public nodes are bundled |
| Real TCP/UDP through virtual addresses | Upstream core feasibility passes through real Linux TUN devices, TCP and UDP underlays, both directions | QuickLAN helper integration, supported desktop OS pairs and different physical networks |
| Failures, cleanup and recovery | Model tests; Linux lab abrupt core stop/restart and owned cleanup pass | Installed desktop crash, sleep/wake, interface change, firewall refusal, MTU and VPN overlap matrix |
| Native packaging | Apple Silicon/Intel DMGs and Windows x64 EXE built in passing native CI; Windows installer smoke passed | Signed/notarized delivery; ordinary-user, upgrade, repair, minimum-OS and helper acceptance tests |
| Source, notices, SBOM, security checks | Public GitHub source; pinned dependencies; private vulnerability reporting; wrapper scans pass with documented warnings | Pinned upstream vulnerability remediation, complete core distribution materials before bundling core, security review |
| Performance and internet traversal | No performance or NAT success claims | Controlled measured benchmarks and realistic NAT/relay topologies |

## Verified without a second physical device

[Native CI](https://github.com/BenjaminD2023/QuickLAN/actions/runs/34031905272) passed all three platform jobs and wrapper advisory/secret scans. The Windows runner installed the actual EXE, opened a native QuickLAN window, closed it gracefully and uninstalled it with routes/DNS unchanged. This is Windows Server CI evidence, not Windows 11 ordinary-user or UAC testing.

[Networking feasibility CI](https://github.com/BenjaminD2023/QuickLAN/actions/runs/34032344518) ran two isolated Linux kernel network stacks linked only to each other. Each received a real TUN interface and virtual IPv4 address. Both directions passed exact TCP and UDP payload checks, remote core death/restart and cleanup. The host routes and DNS were unchanged. No public egress was possible. The lab denies all IPv4 management callers; this is not a production helper authentication solution.

These tests establish that the chosen core can provide real virtual-IP transport in the tested topology. They do not establish that every requested product feature is implemented or safe. See [TEST_MATRIX.md](TEST_MATRIX.md) for the full evidence and executable follow-up procedures.
