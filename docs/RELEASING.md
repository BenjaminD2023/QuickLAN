# Engineering builds and public release gates

Nothing has been published. No Git remote, GitHub project, hosting account, Apple Developer identity, Windows signing certificate, hosted runner quota or public node capacity has been configured by this work. The local repository is suitable for review; it is not a public beta.

## Local engineering bundle

Use the README's locked setup/check commands. `npm run desktop:build` produces `src-tauri/target/release/bundle/macos/QuickLAN.app`. Package the actual app without installing a privileged service:

```sh
mkdir -p artifacts
ditto -c -k --sequesterRsrc --keepParent src-tauri/target/release/bundle/macos/QuickLAN.app artifacts/QuickLAN-0.1.0-macos-arm64-engineering.zip
python3 scripts/build-manifest.py --bundle-root artifacts --target aarch64-apple-darwin
```

`BUILD.json` and `SHA256SUMS` describe the bytes and checkout. They are unsigned build metadata, not a signed provenance attestation. A local ad-hoc signature is not Developer ID signing or notarization. The engineering app bundles its runtime UI and notices but no operational networking service/core. It must retain the visible engineering warning.

## Native CI

`.github/workflows/ci.yml` declares Windows x64, Apple Silicon and Intel Mac builds, tests and engineering bundles. It uses immutable action revisions, read-only tokens, no persisted checkout credentials, no signing secrets, no `pull_request_target`, short artifact retention and bounded jobs. It has **not run** on GitHub. Fix native failures before recording a CI pass.

As checked 2026-09-06, GitHub documents `macos-15` as Apple Silicon, `macos-15-intel` as Intel and `windows-2022` as x64. See [runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners). Public repositories can use standard hosted runners without runner-minute charges under GitHub's current offering; concurrency, storage, retention, usage limits and eligibility still apply. This user's plan/quota is unverified. Private jobs and larger runners may cost money. Check [limits](https://docs.github.com/en/actions/reference/limits) before enabling workflows. A runner's administrative privileges or disabled Windows UAC do not prove installed-user consent behavior.

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

## Public beta prerequisites

Resolve all critical gates in TEST_MATRIX.md, including:

1. Remediate pinned upstream dependency advisories, authenticated core management, implicit public STUN, strict direct-only and unauthorized-route enforcement. Preserve a buildable, reviewed upstream patch set and record new digests.
2. Implement authenticated native helpers/installers, executable substitution defenses, live conflict checks, crash recovery and owned-resource cleanup. Verify permission denial and uninstall on all supported targets.
3. Prove real OS virtual-IP TCP and UDP traffic between ordinary users across macOS/macOS, Windows/Windows and mixed pairs on separate physical networks. Test controlled relay/outage/migration/NAT conditions and truthful failures.
4. Complete dependency license review, missing notice texts, platform resources, SBOM and the separate core's corresponding-source bundle. Include exact source, lockfiles, any modifications and build/install instructions. A source URL alone is not the planned LGPL compliance mechanism. Do not relicense upstream.
5. Verify signed/notarized native packages, install/upgrade/uninstall and operational documentation; configure private security reporting and moderation contact. Record benchmark conditions before making performance claims.

Regenerate inventories with `cargo metadata --locked --format-version 1 --manifest-path src-tauri/Cargo.toml > .cache/native-metadata.json`, `python3 scripts/generate-notices.py`, and `npm sbom --omit=dev --sbom-format cyclonedx`. Run current advisory and secret scans; resolve findings, do not silently ignore them. Scan both wrapper locks and the precise core source lock. Scan outputs are not an independent audit.

## Publication after review

Choose the intended owner explicitly and verify the path and reviewed file list. Do not publish caches, test credentials, signing files or unrelated workspace content. This command is documentation only and has not been executed:

```sh
cd /Users/benjamin/QuickLAN
git status --short
git ls-files
gh auth status
gh repo create OWNER/QuickLAN --public --source=. --remote=origin --push
```

Replace `OWNER` with the confirmed account/organization. Enable private vulnerability reporting and required branch/environment protections. Label any initial source publication an engineering preview; do not upload a public beta until gates are met. Repository hosting is for source/downloads and must not become a networking dependency. No hosted nodes or recurring services have been purchased or deployed.
