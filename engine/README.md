# QuickLAN networking engine

This is a separate GPL-3.0 executable using the pinned EasyTier 2.6.4 library with the retained policy patch. The desktop/application crates remain Apache-2.0. Upstream library licensing is preserved; this does not relicense EasyTier or Wintun.

The engine never starts EasyTier's TCP management server. A desktop-owned Unix socket or Windows named pipe checks kernel-reported process IDs in both directions. Only a narrow, bounded start/status/stop protocol is accepted. Network credentials travel in that channel, not arguments or persistent config files. The UI stays unprivileged; macOS authorization and Windows UAC elevate only this process for a connection. There is no permanently installed daemon or invisible service.

The engine checks OS routing conflicts before creating an interface. Session-specific adapter names avoid reusing another VPN's adapter. Loss of control IPC, failed heartbeats and shutdown lead to teardown and process exit. Windows uses the original signed Wintun 0.14.1 DLL with a pinned digest, loaded by its absolute path. QuickLAN removes upstream's automatic firewall and obsolete-registry changes.

Source preparation: `python3 scripts/prepare-engine.py`. Build with a protobuf compiler available and `cargo build --manifest-path engine/Cargo.toml --release --locked`. Then `python3 scripts/stage-engine.py` prepares the desktop payload and rejects lab-enabled executables. The desktop build embeds the engine's SHA256. The build's source archive, patch and lockfile must accompany distributions, with all required dependency source and license material.

`--features lab` enables an explicit framed stdio test transport. Production staging rejects this build. Native CI runs the real IPC and virtual-interface acceptance program at `crates/quicklan-runtime/examples/engine_smoke.rs`. This implementation is undergoing verification; source compilation alone does not establish safe installed-user operation or cross-device interoperability.
