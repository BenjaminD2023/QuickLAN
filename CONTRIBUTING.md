# Contributing

Read the README, architecture, threat model and test matrix before changing network behavior. Keep changes focused and distinguish tested behavior from intended behavior. Use the pinned toolchain and lockfiles; run the relevant README checks.

Do not replace a missing backend capability with simulated success. Test adapters belong only in explicit test builds. Network changes require controlled, consented peers and private test infrastructure; do not load-test public nodes. Preserve the absence of default routes, DNS changes, subnet routing and implicit relaying.

Changes to invitations, storage, helpers, route validation or diagnostics must include adverse-case evidence. Never commit invitations, credentials, private keys, full upstream logs or unredacted captures. Dependency changes require updated audits, SBOM, licenses and exact artifact pins. Do not suppress an advisory without a documented analysis.

Run UI tests after functional UI changes and inspect the native application after IPC changes. Record platform and topology limitations in TEST_MATRIX.md. Windows CI compilation is not proof of interactive UAC or installation behavior.

Public repository hosting and maintainer contact are not configured yet. Once published, submit a focused pull request with the included template. Original contributions are under Apache-2.0; retain third-party licenses and provenance. Do not copy upstream code into the wrapper without handling its applicable license.
