# QuickLAN engineering handoff

Repository: `/Users/benjamin/QuickLAN`. All application branding uses **QuickLAN**. This is a new local Git repository on `main`, with focused commits. No remote has been added, nothing has been published, and no service, driver, public node or paid infrastructure has been installed or deployed.

## Implemented and exercised

The desktop is Tauri 2 with a React/TypeScript interface and Rust application/domain boundary. Original wrapper code is Apache-2.0. Rust owns network validation, lifecycle state, OS credential access and explicit clipboard commands; the frontend cannot invoke arbitrary processes or load raw configurations. The EasyTier adapter is a separate-process design pinned to **2.6.4, commit 8428a89d2dabc94c97d370ec607c6ca142473626**, under its LGPL-3.0 license.

Working desktop features include saved networks, random IDs/secrets, invitation parsing and preview consent, copying bearer invitations, local rename/forget, replacement-network credentials, configurable saved endpoints/policy, nickname/preferences, light/dark appearance, English and partial Simplified Chinese, help, sanitized diagnostics and explicit quit. One active network is the intended v1 limit. No account, analytics, automatic update checks or background connection is enabled.

The packaged app was exercised through real native IPC on macOS 26.4.1 ARM64: create, invitation copy, helper-unavailable failure, diagnostic preview/copy, quit/relaunch persistence, rename, local forget and dark/system preferences. Temporary test networks were removed. The app has no fake peers or simulated latency.

The developer core lab used four real EasyTier processes, OS-confined to localhost, and passed bidirectional TCP/UDP payloads over both TCP and UDP underlays, correct-secret membership, wrong-secret exclusion, separate-network isolation and restart. This is userspace virtual-address forwarding, **not OS virtual-IP/TUN or cross-device connectivity**.

Validation: 23 Rust tests plus the separately executed native Keychain test, five frontend tests, three explicitly labeled Playwright workflow tests, frontend lint/type/build, workspace/desktop formatting and Clippy, native optimized app build, artifact checksums and Gitleaks source/history scans. npm and QuickLAN-domain Rust scans reported no vulnerabilities; desktop maintenance warnings and core findings remain documented. See [TEST_MATRIX.md](TEST_MATRIX.md), [native record](evidence/native-desktop.json), [security review](SECURITY_REVIEW.md) and [visual QA](DESIGN_QA.md).

## Actual local artifacts

- App: `/Users/benjamin/QuickLAN/src-tauri/target/release/bundle/macos/QuickLAN.app`
- ZIP: `/Users/benjamin/QuickLAN/artifacts/QuickLAN-0.1.0-macos-arm64-engineering.zip`
- Checksums: `/Users/benjamin/QuickLAN/artifacts/SHA256SUMS`
- Unsigned build metadata: `/Users/benjamin/QuickLAN/artifacts/BUILD.json`

The ARM64 executable has only its linker's ad-hoc signature: no Developer ID, sealed bundle resources or notarization. These are local engineering artifacts. No Windows installer or Intel Mac application was built locally. Native CI configurations exist for all three targets but have not run on hosted runners. The package includes notices and the covered MPL source archive; it does not bundle or elevate a stock EasyTier executable.

## Exact development commands

From `/Users/benjamin/QuickLAN`, with the README's native prerequisites:

```sh
npm ci
npm run desktop:dev
```

Checks and the verified macOS app build:

```sh
npm run check
cargo fmt --all --check
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --all-features --all-targets --locked -- -D warnings
cargo clippy --manifest-path src-tauri/Cargo.toml --locked -- -D warnings
npx playwright install chromium --only-shell
npm run test:e2e
npm run desktop:build
```

Core lab on the verified Apple Silicon host:

```sh
python3 scripts/fetch-core.py --target macos-aarch64
python3 scripts/core-integration.py
python3 scripts/core-integration.py --transport udp
```

## Unfinished work and external requirements

**This app cannot yet connect two ordinary users by system virtual IP and is not ready for public beta.** Connect deliberately fails. Authenticated macOS/Windows helpers, protected core management, live OS route checks, crash reconciliation and installed-user lifecycle are not implemented. Typed request validation is only the beginning of a helper boundary.

Stock EasyTier has mutable IP-whitelisted management TCP, a separate implicit TCP STUN list, and unverified strict direct-only and hostile-route behavior. Its pinned source lock produced 23 vulnerability entries; their exact binary reachability is unverified. Core remediation and policy tests come before privileged integration. No-public and direct-only cannot be claimed from a UI option.

Required evidence includes real multi-device virtual-IP traffic, controlled NAT/relay and outage/migration cases, firewall refusal, VPN overlap, MTU/IPv6, sleep/Wi-Fi change, install/elevation refusal/repair/upgrade/uninstall, and measured resource/traffic benchmarks. Only this ARM64 Mac was available; no elevated test access, Windows or Intel Mac environment, second device or controlled remote topology was supplied. Executable verification procedures are in the test matrix.

Before distribution, finish license review and core corresponding-source packaging. Four missing license texts remain in nonlaunch-target inventory; the recorded normal graphs for the three intended targets have no missing text entries. Configure private security reporting, branch protection, reviewed signing workflows and actual Apple/Windows credentials. These credentials and potential distribution costs are separate from server costs. No unlimited public relay capacity is assumed. [RELEASING.md](RELEASING.md) contains conditional signing and exact publication commands; neither signing nor publication occurred.
