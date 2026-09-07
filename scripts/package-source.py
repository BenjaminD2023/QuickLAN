#!/usr/bin/env python3
"""Export a clean release commit plus buildable desktop/Android engine source."""
import argparse
import gzip
import hashlib
import json
import pathlib
import shutil
import subprocess
import tarfile
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser()
p.add_argument('--output', type=pathlib.Path, required=True)
p.add_argument('--check-build', action='store_true', help='Also compile the exported engine offline')
a = p.parse_args()


def run(args, cwd=ROOT, **kwargs):
    return subprocess.run(args, cwd=cwd, check=True, **kwargs)


if run(['git', 'status', '--porcelain'], capture_output=True).stdout.strip():
    raise SystemExit('Commit reviewed changes before exporting corresponding source')
commit = run(['git', 'rev-parse', 'HEAD'], capture_output=True, text=True).stdout.strip()
epoch = int(run(['git', 'show', '-s', '--format=%ct', 'HEAD'], capture_output=True).stdout)
version = json.loads((ROOT / 'package.json').read_text())['version']
vendor = ROOT / '.cache/engine-vendor'
upstream = ROOT / '.cache/quicklan-easytier'
if not vendor.is_dir() or not upstream.is_dir():
    raise SystemExit('Run prepare-engine.py and cargo vendor first; see RELEASING.md')

# Re-applying the checked-in patch in a fresh preparation makes the source being
# exported unambiguous. This only replaces the script-owned generated cache tree.
run(['python3', 'scripts/prepare-engine.py'])
with tempfile.TemporaryDirectory(prefix='quicklan-source-') as temporary:
    tmp = pathlib.Path(temporary)
    export = tmp / f'QuickLAN-{version}-source'
    export.mkdir()
    archive = tmp / 'tracked.tar'
    run(['git', 'archive', '--format=tar', '-o', str(archive), commit])
    with tarfile.open(archive) as source:
        for member in source.getmembers():
            if pathlib.PurePosixPath(member.name).is_absolute() or '..' in pathlib.PurePosixPath(member.name).parts:
                raise SystemExit('Unsafe source member')
            if member.issym() or member.islnk():
                raise SystemExit('Tracked source links require explicit review')
        source.extractall(export)
    shutil.copytree(vendor, export / 'vendor', symlinks=False)
    # QuickLAN does not build these upstream executables, kernel-driver copies,
    # optional apps or contributed plugins. Required EasyTier source is retained.
    shutil.copytree(upstream, export / '.cache/quicklan-easytier',
                    ignore=shutil.ignore_patterns('.git', 'target', 'third_party', '*.zip'))
    (export / '.cargo').mkdir(exist_ok=True)
    config = (ROOT / '.cache/engine-vendor-config.toml').read_text()
    if 'directory = ".cache/engine-vendor"' not in config:
        raise SystemExit('Unexpected vendor configuration')
    (export / '.cargo/config.toml').write_text(config.replace('directory = ".cache/engine-vendor"', 'directory = "vendor"'))
    (export / 'SOURCE_BUILD.md').write_text(f'''# QuickLAN {version} corresponding source

Release commit: `{commit}`. Modified EasyTier source is already present under
`.cache/quicklan-easytier`. Changes are identified in `upstream/quicklan.patch`.
All locked desktop and Android engine dependency sources are in `vendor`, with
original notices. Android frontend lockfiles and Gradle build inputs are included.

Install Rust 1.96.0, a C/C++ platform toolchain and a protobuf compiler first.
From this archive's root, build without a network connection:

```
cargo build --manifest-path engine/Cargo.toml --release --locked --offline
```

The engine uses OS TUN/utun on Unix. Windows runtime additionally uses the official
Wintun 0.14.1 DLL pinned in `upstream/wintun.lock.json`; its binary redistribution
license is included. The staging script fetches/verifies that DLL on Windows.
Unused upstream precompiled driver/Packet library copies are omitted.

To use a modified engine, run `python3 scripts/stage-engine.py`, then rebuild the
desktop with its prerequisites and `npm ci` / `npm run tauri -- build -- --locked`.
Desktop dependencies may need internet access; the offline vendor set covers the
network engine. Staging updates the desktop's expected helper digest on rebuild.
No project signing key is required. macOS builds use an ad-hoc integrity signature;
ordinary OS networking elevation still applies. Do not disable OS protections.

Android prerequisites: JDK 17, Android SDK platform/build-tools 35, NDK
27.2.12479018, and Rust targets aarch64-linux-android and x86_64-linux-android.
Set ANDROID_HOME and JAVA_HOME, then run:

```
npm ci
python3 scripts/build-android.py --skip-prepare
```

The script uses the bundled patched source and vendored Rust dependencies.
Gradle, Android SDK and npm dependencies may still require internet access.
Output: artifacts/android/QuickLAN-{version}-android-debug.apk, locally
debug-signed and installable without a project signing key. Release APKs are
unsigned until the distributor signs them. A differently signed APK cannot
update an installed copy; uninstalling that copy deletes its local credentials.
The Android APK combines the engine in-process and is GPL-3.0-only as a whole.

Engine: GPL-3.0-only. Modified EasyTier: LGPL-3.0. Original independent desktop,
domain, IPC and runtime: Apache-2.0. See THIRD_PARTY_NOTICES and licenses/.
''')
    # Resolution must succeed entirely from exported files with the release lock.
    run(['cargo', 'metadata', '--manifest-path', 'engine/Cargo.toml', '--locked', '--offline', '--format-version', '1'],
        cwd=export, stdout=subprocess.DEVNULL)
    run(['cargo', 'metadata', '--manifest-path', 'android/native/Cargo.toml', '--locked', '--offline', '--format-version', '1'],
        cwd=export, stdout=subprocess.DEVNULL)
    if a.check_build:
        run(['cargo', 'build', '--manifest-path', 'engine/Cargo.toml', '--locked', '--offline', '--release',
             '--target-dir', str(ROOT / '.cache/source-build-target')], cwd=export)
    a.output = a.output.resolve()
    a.output.parent.mkdir(parents=True, exist_ok=True)
    def normalize(info):
        info.uid = info.gid = 0
        info.uname = info.gname = ''
        info.mtime = epoch
        return info
    with a.output.open('wb') as raw, gzip.GzipFile(filename='', mode='wb', fileobj=raw, mtime=epoch) as zipped:
        with tarfile.open(fileobj=zipped, mode='w') as tar:
            tar.add(export, arcname=export.name, filter=normalize)
print(json.dumps({'path': str(a.output), 'commit': commit,
                  'sha256': hashlib.sha256(a.output.read_bytes()).hexdigest(),
                  'offline_resolution': True, 'offline_build': a.check_build}, indent=2))
