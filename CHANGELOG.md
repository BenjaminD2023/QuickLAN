# Changelog

## 0.2.0 — native networking preview

- Bundled a separate elevated networking engine with authenticated local IPC, real adapters, live peer state and supervised cleanup.
- Enforced private-subnet traffic, no implicit public assistance, no unrelated forwarding and direct-only application traffic.
- Passed real bidirectional TCP/UDP, forced relay, wrong credential, restart and direct-link-loss tests in isolated Linux stacks.
- Added explicit TCP application-port checks, stale-peer handling and a local host endpoint picker.
- Removed known vulnerability findings from the selected engine lock; preserved upstream licenses and added exact corresponding-source packaging.
- Native DMG/EXE package verification runs the embedded/installed helper. Physical device pairs, interactive permission acceptance and signed distribution remain unverified.


## 0.1.0 — draft engineering preview

- Added Rust domain model, versioned invitations, explicit preview consent, OS credential storage, bounded diagnostics, lifecycle and route validation.
- Added Tauri desktop create/import/invite/rename/forget/replacement/settings flows, preferences, help and honest unavailable connection state.
- Pinned EasyTier 2.6.4 and verified Apple Silicon, Intel Mac and Windows x64 artifacts.
- Exercised actual TCP/UDP core payload exchanges in an isolated local userspace lab.
- Added adverse-input tests, UI automation, design evidence, native CI configuration and release documentation.
- Published source and staged native DMG/EXE release assets, build metadata, checksums and license materials.
- Passed Windows credential and installer smoke checks and both native Mac builds.
- Passed Linux real-TUN bidirectional TCP/UDP, abrupt stop/restart and cleanup feasibility tests.
- System virtual networking, protected helper installation, strict connection policies and public beta remain incomplete.
