# Test matrix

Recorded 2026-09-06. **Not a beta release.** “Implemented” describes code, “local” describes execution here, “manual” describes direct inspection, and “CI” requires an actual hosted run. No hosted CI runs have occurred. No test in this ledger proves multi-machine TUN connectivity.

Local environment: Apple Silicon, macOS 26.4.1, Rust 1.96.0, Node 22.22.3, npm 10.9.8, Python 3.9, Xcode. Elevated execution unavailable (`sudo -n` requires a password). No Windows machine, second Mac, Intel execution environment, signing identity or controlled remote NAT/relay infrastructure supplied.

| Feature / gate | Implemented | Locally tested | CI tested | Manually verified | Evidence / limitation |
|---|---|---|---|---|---|
| Core release and native ARM binary pin | Yes | Pass | No | Source/CLI inspected | `upstream/easytier.lock.json`, `evidence/artifact-macos-aarch64.json` |
| Intel Mac and Windows x64 core archive integrity | Yes | Pass, hashes only | No | Not executed | `evidence/artifact-{macos-x86_64,windows-x86_64}.json` |
| Actual TCP and UDP payloads, both directions | Lab only | Pass, TCP and UDP underlays | No | Results inspected | `python3 scripts/core-integration.py [--transport udp]`, `evidence/core-integration-{tcp,udp}.json`; userspace port-forwarded virtual addresses, localhost-confined four-process lab |
| Correct secret, wrong secret, separate-network isolation, restart | Lab only | Pass | No | Recorded output inspected | Same integration records; no physical devices or TUN |
| Invitation round trip, versions, size, fields, endpoints, consent | Yes | Pass | No | UI preview inspected | `cargo test --all-features --locked`; 23 tests pass, including 512-case property runs for each fuzz target |
| Secret redaction and diagnostic projection | Yes | Pass | No | UI preview inspected | Rust tests plus five frontend trust-boundary tests; no raw core output retained |
| OS metadata permissions, atomic replacement, symlink rejection | macOS implementation | Pass | No | Not crash-injected | Rust tests; future schemas rejected. Cross-store crash recovery remains incomplete |
| Actual OS credential write/read/delete | Yes | Pass on native Mac | No | OS-backed test executed | `cargo test --all-features --locked --test security native_credential_round_trip -- --ignored`; one passed; isolated random entry deleted |
| Windows credential storage | Uses native Keyring backend | Not verified | No | No | Needs native Credential Manager protection/ACL/recovery tests |
| Lifecycle serialization and stale-generation rejection | Domain implementation | Pass | No | No live engine integration | Rust tests, single native mutex; no production engine process exists |
| Safe subnet, overlap and forbidden learned-route validators | Domain implementation | Pass | No | No OS enforcement | Saved-network collisions tested; live route enumeration and core data-plane rejection missing |
| Missing helper / disabled policy failure | Yes, denies connection | Pass | No | UI inspected | Rust tests; no fake permission prompt or repair success |
| Helper peer authentication and bounded privileged IPC | Schema only | Malformed command rejection | No | No | OS service, peer credentials/signature binding, installer and protected core IPC **not implemented** |
| Create/import/invite/settings/forget UI | Yes | 3 Playwright workflow tests pass | No | In-app browser inspected | `npm run test:e2e`; explicit labeled test adapter; does not validate native IPC |
| Keyboard, focus, dark theme, narrow layout | Yes | Pass | No | Screenshots inspected | `evidence/ui-*.png`, `docs/DESIGN_QA.md`; no browser overflow at 390px |
| Production cannot use UI test adapter | Yes | Pass | No | Production bundle scanned | `npm run build && npm test`; test hook and banner absent from generated JS |
| Frontend type/lint/build and Rust domain Clippy | Yes | Pass | No | No | README commands, `-D warnings` |
| Desktop ARM64 compile / bundle | Yes | Pass, optimized app bundle | No | Native create/copy/error/relaunch/rename/forget and theme verified | `evidence/native-desktop.json`; compilation alone is not end-user compatibility |
| macOS Intel / Windows installer builds | CI configured | Not verified | No | No | `.github/workflows/ci.yml`; not run on hosted runners |
| npm / wrapper Rust vulnerability checks | Yes | Executed | No | Advisories reviewed | `evidence/*audit.json`, `docs/SECURITY_REVIEW.md`; distinguish warnings from vulnerability count |
| SBOM and dependency notices | Generated | Inventory checked | No | Incomplete legal review | `evidence/dependencies.cdx.json`, `evidence/license-inventory.json`, `licenses/`; missing texts gate public redistribution |
| OS virtual-IP TCP/UDP between ordinary devices | No | Not verified | No | No | Required macOS/macOS, Windows/Windows and mixed OS pairs on real physical networks |
| Direct-only, forced relay, migration, outage, reconnect paths | Direct-only disabled | Not verified | No | No | Packet-level proof required at endpoints and controlled relay, both directions |
| No-public assistance | Disabled | Stock violation observed | No | Source inspected | Separate implicit TCP STUN remains; lab OS confinement is not a production solution |
| NAT traversal, blocked UDP, IPv6, MTU, VPN overlap | No end-to-end integration | Not verified | No | No | Controlled topology matrix required |
| Sleep/wake, Wi-Fi change, abrupt helper death | No end-to-end integration | Not verified | No | No | Must verify route ownership and cleanup after crashes |
| Installation, elevation refusal, repair, upgrade, uninstall | Preview package only | Not verified | No | No | Requires native machines and privileged tests; Windows hosted runners' disabled UAC cannot prove consent behavior |
| Route/DNS/firewall/internet preservation | No mutations made by preview | Lab creates none | No | No privileged run | Snapshot-and-diff procedure below is required for helper integration |
| Startup, idle memory/CPU, throughput, loss and latency benchmarks | No | Not measured | No | No | Lab wall-clock duration is not a benchmark |

## Executable lab and platform procedure

Run the README checks in a disposable checkout. Core lab fixtures contain only ephemeral peer metadata; credentials and raw configs stay in private temporary directories and are removed. To exercise host applications, implement and validate the protected helper first—do not elevate the stock RPC endpoint.

On each macOS test device, capture before/after state to private files with `netstat -rn`, `scutil --dns`, `ifconfig`, and `route -n get default`. On Windows use PowerShell `Get-NetRoute`, `Get-DnsClientServerAddress`, `Get-NetAdapter`, and `Get-NetFirewallRule`. Record only sanitized diffs. Verify ordinary HTTPS still works. Treat all unrelated changes as failures, including after termination/uninstall.

After the service gate is cleared, assign agreed overlay addresses through QuickLAN. On each machine start the Python service probe in `scripts/service-probe.py` bound to its real assigned overlay address. Run its client mode from the other machine with both TCP and UDP, then reverse roles. A successful localhost lab is not a substitute. Do not automatically open firewall ports; explicitly permit only the chosen test port and overlay scope on consenting test machines.

The probe itself passed a localhost TCP/UDP echo check using its explicit `--loopback-lab` flag (`evidence/service-probe.json`). That verifies the diagnostic utility only. Example real-overlay commands after helper validation: `python3 scripts/service-probe.py server --address ACTUAL_OVERLAY_IP --seconds 60` on the host, and `python3 scripts/service-probe.py client --address ACTUAL_OVERLAY_IP` on the other peer. Replace the address with the real assigned address; the script does not create one.

Repeat over separate physical networks with ordinary NAT, blocked UDP and reachable IPv6. For each topology exercise wrong secrets, separate groups, coordinated subnet collision, forced relay, bootstrap disappearance, migration, sleep/wake and interface switch. For direct-only, capture at both endpoints and the controlled relay, assert no application payload traverses any intermediate peer during establishment, failure and reconnect. Keep allowed discovery control traffic distinct.

Refuse privilege, remove/corrupt the helper, alter the pinned core, crash each process, restart repeatedly, and attempt a second connection concurrently. Verify truthful states, no duplicate engine, no stale routes and safe teardown. Install/repair/upgrade/uninstall on all three native architectures before beta. Publish only sanitized test reports and measured benchmark conditions.
