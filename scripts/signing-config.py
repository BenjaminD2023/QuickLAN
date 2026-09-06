#!/usr/bin/env python3
"""Prepare explicit signing configuration; credentials are never written or printed."""
import argparse
import json
import os
import pathlib
import re
import urllib.parse

p = argparse.ArgumentParser()
p.add_argument('--platform', choices=['macos', 'windows'], required=True)
p.add_argument('--output', type=pathlib.Path, required=True)
a = p.parse_args()
if a.platform == 'macos':
    identity = os.environ.get('APPLE_SIGNING_IDENTITY', '')
    if not identity.startswith('Developer ID Application: ') or any(ord(c) < 32 for c in identity):
        p.error('A real Developer ID Application identity is required')
    config = {'bundle': {'macOS': {'signingIdentity': identity, 'hardenedRuntime': True}}}
else:
    thumbprint = os.environ.get('QUICKLAN_WINDOWS_CERT_THUMBPRINT', '')
    timestamp = os.environ.get('QUICKLAN_TIMESTAMP_URL', '')
    u = urllib.parse.urlsplit(timestamp)
    if not re.fullmatch('[A-Fa-f0-9]{40}', thumbprint) or u.scheme != 'https' or not u.hostname or u.username or u.password or u.fragment:
        p.error('A valid certificate thumbprint and provider-approved HTTPS timestamp URL are required')
    config = {'bundle': {'windows': {'certificateThumbprint': thumbprint, 'digestAlgorithm': 'sha256', 'timestampUrl': timestamp}}}
a.output.parent.mkdir(parents=True, exist_ok=True)
fd = os.open(a.output, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
with os.fdopen(fd, 'w') as f:
    json.dump(config, f, indent=2)
    f.write('\n')
print('Created private signing configuration. No artifact has been signed or verified.')
