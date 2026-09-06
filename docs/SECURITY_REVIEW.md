# Dependency and boundary review

## Current 0.2.0 engine

The selected patched engine lock has zero known vulnerability findings and five
unmaintained-package warnings in `evidence/engine-rust-audit.json`: derivative,
encoding, paste, proc-macro-error and rustls-pemfile. This scan covers the actual
selected source graph; the historical stock-lock findings below are not the new
engine result. Windows capture/thunk and implicit external DNS dependencies have
been removed from this graph. The stock core is never elevated. Native
kernel-authenticated IPC and executable substitution defenses are implemented;
see NATIVE_ENGINE.md and THREAT_MODEL.md. Neither zero findings nor passing tests
constitute an independent security audit.

Desktop and engine notice inventories now report no missing license-text entries.
Exact source headers and supplemental provenance notes are retained for crates
that omit license files; this is not an independent legal opinion. Corresponding
source is exported with the release.

## Historical 0.1.0 / stock-core baseline


This is an engineering review, not an independent security audit. Machine-readable evidence lives in `docs/evidence/`. Scans ran on 2026-09-06 using cargo-audit 0.22.2 / RustSec and npm audit. Lockfile coverage includes target-conditional packages that may not be linked on this Mac.

- npm: zero reported vulnerabilities.
- QuickLAN Rust domain lock: zero vulnerability findings and zero warnings after replacing `rand 0.8.5` with patched `0.8.6`. The previous RUSTSEC-2026-0097 logger/thread-RNG unsoundness was addressed despite QuickLAN's use of OsRng.
- Tauri desktop lock: zero vulnerability-class findings; 16 unmaintained-package warnings and one unsoundness warning remain. The normal dependency graphs for all three initial targets contain no GTK/glib/atk packages, verified with locked `cargo tree --target ... --edges normal` (see `evidence/target-dependency-review.json`). Those Linux entries are outside the initial target graphs. Unmaintained `unic-*` and `proc-macro-error` require upstream maintenance tracking. `RUSTSEC-2024-0429` concerns `glib::VariantStrIter`. These warnings are retained, not suppressed.
- EasyTier 2.6.4 source lock: **23 vulnerability entries**, plus 25 unmaintained, 11 unsoundness and two yanked warnings. A package can have multiple advisories. The exact upstream binary build feature graph and reachability have not been audited; these results are not a claim that every issue is reachable in every downloaded binary. They are an additional release gate. The stock binaries are not bundled or elevated by QuickLAN.

The source scan includes advisories for bytes, crossbeam, h2, hickory-proto, idna, maxminddb, openssl, quick-xml, quinn-proto, rkyv, rsa, rustls-webpki and tracing-subscriber. See `evidence/upstream-rust-audit.json` for IDs, versions, affected functions and remediation ranges. Remediation must preserve reproducible source/build provenance and rerun network compatibility tests; a silently replaced core is unacceptable.

Checksum verification establishes agreement with recorded release assets. It does not attest to publisher identity, supply-chain integrity or source/binary equivalence. A future released core needs trustworthy build provenance, policy fixes and license-complete corresponding source.

The desktop has no shell/filesystem/network plugin grants, remote pages, arbitrary process launch API, continuous clipboard reads or browser secret persistence. Rust command errors expose fixed codes. Stock core logs are discarded in the lab. Diagnostics are a small field allowlist with a bounded in-memory event ring. The privileged helper is absent, rather than an unauthenticated service being accepted as secure.

The secret scanner is Gitleaks 8.30.1 downloaded from its official release and checked against the recorded SHA-256 digest (`evidence/secret-scanner-artifact.json`). Scan only reviewed project source, with redaction enabled; inspect the final scan record separately. Neither a clean secret scan nor the invitation property tests prove absence of all leaks or parser vulnerabilities.
