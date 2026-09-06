#!/usr/bin/env python3
"""Real OS virtual-IP proof inside two isolated Linux network namespaces.

CI lab only, never a production helper. No public/default route exists inside
either namespace. The IPv4 management socket rejects every IPv4 caller through
an IPv6-only whitelist. No credentials, raw core output or packet dumps persist.
"""
import argparse
import hashlib
import json
import os
import pathlib
import secrets
import signal
import subprocess
import tempfile
import time
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--transport', choices=['tcp', 'udp'], default='udp')
args = parser.parse_args()
if os.geteuid() != 0 or os.environ.get('CI') != 'true':
    raise SystemExit('Restricted to an explicitly launched, isolated Linux CI lab.')
if not pathlib.Path('/dev/net/tun').exists():
    raise SystemExit('The runner has no TUN device.')

def command(argv, check=True, timeout=10):
    return subprocess.run(argv, capture_output=True, text=True, timeout=timeout, check=check)

def wait_for(predicate, seconds=30):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        if predicate():
            return
        time.sleep(0.25)
    raise AssertionError('Timed out waiting for a real kernel interface or payload exchange.')

def normalized_routes():
    return json.dumps(json.loads(command(['ip', '-j', 'route']).stdout), sort_keys=True)

lock = json.loads((ROOT/'upstream/easytier.lock.json').read_text())
archive_name = 'easytier-linux-x86_64-v2.6.4.zip'
archive = ROOT/'.cache'/archive_name
assert hashlib.sha256(archive.read_bytes()).hexdigest() == lock['artifacts'][archive_name]['sha256']
before_routes = normalized_routes()
before_dns = pathlib.Path('/etc/resolv.conf').read_bytes()
names = ['qlan-' + secrets.token_hex(5) for _ in range(2)]
created = []
children = []
cores = []
checks = []
env = {'PATH': '/usr/sbin:/usr/bin:/sbin:/bin', 'HOME': '/nonexistent', 'LANG': 'C.UTF-8'}

def ns(index, argv, **kwargs):
    return command(['ip', 'netns', 'exec', names[index]] + argv, **kwargs)

def start(index, argv):
    child = subprocess.Popen(['ip', 'netns', 'exec', names[index]] + argv,
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        env=env, cwd=env['HOME'], start_new_session=True)
    children.append(child)
    return child

def stop(child):
    if child.poll() is None:
        os.killpg(child.pid, signal.SIGTERM)
        try:
            child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGKILL)
            child.wait(timeout=5)

def interface(index):
    target = f'10.73.42.{index+1}'
    for item in json.loads(ns(index, ['ip', '-j', '-4', 'addr']).stdout):
        if any(a.get('local') == target for a in item['addr_info']):
            return item['ifname']
    return None

try:
    with tempfile.TemporaryDirectory(prefix='quicklan-tun-', dir='/tmp') as temporary:
        directory = pathlib.Path(temporary)
        directory.chmod(0o700)
        env['HOME'] = str(directory)
        with zipfile.ZipFile(archive) as zipped:
            for name in ['easytier-core', 'easytier-cli']:
                data = zipped.read('easytier-linux-x86_64/' + name)
                path = directory/name
                fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o500)
                with os.fdopen(fd, 'wb') as stream:
                    stream.write(data)
        core = str(directory/'easytier-core')
        cli = str(directory/'easytier-cli')
        assert command([core, '--version']).stdout.strip() == lock['version_output']
        for name in names:
            command(['ip', 'netns', 'add', name])
            created.append(name)
        # Both ends move out of the host namespace. There is no host bridge,
        # forwarding rule, NAT, public link or default route in either lab stack.
        command(['ip', 'link', 'add', 'underlay-a', 'netns', names[0], 'type', 'veth',
            'peer', 'name', 'underlay-b', 'netns', names[1]])
        group, credential = secrets.token_hex(16), secrets.token_hex(32)
        for i, name in enumerate(names):
            underlay = ['underlay-a', 'underlay-b'][i]
            ns(i, ['ip', 'link', 'set', 'lo', 'up'])
            ns(i, ['ip', 'addr', 'add', f'192.0.2.{i+1}/30', 'dev', underlay])
            ns(i, ['ip', 'link', 'set', underlay, 'up'])
            assert not ns(i, ['ip', 'route', 'show', 'default']).stdout.strip()
            assert ns(i, ['ip', 'route', 'get', '1.1.1.1'], check=False).returncode != 0
            text = f'''instance_name = "quicklan-tun-lab-{i}"
hostname = "lab-{i}"
ipv4 = "10.73.42.{i+1}/24"
dhcp = false
listeners = ["{args.transport}://192.0.2.{i+1}:11010"]
routes = []
stun_servers = []
stun_servers_v6 = []
ipv6_public_addr_auto = false
ipv6_public_addr_provider = false
exit_nodes = []
[network_identity]
network_name = "{group}"
network_secret = "{credential}"
[flags]
no_tun = false
enable_encryption = true
encryption_algorithm = "aes-256-gcm"
enable_ipv6 = false
accept_dns = false
enable_exit_node = false
proxy_forward_by_system = false
disable_upnp = true
bind_device = false
disable_p2p = true
disable_udp_hole_punching = true
disable_tcp_hole_punching = true
private_mode = true
relay_network_whitelist = ""
disable_relay_data = true
disable_kcp_input = true
disable_quic_input = true
enable_udp_broadcast_relay = false
'''
            if i:
                text += f'\n[[peer]]\nuri = "{args.transport}://192.0.2.1:11010"\n'
            path = directory/f'{i}.toml'
            fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
            with os.fdopen(fd, 'w') as stream:
                stream.write(text)
            argv = [core, '--config-file', str(path), '--rpc-portal', '127.0.0.1:15888',
                '--rpc-portal-whitelist', '::1/128', '--secure-mode', 'true',
                '--disable-env-parsing', '--console-log-level', 'off']
            cores.append((start(i, argv), argv))
        for i in range(2):
            wait_for(lambda: interface(i) is not None)
            try:
                denial = ns(i, [cli, '-p', '127.0.0.1:15888', '-o', 'json', 'node'], check=False, timeout=5)
                assert denial.returncode != 0, 'Privileged management unexpectedly accepted a caller.'
            except subprocess.TimeoutExpired:
                pass  # Rejected/retried RPC must never yield a successful response.
            start(i, ['/usr/bin/python3', str(ROOT/'scripts/service-probe.py'), 'server',
                '--address', f'10.73.42.{i+1}', '--seconds', '300'])
        checks += ['Distinct kernel TUN interfaces with real virtual IPv4 addresses',
            'No public/default route in either namespace', 'IPv4 management requests denied in both namespaces']
        def exchange(i):
            result = ns(i, ['/usr/bin/python3', str(ROOT/'scripts/service-probe.py'), 'client',
                '--address', f'10.73.42.{2-i}'], check=False, timeout=12)
            return result.returncode == 0
        for i in range(2):
            wait_for(lambda: exchange(i))
            checks.append(f'OS virtual-IP TCP and UDP exact payload echo: {i} -> {1-i} -> {i}')
        stop(cores[1][0])
        wait_for(lambda: interface(1) is None)
        assert not exchange(0), 'Traffic unexpectedly survived removal of the remote TUN.'
        cores[1] = (start(1, cores[1][1]), cores[1][1])
        wait_for(lambda: interface(1) is not None)
        for i in range(2):
            wait_for(lambda: exchange(i))
        checks.append('Abrupt remote core stop removes TUN; restart restores bidirectional TCP/UDP')
        for child in children:
            stop(child)
finally:
    for child in children:
        stop(child)
    for name in reversed(created):
        remaining = command(['ip', 'netns', 'pids', name]).stdout.strip()
        assert not remaining, 'Owned namespace still has a live process.'
        command(['ip', 'netns', 'delete', name])

assert before_routes == normalized_routes()
assert before_dns == pathlib.Path('/etc/resolv.conf').read_bytes()
checks.append('Owned processes/interfaces/namespaces removed; host routes and DNS unchanged')
report = {'core': lock['version_output'], 'transport': args.transport,
    'topology': 'Two isolated Linux kernel network stacks, veth-only underlay, real TUN devices',
    'checks_passed': checks,
    'limitations': ['Feasibility lab, not QuickLAN desktop/helper integration',
        'Not Windows/macOS cross-device evidence', 'Not NAT traversal, forced relay or strict direct-only policy proof',
        'No production privilege or management-authentication claim']}
output = ROOT/f'docs/evidence/tun-feasibility-{args.transport}.json'
output.write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps(report, indent=2))
