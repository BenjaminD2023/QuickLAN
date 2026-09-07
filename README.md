# QuickLAN

Private virtual networks with friends. Create a network, share an invitation, and connect to a game or application by virtual IP. QuickLAN uses Tauri, React and Rust with a separately packaged, modified EasyTier 2.6.4 engine. It requires no account, paid API or project-operated server. QuickLAN is independent of EasyTier.

**0.2.1 native preview:** real networking is implemented. Native CI creates and removes actual adapters on Windows x64, Apple Silicon and Intel Mac. Isolated Linux stacks exchange real TCP and UDP through QuickLAN virtual IPs. Physical Windows/Mac device pairs, arbitrary internet NAT traversal, ordinary-user permission dialogs and minimum OS versions remain unverified. This is not a production-ready or signed public beta.

[GitHub releases](https://github.com/BenjaminD2023/QuickLAN/releases) · [Feature status](docs/FEATURES.md) · [Test evidence](docs/TEST_MATRIX.md)

## Connect with friends

1. Install the package for your architecture. The networking engine is included; end users need no Rust, Node or separate core installation.
2. Create a network. For friends on the same router, use **Host on the same Wi-Fi or LAN** and choose your Wi-Fi/Ethernet address. For remote friends, open connection settings and enter an endpoint they can reach, or an explicitly selected compatible shared node with permission from its operator.
3. Connect and approve the operating system's networking permission request. Keep the host connected while friends join.
4. Copy the invitation and share it privately. Friends preview it, save the network, then connect. Copy the host's **virtual** IP into the game's direct-connect screen.
5. Use peer details to check an explicitly selected TCP application port. Preferences → System firewall can explicitly disable or enable the built-in firewall with administrator authorization. This applies to the whole computer and persists after QuickLAN quits; use Enable firewall to turn protection back on. Application-specific rules can also be configured in the OS.

See [firewall controls](docs/FIREWALL.md) for scope, verification and managed-policy behavior.

An invitation alone cannot find arbitrary remote computers behind NAT. No public discovery/relay service is built in. Local interface choices can include VPN/container addresses; choose an address reachable by your friends. Changing routers or local addresses may require editing connection settings and sharing a fresh invitation. [Troubleshooting](docs/TROUBLESHOOTING.md) explains setup and failure states. [Optional node setup](docs/NODE_SETUP.md) covers existing host hardware and operator responsibilities.

## Develop and package

Verified toolchain: Rust 1.96.0, Node 22.22.3, Python 3.9+, protobuf compiler and native [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/). Builds are tested on macOS 15 ARM64/Intel and Windows Server 2022 x64 CI. macOS 13+ and Windows 11 x64 are candidate end-user baselines, not verified minimum-version claims.

```sh
python3 scripts/prepare-engine.py
cargo build --manifest-path engine/Cargo.toml --release --locked
python3 scripts/stage-engine.py
npm ci
npm run desktop:dev
```

To package on a native Mac use `npm run tauri -- build --bundles app,dmg -- --locked`; on Windows use `npm run tauri -- build --bundles nsis -- --locked`. Output: `src-tauri/target/release/bundle/`. macOS helper and app are ad-hoc signed for integrity only; Windows app/installer are unsigned. No Developer ID, Authenticode or notarization credentials are configured.

```sh
npm run check
cargo fmt --all --check
cargo clippy --workspace --all-features --all-targets --locked -- -D warnings
cargo clippy --manifest-path src-tauri/Cargo.toml --locked -- -D warnings
npx playwright install chromium --only-shell
npm run test:e2e
```

`npm run dev` is a browser interface with no native storage/networking. Playwright uses an explicitly labeled test adapter; it does not prove VPN connectivity. Native and isolated networking acceptance procedures are in [TEST_MATRIX.md](docs/TEST_MATRIX.md) and `.github/workflows/networking.yml`.

## Trust and scope

An invitation is a bearer credential. Anyone possessing it may join. Names, peer IDs and addresses are not verified personal identities. Forget is local; replacement credentials create a new network and do not revoke communication on the old one. Trusted peers may reach services allowed by your firewall. This is not an anonymity tool.

One active network per device. No automatic connection, background tray networking, telemetry, automatic updates, DNS/default-route changes, internet exit node, subnet proxy or LAN broadcast bridging. Closing the window or choosing Quit disconnects. Virtual IPs may change after reconnect; applications bound to an old address must be rebound. Relay use requires explicit configuration and consent. No unlimited bandwidth, universal LAN discovery or unmeasured performance claim is made.

[Architecture](docs/ARCHITECTURE.md) · [Native helper](docs/NATIVE_ENGINE.md) · [Threat model](docs/THREAT_MODEL.md) · [Security review](docs/SECURITY_REVIEW.md) · [Release procedure](docs/RELEASING.md)

## License

Original desktop/domain/IPC/runtime code: Apache-2.0. The separate `quicklan-engine` program: GPL-3.0-only with an explicit Wintun linking permission. Modified EasyTier retains LGPL-3.0. Wintun's official binary has its own redistribution license. See [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES), `licenses/`, and the corresponding-source archive supplied with each networking binary release. No independent security audit is claimed.
