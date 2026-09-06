#!/usr/bin/env python3
"""Hash bundle payloads and record unsigned build metadata; not an attestation."""
import argparse
import hashlib
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--bundle-root', required=True, type=pathlib.Path)
parser.add_argument('--target', required=True)
args = parser.parse_args()
bundle = args.bundle_root.resolve()
if not bundle.is_dir():
    raise SystemExit('Bundle directory does not exist')

def sha(path):
    digest = hashlib.sha256()
    with path.open('rb') as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()

files = sorted(p for p in bundle.rglob('*') if p.is_file() and not p.is_symlink() and p.name not in ('BUILD.json', 'SHA256SUMS'))
if not files:
    raise SystemExit('No artifacts found')
records = [{'path': p.relative_to(bundle).as_posix(), 'sha256': sha(p)} for p in files]
commit = subprocess.run(['git', 'rev-parse', 'HEAD'], cwd=ROOT, capture_output=True, text=True)
status = subprocess.run(['git', 'status', '--porcelain'], cwd=ROOT, capture_output=True, text=True, check=True)
manifest = {
    'product': 'QuickLAN', 'version': json.loads((ROOT/'package.json').read_text())['version'], 'channel': 'native-preview',
    'target': args.target, 'commit': commit.stdout.strip() if commit.returncode == 0 else None,
    'working_tree_dirty': bool(status.stdout.strip()),
    'developer_id_or_authenticode_verified': False,
    'limitation': 'Locally generated unsigned metadata, not cryptographic provenance or a reproducible-build claim.',
    'toolchain': {tool: subprocess.check_output([tool, '--version'], text=True).strip() for tool in ['rustc', 'node']},
    'lockfiles': {p: sha(ROOT/p) for p in ['Cargo.lock', 'engine/Cargo.lock', 'upstream/quicklan.patch', 'src-tauri/Cargo.lock', 'package-lock.json', 'upstream/easytier.lock.json']},
    'files': records,
}
(bundle/'BUILD.json').write_text(json.dumps(manifest, indent=2)+'\n')
(bundle/'SHA256SUMS').write_text(''.join(f"{r['sha256']}  {r['path']}\n" for r in records))
print(f'Recorded {len(records)} payload files for {args.target}')
