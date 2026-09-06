# Native preview releases

Published: [QuickLAN 0.2.0 native networking preview](https://github.com/BenjaminD2023/QuickLAN/releases/tag/v0.2.0-preview.1).
All ten published asset sizes and SHA-256 digests match the verified local payloads;
see [publication evidence](evidence/github-release-020.json).

Source: https://github.com/BenjaminD2023/QuickLAN. Version 0.2.0 packages include
the production networking helper; 0.1.0 engineering draft binaries remain
historical and cannot connect. Use a prerelease channel until ordinary-user,
physical-device and signing acceptance is complete. Public upload of unsigned
previews does not imply a signed public beta or production readiness.

## Build and verify

The native workflow builds/tests on Windows x64, Apple Silicon and Intel Mac.
It verifies real adapters, both-end authenticated IPC and cleanup; mounted Mac
DMGs and installed Windows payloads are checked against the exact staged helper.
The Windows smoke also opens/closes the native window and uninstalls. Separate
Linux namespaces exercise real TCP/UDP, forced relay and direct-only migration.
Actions are pinned, tokens read-only, checkout credentials not persisted, with
no signing secrets or pull_request_target. Forks never receive signing material.

```
python3 scripts/prepare-engine.py
cargo build --manifest-path engine/Cargo.toml --release --locked
python3 scripts/stage-engine.py
npm ci
# Native macOS
npm run tauri -- build --bundles app,dmg -- --locked
# Native Windows
npm run tauri -- build --bundles nsis -- --locked
```

Output is under `src-tauri/target/release/bundle/`. Never stage a `lab` build.
Regenerate both metadata inventories and audit all three Cargo lockfiles plus
npm. Preserve the official Wintun and WebView2 resources/notices. The helper's
macOS signature is applied before its digest is embedded in the desktop.

## Corresponding source

From a reviewed clean commit, with the pinned source already prepared:

```
cargo vendor --manifest-path engine/Cargo.toml --locked .cache/engine-vendor > .cache/engine-vendor-config.toml
python3 scripts/package-source.py --output artifacts/QuickLAN-0.2.0-corresponding-source.tar.gz --check-build
```

The script exports tracked source at HEAD, the exact patched EasyTier tree,
vendor source and an offline engine build configuration. It checks locked offline
resolution and optionally compiles the export. No compiled unused upstream
Packet/WinDivert drivers are included. Toolchain/protobuf are prerequisites.
The source archive and original license notices must accompany engine binaries.

## Publish the reviewed preview

Download artifacts only from the passing reviewed commit. Keep:

- `QuickLAN_0.2.0_aarch64.dmg`, `QuickLAN_0.2.0_x64.dmg` and `QuickLAN_0.2.0_x64-setup.exe`.
- Matching corresponding source, notices, SBOMs, verification records and per-target BUILD metadata.
- SHA256SUMS covering the actual release assets.

Verify the DMGs and exact helper digests, ensure native tests passed, scan the
reviewed source with redacted Gitleaks, then upload to a GitHub **prerelease**.
Read release metadata back and compare asset digests. Build manifests are unsigned
metadata, not cryptographic provenance or a reproducible-build claim. Do not copy
an old 0.1.0 preview into the new release. The app performs no automatic update.

No Apple Developer ID, Windows publisher certificate or notarization credentials
are available. Mac signatures are ad-hoc; Windows app/installer unsigned. OS
warnings or installation blocks may occur. Do not disable OS protections globally.
Before a signed beta, verify ordinary-user permission refusal, upgrade/repair,
minimum versions, physical OS pairs and the network topology matrix. No benchmark
or unlimited-scale claim is permitted without measured conditions.

## Conditional signing

Signing is opt-in. The default build receives no signing secrets. `scripts/signing-config.py` writes a private, fixed-schema Tauri config only when the relevant environment variables are present; it neither purchases certificates nor signs anything itself.

For macOS, obtain a real Developer ID Application identity and configure Tauri's documented `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` and either App Store Connect API notarization credentials or the supported Apple ID credentials. Do not save secrets in the repository or shell history. Then:

```sh
python3 scripts/signing-config.py --platform macos --output .cache/signing.json
npm run tauri -- build --config .cache/signing.json --bundles app,dmg -- --locked
codesign --verify --deep --strict src-tauri/target/release/bundle/macos/QuickLAN.app
xcrun stapler validate src-tauri/target/release/bundle/macos/QuickLAN.app
spctl --assess --type execute src-tauri/target/release/bundle/macos/QuickLAN.app
```

For Windows, provision a legitimate code-signing certificate/key using its provider's supported protected storage. Set `QUICKLAN_WINDOWS_CERT_THUMBPRINT` and the provider-approved `QUICKLAN_TIMESTAMP_URL`, generate `--platform windows` config, then run `npm run tauri -- build --config .cache/signing.json --bundles nsis -- --locked` on Windows. Verify the executable and installer with `signtool verify /pa /all /v <actual-artifact>`. Do not imply signatures guarantee immediate SmartScreen reputation. No Windows signing command has been exercised here.

Follow the current primary [macOS](https://v2.tauri.app/distribute/sign/macos/) and [Windows](https://v2.tauri.app/distribute/sign/windows/) signing guides. Developer memberships, certificates and key services may have costs separate from server costs. No price or free certificate is promised.

Before introducing a signing workflow, require a protected `release` environment restricted to reviewed main-branch commits and maintainer approval. Enforce branch protection and minimal permissions; never reuse an untrusted PR artifact or run PR code with signing secrets. Protect signing keys and notarization credentials, and retain verified signatures, checksums and provenance for the exact bytes delivered. Environment protections require repository-owner configuration; merely naming an environment is insufficient.


The conditional signing recipe is preparation, not a tested signing pipeline.
Sign a helper with the desired real identity before staging/embedding its hash;
changing helper bytes after desktop compilation breaks verification. A future
release workflow must verify signatures on helper, app and installer separately.
