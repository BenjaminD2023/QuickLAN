#!/usr/bin/env python3
"""Rebuild the generated EasyTier source tree from a verified archive and patch.

Run before Cargo, never while a build is using .cache/quicklan-easytier.
This removes only that generated, ignored source tree; it is not an editable fork.
"""
import hashlib
import json
import pathlib
import posixpath
import shutil
import subprocess
import tarfile
import tempfile
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[1]
spec = json.loads((ROOT / 'upstream/easytier.lock.json').read_text())['source']
cache = ROOT / '.cache'
cache.mkdir(exist_ok=True)
if cache.is_symlink():
    raise SystemExit('Refusing a symlink cache')
archive = cache / 'easytier-v2.6.4.tar.gz'
if not archive.exists():
    with urllib.request.urlopen(spec['url'], timeout=300) as response:
        payload = response.read(100_000_001)
    if len(payload) > 100_000_000 or hashlib.sha256(payload).hexdigest() != spec['sha256']:
        raise SystemExit('Source download digest mismatch')
    with archive.open('xb') as output:
        output.write(payload)
if archive.is_symlink() or hashlib.sha256(archive.read_bytes()).hexdigest() != spec['sha256']:
    raise SystemExit('Source archive digest mismatch')
with tempfile.TemporaryDirectory(prefix='quicklan-source-', dir=cache) as tmp:
    with tarfile.open(archive) as source:
        for member in source.getmembers():
            path = pathlib.PurePosixPath(member.name)
            if path.is_absolute() or '..' in path.parts or path.parts[0] != 'EasyTier-2.6.4':
                raise SystemExit('Unsafe archive member')
            if member.issym() or member.islnk():
                target = posixpath.normpath(posixpath.join(posixpath.dirname(member.name), member.linkname)) if member.issym() else member.linkname
                if not target.startswith('EasyTier-2.6.4/') or member.linkname.startswith('/'):
                    raise SystemExit('Unsafe archive link')
            elif not member.isfile() and not member.isdir():
                raise SystemExit('Unsupported archive member')
        # Materialize the five internal source links so Windows builders do not
        # need Developer Mode / symlink privileges merely to read licenses/assets.
        source.extractall(tmp, members=[m for m in source.getmembers() if not m.issym() and not m.islnk()])
        for member in source.getmembers():
            if not member.issym() and not member.islnk():
                continue
            target = posixpath.normpath(posixpath.join(posixpath.dirname(member.name), member.linkname)) if member.issym() else member.linkname
            src, dst = pathlib.Path(tmp) / target, pathlib.Path(tmp) / member.name
            if src.is_dir():
                shutil.copytree(src, dst)
            else:
                shutil.copyfile(src, dst)
    generated = pathlib.Path(tmp) / 'EasyTier-2.6.4'
    # Isolate git apply from the enclosing QuickLAN worktree. Without this,
    # Git can silently skip paths because the source lives in a subdirectory.
    subprocess.run(['git', 'init', '-q'], cwd=generated, check=True)
    subprocess.run(['git', 'config', 'core.autocrlf', 'false'], cwd=generated, check=True)
    subprocess.run(['git', 'apply', '--check', str(ROOT / 'upstream/quicklan.patch')], cwd=generated, check=True)
    subprocess.run(['git', 'apply', str(ROOT / 'upstream/quicklan.patch')], cwd=generated, check=True)
    output = cache / 'quicklan-easytier'
    if output.is_symlink():
        raise SystemExit('Refusing a symlink generated tree')
    if output.exists():
        shutil.rmtree(output)
    generated.rename(output)
print('Prepared verified EasyTier 2.6.4 source with the QuickLAN policy patch.')
