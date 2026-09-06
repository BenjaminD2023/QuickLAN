#!/usr/bin/env python3
"""Stage the built production engine and verified original Wintun DLL for packaging."""
import hashlib
import json
import pathlib
import shutil
import subprocess
import sys
import urllib.request
import zipfile
ROOT = pathlib.Path(__file__).resolve().parents[1]
windows = sys.platform == 'win32'
name = 'quicklan-engine.exe' if windows else 'quicklan-engine'
binary = ROOT / 'engine/target/release' / name
version = subprocess.check_output([str(binary), '--version'], text=True).strip()
if version != 'quicklan-engine-0.2.0-easytier-2.6.4':
    raise SystemExit('Unexpected engine version')
probe = subprocess.run([str(binary), '--stdio-lab'], input=b'', capture_output=True, timeout=5)
if probe.returncode != 2 or probe.stdout:
    raise SystemExit('Refusing to package an engine with the lab-only stdio transport enabled')
output = ROOT / 'src-tauri/resources/engine'
output.mkdir(parents=True, exist_ok=True)
shutil.copyfile(binary, output/name)
(output/name).chmod(0o755)
if windows:
    from pe_imports import imports
    required_dlls = imports(binary.read_bytes())
    if any(name in required_dlls for name in ['packet.dll', 'wpcap.dll', 'windivert.dll']):
        raise SystemExit('Unexpected packet-capture driver dependency')
    print('Verified Windows imports: ' + ', '.join(required_dlls))
    spec = json.loads((ROOT/'upstream/wintun.lock.json').read_text())
    archive = ROOT/'.cache/wintun-0.14.1.zip'
    if not archive.exists():
        with urllib.request.urlopen(spec['url'], timeout=180) as response:
            data = response.read(5_000_001)
        if len(data) > 5_000_000 or hashlib.sha256(data).hexdigest() != spec['sha256']:
            raise SystemExit('Wintun archive digest mismatch')
        archive.write_bytes(data)
    if hashlib.sha256(archive.read_bytes()).hexdigest() != spec['sha256']:
        raise SystemExit('Cached Wintun digest mismatch')
    with zipfile.ZipFile(archive) as z:
        dll = z.read(spec['member'])
        if hashlib.sha256(dll).hexdigest() != spec['dll_sha256']:
            raise SystemExit('Wintun DLL digest mismatch')
        (output/'wintun.dll').write_bytes(dll)
        (output/'Wintun-Binary-LICENSE.txt').write_bytes(z.read(spec['license_member']))
print('Production engine staged. The desktop build embeds its exact SHA256.')
