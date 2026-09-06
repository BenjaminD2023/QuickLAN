# Covered source availability

The desktop normal dependency graph includes `selectors 0.36.1` under MPL-2.0. Its unmodified crate source is included in `sources/selectors-0.36.1.crate`; the adjacent JSON records the crates.io URL and verified Cargo.lock checksum. The complete MPL-2.0 text is retained in `supplemental/` and the combined dependency notices. Source files retain their original notices. QuickLAN has not modified these files. Extract the gzip tar archive and use Cargo with its manifest to inspect/build the library and fetch its declared dependencies.

EasyTier LGPL code is not included in the desktop executable or bundled as an engine. Its locally downloaded source and binaries are for explicit development tests only. If a future package includes a core executable, ship its exact corresponding source, lockfiles, patch set, build recipes and required license notices alongside it before public distribution.

`docs/evidence/license-inventory.json` covers a broader all-target inventory than the three launch targets. Four missing texts remain for nonlaunch target packages; the recorded normal dependency graphs for the three initial targets have no missing license-text entries. This inventory is not a substitute for checking platform resources, build provenance and legal redistribution obligations.
