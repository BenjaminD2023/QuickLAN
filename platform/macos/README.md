# macOS system helper release gate

**Not implemented or installed in this engineering build.** The app must remain unprivileged. No launch daemon, authorization prompt, privileged executable or firewall rule is installed by QuickLAN today. `quicklan_core::helper::status` reports this honestly.

Candidate deployment baseline is macOS 13+ (SMAppService), both aarch64 and x86_64. The local machine is macOS 26.4.1 aarch64. Do not change this file to “supported” based on a successful app build.

Required implementation before connections can be enabled:

1. Fix/replace the stock core's unauthenticated TCP management portal. A signed wrapper around it is insufficient. Keep the patched source and corresponding build recipe; retain LGPL notices.
2. Use a signed SMAppService daemon plus protected Mach/XPC management, authorize each client using its OS audit token and a fixed designated code requirement. A PID/path/UID string supplied by the client is not an identity. Deny same-user foreign apps and other users.
3. Resolve the service/core from root-owned installation locations, reject writable ancestors and symlinks, use no-follow descriptor operations, verify executable signature/digest at the execution boundary and prevent substitution after verification. Never run paths copied from writable app bundles as root.
4. Pass a typed, versioned request and secret over authenticated IPC; the helper repeats invitation/range/endpoint validation. No arbitrary config, command, path or flags. Core launch must clear inherited ET_* variables. Restrict management methods and lifetime; stop when the controlling session disappears unless background behavior was explicitly accepted.
5. Implement owned-resource journaling, cancellation and crash reconciliation with bounded restart/backoff. Confirm route/DNS/firewall invariants and remove only resources the service owns.
6. Implement native install/consent/refusal, status, repair, upgrade and uninstall. Verify startup/shutdown and stale helper detection on both architectures. Signing/notarization credentials are not present in the repository; unsigned development is not a signed distribution.

Use the test procedures in docs/TEST_MATRIX.md. `scripts/core-integration.py` uses the deprecated macOS `sandbox-exec` **only as a local test harness**, never as a production elevation/service design.
