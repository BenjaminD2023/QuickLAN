# QuickLAN implementation checklist

Status: engineering preview, not a public beta. No publication, service installation, system VPN or public infrastructure deployment has occurred.

- [x] Inspect and pin EasyTier 2.6.4 source, license, capabilities and three desktop artifact digests.
- [x] Run real isolated TCP/UDP core integration with negative peers and restart; explicitly distinguish userspace traffic from host virtual-IP evidence.
- [x] Implement validated invitations, native secret storage, lifecycle model and exact-version peer parser.
- [x] Implement real desktop saved-network, preview/import, invitation copy, local management, diagnostics, preferences and help flows.
- [x] Reject unavailable direct-only and system connection requests; expose release gaps clearly.
- [x] Verify unit/property tests, UI automation, frontend build/lint, native Keychain round trip and desktop Clippy.
- [x] Prepare native CI, dependency scans, SBOM/notices, engineering packaging and release documentation.
- [ ] Implement authenticated native helper and protected core management. Typed request validation alone does not complete this.
- [ ] Remediate upstream dependency/security findings and enforce no-public, direct-only, forwarding and hostile-route policies in the data plane.
- [ ] Implement live route inspection, owned-resource journaling, crash recovery, real service reachability UI and complete installed-user lifecycle.
- [ ] Verify host virtual-IP traffic, NAT/relay topology, helper consent/refusal, cleanup and installers on all three target architectures.
- [ ] Resolve missing dependency notices, complete corresponding-source redistribution, obtain signing credentials and validate signed public artifacts.

See TEST_MATRIX.md for separate implementation, local, CI and manual status. Native artifact details are recorded in evidence/native-desktop.json. No release gate is passed solely by compilation.
