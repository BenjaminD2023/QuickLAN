# Native preview releases

Published desktop: [QuickLAN 0.2.1 native networking preview](https://github.com/BenjaminD2023/QuickLAN/releases/tag/v0.2.1-preview.1).
Android and route-coexistence changes ship as 0.2.2 prerelease assets. 0.2.1
desktop checksums remain in [publication evidence](evidence/github-release-021.json).

Source: https://github.com/BenjaminD2023/QuickLAN. Version 0.2.2 packages include
the production networking helper on desktop and an in-process Android engine;
0.1.0 engineering draft binaries remain historical and cannot connect. Use a
prerelease channel until ordinary-user, physical-device and signing acceptance
is complete. Public upload of unsigned previews does not imply a signed public
beta, Play Store listing, or production readiness.

The 0.2.1 release also verifies explicit firewall transitions and restores the
CI machine’s original firewall settings; see [firewall controls](FIREWALL.md).

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

## Android APK

Install JDK 17, Android SDK `platforms;android-35`, `build-tools;35.0.0` and
`ndk;27.2.12479018`; set `JAVA_HOME` and `ANDROID_HOME`. Install Rust targets
`aarch64-linux-android` and `x86_64-linux-android`, Node/npm and protobuf.

```sh
rustup target add aarch64-linux-android x86_64-linux-android
npm ci
npm run android:build
adb install -r artifacts/android/QuickLAN-0.2.2-android-debug.apk
```

The script builds the actual JNI engine, stages the shared frontend and notices,
and invokes the checked-in Gradle wrapper. Both 64-bit ABIs ship by default;
`npm run android:build -- --abi arm64-v8a` builds only ARM64. Android 8/API 26 is
the declared minimum, not a minimum-device verification claim. A current Android
System WebView with secure web-message support is required.

`npm run android:release` creates an **unsigned** release APK. Debug APKs use the
local Android debug key; they are for sideload testing, not Play Store submission.
No Android release key or store listing is configured. Keep release keys outside
the repository and use Android `apksigner` only in a reviewed signing environment.
Verify the final signed bytes with `apksigner verify --verbose` and
`zipalign -c -P 16 4 <apk>`, then record SHA-256. Preserve one signing identity for
updates; uninstalling an differently signed copy deletes its private credentials.

`.github/workflows/android.yml` builds and checks APK packaging. Local emulator
runtime evidence is recorded separately in TEST_MATRIX; a workflow file is not
proof that hosted CI or physical-device acceptance ran. Include corresponding
source, Android notices/SBOM, APK build metadata and test limitations before any
public Android release. Existing published desktop assets do not include Android.

## Corresponding source

From a reviewed clean commit, with the pinned source already prepared:

```
cargo vendor --manifest-path engine/Cargo.toml --sync android/native/Cargo.toml --locked .cache/engine-vendor > .cache/engine-vendor-config.toml
python3 scripts/package-source.py --output artifacts/QuickLAN-0.2.2-corresponding-source.tar.gz --check-build
```

The script exports tracked source at HEAD, the exact patched EasyTier tree,
vendor source and an offline engine build configuration. It checks locked offline
resolution for desktop and Android Rust engines, and optionally compiles the
desktop export. No compiled unused upstream Packet/WinDivert drivers are included.
Toolchain/protobuf are prerequisites. Gradle/SDK/npm artifacts are not vendored.
The source archive and original notices must accompany desktop engine and Android
APK releases; the combined Android application is GPL-3.0-only.

## Publish the reviewed preview

Download artifacts only from the passing reviewed commit. Keep:

- `QuickLAN_0.2.1_aarch64.dmg`, `QuickLAN_0.2.1_x64.dmg` and `QuickLAN_0.2.1_x64-setup.exe`.
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
