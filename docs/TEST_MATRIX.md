# Test matrix

## Current 0.2.1 firewall release

[Native run 34082162974](https://github.com/BenjaminD2023/QuickLAN/actions/runs/34082162974)
passes all four jobs at `eaf7c02aa825d9d9ccf507af7ff73a05bc2907c5`.
[Security checks](https://github.com/BenjaminD2023/QuickLAN/actions/runs/34082162971)
also pass. The exact corresponding-source export passes a locked offline engine
release build. See [native evidence](evidence/native-021.json) and
[public release checksums](evidence/github-release-021.json).

| Check | Result | Scope |
|---|---|---|
| Windows firewall runtime | Pass | All three effective profiles: enable, disable, enable; confirmation refusal; original settings restored |
| macOS firewall runtime, ARM and Intel | Pass | Actual application firewall transitions; original global/block-all settings restored |
| Native adapters and packaged helper | Pass on all desktop targets | Actual launcher, IPC, connect/stop and cleanup; Windows install/window/uninstall; both mounted DMGs |
| Real virtual-IP traffic | Pass on isolated Linux stacks | TCP/UDP, forced relay, wrong credentials, restart, cleanup and direct-only path loss |
| Browser workflows | Seven pass on each desktop job | Explicit test adapter; includes confirmation/cancel, mixed state, disable/enable, denial and unreadable firewall state |
| Release files | All ten public assets verified | Original per-target BUILD metadata; SHA-256 and size match; source and notices included |

The original Windows BUILD dirty-checkout flag is retained. Its six pinned input
hashes match a clean CRLF checkout; no signed provenance or reproducible-binary
claim is made. Physical OS pairs, ordinary-user permission dialogs, managed policy
combinations, arbitrary NAT and publisher signing remain unverified.

The acceptance harness restores macOS block-all mode before restoring a disabled
global state. Its delayed-frame IPC test holds the client open through OS identity
authentication and the server acknowledgement, avoiding a premature test-peer exit.


## Previous 0.2.0 native implementation

The original 0.1.0 matrix is retained below as historical evidence, not current
feature status. The following checks execute QuickLAN's new engine, not a stock
core feasibility substitute. The user has no second physical device.

| Check | Local | Native/isolated CI | Limit |
|---|---|---|---|
| Domain, invitations, redaction, lifecycle, route and real TCP probe | Pass: 24 integration + 4 unit tests | Pass on native runners | One separate OS-vault test is opt-in |
| Unix kernel PID IPC and substitution-denying elevation script | Pass: 3 IPC + 2 runtime tests | Pass, actual production IPC | Interactive OS permission denial is not manually verified |
| Actual TUN/utun/Wintun, normal stop and controller-loss cleanup | No local sudo | Pass Windows x64 and both Macs in 34038650582; revised Macs pass 34041033461 | Not physical-device payload evidence |
| Direct TCP and UDP underlay, bidirectional virtual-IP TCP/UDP, wrong secret, restart and cleanup | No local sudo | Pass in 34041033461 | Linux namespaces with no public route |
| Forced relay over TCP and UDP underlay | No local sudo | Pass in 34041033461 | Two endpoint stacks plus a nonmember relay; IP forwarding disabled |
| Direct-only both directions through path loss | Packet guard unit passed | Pass in 34041033461: direct payload succeeds, then cannot flow with only relay control alive | Does not exhaust every routing race/topology |
| Mounted DMG helper digest, production-only mode, bundle integrity, real adapter/cleanup | CI only | ARM and Intel pass in 34041033461 | Ad-hoc integrity, not Developer ID/notarization |
| Installed Windows helper and DLL, adapter, native window, close/uninstall | CI only | Pass in 34042764316: installed helper, full launcher, adapter, window, close/uninstall | Earlier installed preview/window tests are separate evidence |
| UI local hosting, consent, create/join/settings, error, themes, diagnostics and live peer/probe | Five workflows pass; latest policy-control regression passes | Included in native workflow | Explicit labeled browser simulation, not native VPN acceptance |
| Selected engine vulnerabilities and notices | Zero vulnerability findings; five maintenance warnings; no missing license texts | Current audit workflow includes exact engine lock | Not an independent audit/legal review |
| Exact corresponding-source offline build | Locked offline release builds pass; exact code is b80cfd5 | Not a reproducible-build claim | Includes vendor source; toolchain/protoc are prerequisites |
| Physical Windows/Mac pairs, ordinary-user permission, minimum OS, NAT/IPv6/sleep/Wi-Fi/MTU/VPN changes | Unavailable/not verified | Not established by these runs | Follow the executable procedure below |

Earlier implementation evidence: [run 34041033461](https://github.com/BenjaminD2023/QuickLAN/actions/runs/34041033461),
[direct TCP](evidence/engine-integration-tcp.json), [direct UDP](evidence/engine-integration-udp.json),
[relay TCP](evidence/engine-integration-tcp-relay.json), [relay UDP](evidence/engine-integration-udp-relay.json).
The Windows packet-parser regression in that historical run was corrected.
[Final run 34042764316](https://github.com/BenjaminD2023/QuickLAN/actions/runs/34042764316)
passes all four jobs at code commit b80cfd5a9260e2ca71d87616115ef43873cd874c.
This run also exercises the actual desktop AppleScript/ShellExecute launcher,
two connect/stop cycles, duplicate-start refusal and startup cancellation on
Windows and both Macs, including packaged/installed helper resources. Native CI tests do not establish
physical cross-platform interoperability or universal internet traversal.

Reproduce using `.github/workflows/networking.yml` on disposable runners. Build
production engine, stage it, build `quicklan-runtime` example `engine_smoke`, and
run that driver elevated with CI=true. The Linux payload lab additionally builds
engine bins/examples with `--features lab` and runs `scripts/engine-integration.py`
for TCP/UDP, with and without `--relay-lab`. Lab binaries are refused by packaging.
Do not set CI=true to run destructive namespace tests on an everyday workstation.

## Historical 0.1.0 engineering preview


Recorded 2026-09-06. **Not a beta release.** “Implemented” describes code, “local” describes execution here, “manual” describes direct inspection, and “CI” requires an actual hosted run. Native CI and isolated Linux TUN feasibility CI passed; see linked records below. No test in this ledger proves connectivity between separate physical Windows/macOS devices.

Local environment: Apple Silicon, macOS 26.4.1, Rust 1.96.0, Node 22.22.3, npm 10.9.8, Python 3.9, Xcode. Elevated execution unavailable (`sudo -n` requires a password). No second physical device, signing identity or controlled remote NAT/relay infrastructure supplied. Native hosted runners now provide Windows Server 2022 and macOS 15 ARM64/Intel build environments.

| Feature / gate | Implemented | Locally tested | CI tested | Manually verified | Evidence / limitation |
|---|---|---|---|---|---|
| Core release and native ARM binary pin | Yes | Pass | No | Source/CLI inspected | `upstream/easytier.lock.json`, `evidence/artifact-macos-aarch64.json` |
| Intel Mac and Windows x64 core archive integrity | Yes | Pass, hashes only | No | Not executed | `evidence/artifact-{macos-x86_64,windows-x86_64}.json` |
| Actual TCP and UDP payloads, both directions | Lab only | Pass, TCP and UDP underlays | No | Results inspected | `python3 scripts/core-integration.py [--transport udp]`, `evidence/core-integration-{tcp,udp}.json`; userspace port-forwarded virtual addresses, localhost-confined four-process lab |
| Correct secret, wrong secret, separate-network isolation, restart | Lab only | Pass | No | Recorded output inspected | Same integration records; no physical devices or TUN |
| Invitation round trip, versions, size, fields, endpoints, consent | Yes | Pass | Pass | UI preview inspected | `cargo test --all-features --locked`; 23 tests pass, including 512-case property runs for each fuzz target |
| Secret redaction and diagnostic projection | Yes | Pass | Pass | UI preview inspected | Rust tests plus five frontend trust-boundary tests; no raw core output retained |
| OS metadata permissions, atomic replacement, symlink rejection | macOS implementation | Pass | No | Not crash-injected | Rust tests; future schemas rejected. Cross-store crash recovery remains incomplete |
| Actual OS credential write/read/delete | Yes | Pass on native Mac | No | OS-backed test executed | `cargo test --all-features --locked --test security native_credential_round_trip -- --ignored`; one passed; isolated random entry deleted |
| Windows credential storage | Native Keyring backend | No physical Windows device | Pass, actual roundtrip | CI result inspected | Native Credential Manager write/read/delete passed; additional ACL/recovery tests remain |
| Lifecycle serialization and stale-generation rejection | Domain implementation | Pass | Pass | No live engine integration | Rust tests, single native mutex; no production engine process exists |
| Safe subnet, overlap and forbidden learned-route validators | Domain implementation | Pass | Pass | No OS enforcement | Saved-network collisions tested; live route enumeration and core data-plane rejection missing |
| Missing helper / disabled policy failure | Yes, denies connection | Pass | Pass | UI inspected | Rust tests; no fake permission prompt or repair success |
| Helper peer authentication and bounded privileged IPC | Schema only | Malformed command rejection | No | No | OS service, peer credentials/signature binding, installer and protected core IPC **not implemented** |
| Create/import/invite/settings/forget UI | Yes | 3 Playwright workflow tests pass | Pass | In-app browser inspected | `npm run test:e2e`; explicit labeled test adapter; does not validate native IPC |
| Keyboard, focus, dark theme, narrow layout | Yes | Pass | Pass | Screenshots inspected | `evidence/ui-*.png`, `docs/DESIGN_QA.md`; no browser overflow at 390px |
| Production cannot use UI test adapter | Yes | Pass | Pass | Production bundle scanned | `npm run build && npm test`; test hook and banner absent from generated JS |
| Frontend type/lint/build and Rust domain Clippy | Yes | Pass | Pass | No | README commands, `-D warnings` |
| Desktop ARM64 compile / bundle | Yes | Pass, optimized app bundle | Pass | Native create/copy/error/relaunch/rename/forget and theme verified | `evidence/native-desktop.json`; compilation alone is not end-user compatibility |
| macOS Intel / Windows installer builds | Yes | Both downloaded DMGs verified | Pass on all three targets | Windows native window smoke | `evidence/native-ci.json`, `evidence/windows-installer-smoke.json`; no signing or ordinary-user consent claim |
| npm / wrapper Rust vulnerability checks | Yes | Executed | Pass | Advisories reviewed | `evidence/*audit.json`, `docs/SECURITY_REVIEW.md`; distinguish warnings from vulnerability count |
| SBOM and dependency notices | Generated | Inventory checked | No | Incomplete legal review | `evidence/dependencies.cdx.json`, `evidence/license-inventory.json`, `licenses/`; remaining missing texts are outside the three launch target normal graphs; platform SDK notices accompany installers |
| OS virtual-IP TCP/UDP between ordinary devices | No | Not verified | No | No | Required macOS/macOS, Windows/Windows and mixed OS pairs on real physical networks |
| Direct-only, forced relay, migration, outage, reconnect paths | Direct-only disabled | Not verified | No | No | Packet-level proof required at endpoints and controlled relay, both directions |
| No-public assistance | Disabled | Stock violation observed | No | Source inspected | Separate implicit TCP STUN remains; lab OS confinement is not a production solution |
| NAT traversal, blocked UDP, IPv6, MTU, VPN overlap | No end-to-end integration | Not verified | No | No | Controlled topology matrix required |
| Sleep/wake, Wi-Fi change, abrupt helper death | No end-to-end integration | Not verified | No | No | Must verify route ownership and cleanup after crashes |
| Installation, elevation refusal, repair, upgrade, uninstall | Preview package only | ARM app inspected | Windows install/launch/close/uninstall pass | CI record inspected | Elevation, repair, upgrade and helper paths remain missing; hosted runner cannot prove ordinary-user consent |
| Route/DNS/firewall/internet preservation | Preview has no networking mutations | Mac userspace lab creates no routes | Windows route/DNS unchanged; Linux namespace cleanup passes | Records inspected | Firewall/internet and installed helper acceptance still required |
| Startup, idle memory/CPU, throughput, loss and latency benchmarks | No | Not measured | No | No | Lab wall-clock duration is not a benchmark |

## Hosted verification records

- [Native build/test run 34031905272](https://github.com/BenjaminD2023/QuickLAN/actions/runs/34031905272): all four jobs passed. App build commit `f7dc384d04288ddfc62250d0680f2220c3231cea`; [native-ci.json](evidence/native-ci.json) and [Windows smoke](evidence/windows-installer-smoke.json).
- [Real TUN feasibility run 34032344518](https://github.com/BenjaminD2023/QuickLAN/actions/runs/34032344518): both TCP and UDP underlay jobs passed. Two isolated Linux network namespaces, real TUN addresses, bidirectional TCP/UDP echo, abrupt stop/restart, management denial and owned cleanup. [TCP report](evidence/tun-feasibility-tcp.json), [UDP report](evidence/tun-feasibility-udp.json). Reproduce through the manual `feasibility.yml` workflow on a disposable Linux runner; it requires root to create isolated namespaces and has no public routes.
- These results add OS virtual-IP feasibility, native builds and Windows preview acceptance. They do not complete the missing helper or prove desktop cross-device interoperability, NAT traversal, relay policy or ordinary-user elevation.

## Executable lab and platform procedure

Run the README checks in a disposable checkout. Core lab fixtures contain only ephemeral peer metadata; credentials and raw configs stay in private temporary directories and are removed. For current host-application tests, install a complete native 0.2.0 package; do not elevate the stock RPC endpoint.

On each macOS test device, capture before/after state to private files with `netstat -rn`, `scutil --dns`, `ifconfig`, and `route -n get default`. On Windows use PowerShell `Get-NetRoute`, `Get-DnsClientServerAddress`, `Get-NetAdapter`, and `Get-NetFirewallRule`. Record only sanitized diffs. Verify ordinary HTTPS still works. Treat all unrelated changes as failures, including after termination/uninstall.

Connect the same saved network in QuickLAN and read each actual assigned overlay address. On each machine start the Python service probe in `scripts/service-probe.py` bound to its real assigned overlay address. Run its client mode from the other machine with both TCP and UDP, then reverse roles. A successful localhost lab is not a substitute. Do not automatically open firewall ports; explicitly permit only the chosen test port and overlay scope on consenting test machines.

The probe itself passed a localhost TCP/UDP echo check using its explicit `--loopback-lab` flag (`evidence/service-probe.json`). That verifies the diagnostic utility only. Example real-overlay commands using the native helper: `python3 scripts/service-probe.py server --address ACTUAL_OVERLAY_IP --seconds 60` on the host, and `python3 scripts/service-probe.py client --address ACTUAL_OVERLAY_IP` on the other peer. Replace the address with the real assigned address; the script does not create one.

Repeat over separate physical networks with ordinary NAT, blocked UDP and reachable IPv6. For each topology exercise wrong secrets, separate groups, coordinated subnet collision, forced relay, bootstrap disappearance, migration, sleep/wake and interface switch. For direct-only, capture at both endpoints and the controlled relay, assert no application payload traverses any intermediate peer during establishment, failure and reconnect. Keep allowed discovery control traffic distinct.

Refuse privilege, remove/corrupt the helper, alter the pinned core, crash each process, restart repeatedly, and attempt a second connection concurrently. Verify truthful states, no duplicate engine, no stale routes and safe teardown. Install/repair/upgrade/uninstall on all three native architectures before beta. Publish only sanitized test reports and measured benchmark conditions.

## Final release record

The 0.2.0 native preview binaries were built from commit
b80cfd5a9260e2ca71d87616115ef43873cd874c. Subsequent documentation and explicit
Wintun linking-permission additions do not change compiled source. The matching
corresponding-source archive is compiled offline and includes these notices.
Final machine-readable records are `evidence/native-020.json` and the verification
archive attached to the GitHub release.

Windows BUILD metadata reports a dirty checkout. All its pinned input hashes
are reproducible from a clean checkout with Windows CRLF conversion; the dirty
flag is preserved, not rewritten as clean or advertised as signed provenance.
Mac build metadata reports clean checkouts. No reproducible-binary claim is made.
The newly built local Mac app passed signature-integrity checks, but current
native UI inspection was blocked because the UI tool reported the Mac locked.
