# Architecture and decisions

```
Unprivileged React/Tauri window
  -> allowlisted, typed Tauri commands
  -> Rust App<OsStore>: validation, invitations, state, diagnostics
     -> Keychain / Windows Credential Manager + atomic private metadata
     -> DesktopRuntime worker: engine digest, OS elevation, lifecycle
        -> kernel-authenticated Unix socket / Windows named pipe
        -> separate elevated quicklan-engine (GPL-3.0-only)
           -> patched EasyTier 2.6.4 library -> utun/TUN/Wintun
```

The desktop and networking engine are separate processes and license boundaries. The stock EasyTier management TCP service is never started. Credentials use bounded framed IPC, not arguments or config files. Both ends authenticate peer process IDs through kernel APIs. The Mac launcher stages a hash-verified root-owned copy; Windows holds replacement-denying handles to the executable, driver and their directory ancestry. See [NATIVE_ENGINE.md](NATIVE_ENGINE.md).

One active network is enforced by the runtime worker and single-instance desktop plugin. Lifecycle generations reject stale state. Disconnect waits for actual engine termination before another start; a stuck old engine keeps the application busy. Heartbeat loss or controller EOF closes the adapter. Mac utun and Linux TUN vanish when their handles close; Windows explicitly deletes its owned adapter. No persistent system service or startup registration is installed. Reinstalling replaces the bundled helper; there is no separate service repair database.

Invitations contain a 128-bit random network identifier and a 256-bit random credential, encoded in a bounded versioned token. Exact field/endpoint/subnet validation precedes preview; a local one-use preview ticket binds acceptance to that content. This does not make the network invitation single-use. Credentials are stored in the OS vault, separately from private atomic metadata. Best-effort rollback handles write failures; a crash between stores can still leave an orphan credential or unavailable saved entry. Failures never silently generate a replacement secret.

The frontend refreshes current native state every three seconds. Engine state comes from Instance APIs and actual interface enumeration. Peer paths use live connections/next hops; unavailable latency is absent, not zero. The TCP port checker validates a current peer and overlay address in Rust, then performs one bounded connection without payloads. Local endpoint selection reads OS interfaces only and is initiated by a user action; it does not send discovery probes.

The patched data plane admits only the agreed private IPv4 /24 and local destination/source direction. Unrelated route advertisements, IPv6 and broadcasts are rejected. No DNS, default route, subnet proxy, exit node or broad firewall/profile modification is enabled. Conflicting existing routes are checked before creating an adapter. A later unrelated VPN route change remains an unverified overlap scenario; users should disconnect before changing VPN configurations.

Manual mode has no implicit public endpoint, STUN, external-IP probe or DNS fallback. Explicit shared nodes are configurable and require consent in assisted/direct-only profiles. Their names are descriptive, not authenticated operator keys. There is no hosted control plane, automatic connection, telemetry, updater or tray mode. Closing the window disconnects.

Raw upstream logs are discarded because upstream can log secrets. Diagnostics are a 100-entry in-memory ring and a fixed field allowlist; secrets, payloads, labels and addresses are excluded. Owned secret buffers are zeroized, but parser, clipboard and OS copies can remain. No memory protection guarantee or independent audit is claimed.
