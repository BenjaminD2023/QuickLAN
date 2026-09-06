# Windows system service release gate

**Not implemented or tested on Windows in this engineering build.** The NSIS application packaging target is a per-user engineering app, not a working VPN installer. No service or driver is silently installed. Do not run the whole application elevated.

Candidate baseline: Windows 11 x64 and current WebView2. Native runner compilation is not installation/UAC, Wintun or functional compatibility evidence.

The future helper must use Service Control Manager installation through an explicit UAC consent flow, a fixed protected Program Files executable directory and a service identity with only required rights. Named pipes must reject remote clients, apply an explicit DACL, impersonate and inspect the calling token, and verify the expected signed application/client identity. A pipe name or loopback address alone is not authentication. Deny ordinary local processes and other users. Repeat validation on every request.

Stock EasyTier has a mutable, IP-whitelisted TCP RPC portal: **it must not be executed as SYSTEM**. Fix this boundary before wiring any installer to it. Verification must survive executable/config path substitution and symlink/reparse-point attacks. Never let the UI or invitation select privileged file paths.

Track per-session virtual adapter/interface IDs, overlay routes, optional explicitly accepted scoped firewall rules and service resources. Use a Job Object / service lifecycle to stop owned core children on crash. Do not delete shared Wintun installations or another VPN's resources. Implement idempotent disconnect, repair, upgrade and uninstall, and test permission refusal with UAC enabled (GitHub's hosted Windows runner configuration does not establish this).

DPAPI-backed Windows Credential Manager storage is compiled through keyring's `windows-native` feature. It stores only a small per-network secret; metadata is a separate user-owned file. This backend and its default ACLs need verification on a real Windows installation.
