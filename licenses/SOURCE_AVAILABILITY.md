# Corresponding source and notices

QuickLAN's desktop/domain/IPC/runtime are Apache-2.0. The separate networking
engine is GPL-3.0-only with the specific Wintun linking permission in
`QuickLAN-Wintun-linking-exception.txt`, and links modified EasyTier 2.6.4 under its original
LGPL-3.0 license. `engine/LICENSE`, `GPL-3.0.txt` and `EasyTier-LGPL-3.0.txt`
retain the complete texts. `upstream/quicklan.patch` identifies modifications.

Each native networking release supplies `QuickLAN-0.2.1-corresponding-source.tar.gz`:
QuickLAN source at the release commit, exact modified EasyTier source, Cargo
lockfiles, vendored engine dependency source, build scripts and license notices.
The archive README describes building the engine offline with Rust 1.96 and a
protobuf compiler. No account or payment is required to obtain the archive.
The desktop's embedded helper hash must be regenerated with `stage-engine.py`
and the desktop rebuilt if the helper is modified. QuickLAN does not require a
publisher secret to run a user's rebuilt engine; platform elevation still applies.

The desktop includes unmodified MPL-2.0 `selectors 0.36.1`; its crate source is
included at `sources/selectors-0.36.1.crate`, with original notices, URL and
Cargo.lock checksum in the adjacent JSON. Extract with tar and build with its
Cargo manifest. The engine's complete vendor tree also retains its dependencies'
original source/license files, including MPL-covered code.

`DEPENDENCY_LICENSES.txt` and `ENGINE_DEPENDENCY_LICENSES.txt` include all-target
inventories, some of which are not linked into the launch platforms. Supplemental
license provenance and digests are recorded in `supplemental/sources.json`.
This is an engineering inventory, not an independent legal review. In particular,
encoding-utils' published crate declares MIT but omits its license text; its exact
source and declared metadata are retained, with a clearly identified canonical
MIT template rather than an invented copyright attribution.

Wintun 0.14.1's official signed x64 DLL is redistributed with software using its
API under `Wintun-Binary-LICENSE.txt`; this is not a relicensing of the driver.
Its official archive and DLL hashes are pinned in `upstream/wintun.lock.json`.
Microsoft's WebView2 SDK loader license and notices are retained separately.
