#!/usr/bin/env python3
"""Test the actual firewall runtime on disposable CI; restore original settings even on failure."""
import json
import os
import pathlib
import subprocess
import sys

if os.environ.get('CI') != 'true' or sys.platform not in ('darwin', 'win32'):
    raise SystemExit('Only run on disposable Windows/macOS CI')


def run(args):
    return subprocess.run(args, check=True, capture_output=True, text=True).stdout.strip()


def ps(script):
    return run(['powershell.exe', '-NoLogo', '-NoProfile', '-NonInteractive', '-Command',
                "$ErrorActionPreference='Stop'; " + script])


def mac(*args):
    return run(['sudo', '-n', '/usr/libexec/ApplicationFirewall/socketfilterfw', *args])


if sys.platform == 'win32':
    original = json.loads(ps('Get-NetFirewallProfile -PolicyStore PersistentStore | Select-Object Name,Enabled | ConvertTo-Json -Compress'))
else:
    original = {'global': mac('--getglobalstate'), 'blockall': mac('--getblockall')}
result = None
try:
    binary = 'target/debug/examples/firewall_smoke' + ('.exe' if sys.platform == 'win32' else '')
    # macOS CI elevation uses the same AppleScript path as the desktop.
    command = ['sudo', '--preserve-env=CI', binary] if sys.platform == 'darwin' else [binary]
    result = json.loads(run(command))
finally:
    if sys.platform == 'win32':
        for p in original:
            assert p['Name'] in ('Domain', 'Private', 'Public')
            value = {0:'False', 1:'True', 2:'NotConfigured'}[p['Enabled']]
            ps(f"Set-NetFirewallProfile -PolicyStore PersistentStore -Profile {p['Name']} -Enabled {value}")
        restored = json.loads(ps('Get-NetFirewallProfile -PolicyStore PersistentStore | Select-Object Name,Enabled | ConvertTo-Json -Compress'))
    else:
        mac('--setglobalstate', 'off' if '(State = 0)' in original['global'] else 'on')
        mac('--setblockall', 'on' if 'enabled' in original['blockall'].lower() else 'off')
        restored = {'global': mac('--getglobalstate'), 'blockall': mac('--getblockall')}
    assert original == restored, 'Original firewall settings were not restored'
if result is None:
    raise SystemExit('Firewall acceptance failed')
result.update({'original_settings_restored':True, 'original_settings':original,
               'platform':sys.platform, 'commit':run(['git','rev-parse','HEAD'])})
output = pathlib.Path('src-tauri/target/release/bundle/firewall-smoke.json')
output.parent.mkdir(parents=True, exist_ok=True)
output.write_text(json.dumps(result,indent=2)+'\n')
print('PASS: real firewall disable/enable, confirmation gate, and original settings restored')
