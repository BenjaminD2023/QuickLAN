# Native networking implementation

QuickLAN's `feature/native-networking` branch adds a separate `quicklan-engine`
process using the pinned EasyTier 2.6.4 library with the reviewed patch in
`upstream/quicklan.patch`. This document describes the implementation under test;
it does not change the older `v0.1.0-engineering.1` draft's disabled networking.

## Privilege and lifecycle

The desktop stays unprivileged. Each connection requests OS elevation, creates
one virtual adapter, and supervises one engine. Credentials travel through a
framed local IPC channel, never process arguments or configuration files.
The desktop owns a private Unix socket / restricted Windows named pipe; both
ends check the peer process ID through kernel APIs. No core TCP management
server is started. Losing the controller or its heartbeat tears down the engine.
Disconnect waits for engine acknowledgement and actual process termination;
a failed process that has not exited prevents another engine starting.

On macOS, the elevation script copies the engine into an exclusively created,
root-owned directory and checks the embedded SHA256 before execution. A root
supervisor removes that directory when the engine exits. A hard kill of both
processes can leave an inert binary directory, containing no network secrets.
On Windows, executable, driver and ancestor-directory handles deny replacement
while the elevated process runs. Reparse points are rejected. The Wintun DLL's
exact official digest is checked before use. This is not publisher signing or
notarization; interactive OS approval remains necessary.

## Network restrictions

The patch keeps address allocation inside the invitation's private IPv4 /24.
Packets outside that scope, IPv6 and broadcasts are rejected before TUN delivery
and outbound encryption. No exit node, subnet proxy, userspace IP proxy, DNS
server, UPnP mapping or automatic broad firewall exception starts. Windows does
not switch the network's firewall profile. Existing routes are read before
adapter creation and overlapping networks are rejected, including split-default
VPN routes. Live state checks that the owned adapter still has its assigned IP.

Manual mode has no implicit public node, STUN service, external-IP lookup or DNS
fallback. Explicit IP endpoints are required. Named endpoints in assisted mode
use the OS resolver only; its configured DNS servers remain the user's choice.
Assisted mode can use an explicitly selected compatible shared node; public
services are not bundled or assumed available. Secure Mode must be enabled on
that shared node. Names/operator labels do not authenticate an operator's key.
Direct-only enforcement is implemented at the final packet-send decision and
inbound relay boundaries; its UI remains disabled until controlled migration
acceptance passes. It must never silently fall back to relayed application data.

The overlay is for direct-IP applications, not Ethernet bridging or automatic
LAN-game discovery. OS firewalls remain authoritative. The peer details screen
can make one bounded TCP connection to a current peer's explicitly selected
port; it neither sends payloads nor infers UDP-game compatibility.

## Build and evidence

Install Rust 1.96 and a protobuf compiler, then run:

```
python scripts/prepare-engine.py
cargo build --manifest-path engine/Cargo.toml --release --locked
python scripts/stage-engine.py
npm ci
npm run tauri -- build
```

The staging script refuses lab builds and checks Windows import dependencies.
`engine/examples/relay_lab.rs` and `--stdio-lab` are only test transports and are
never packaged. `scripts/engine-integration.py` uses isolated Linux kernel stacks
with no public/default routes. Native CI separately verifies production IPC,
actual adapters and cleanup on Windows, both Macs and Linux. These tests do not
establish physical Windows-to-Mac interoperability or arbitrary internet NAT
traversal. The user has no second device available for those tests yet.
