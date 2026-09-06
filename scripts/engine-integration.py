#!/usr/bin/env python3
"""Exercise QuickLAN IPC, DHCP and real TUN payloads in isolated Linux stacks."""
import argparse
import hashlib
import json
import os
import pathlib
import queue
import secrets
import signal
import struct
import subprocess
import threading
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser()
p.add_argument('--engine', required=True)
p.add_argument('--transport', choices=['tcp', 'udp'], default='udp')
p.add_argument('--relay-lab', help='Explicit test-only relay executable for the isolated three-stack topology')
a = p.parse_args()
if os.geteuid() != 0 or os.environ.get('CI') != 'true':
    raise SystemExit('Disposable Linux CI only')
engine = str(pathlib.Path(a.engine).resolve(strict=True))
names = ['ql-' + secrets.token_hex(5) for _ in range(3 if a.relay_lab else 2)]
created, children, engines, checks = [], [], [], []

def command(argv, check=True, timeout=15):
    return subprocess.run(argv, capture_output=True, text=True, check=check, timeout=timeout)

def ns(i, argv, **kw):
    return command(['ip', 'netns', 'exec', names[i]] + argv, **kw)

def routes():
    return json.dumps(json.loads(command(['ip', '-j', '-4', 'route', 'show', 'table', 'all']).stdout), sort_keys=True)

def wait(predicate, seconds=50):
    until = time.monotonic() + seconds
    while time.monotonic() < until:
        if predicate():
            return
        time.sleep(.25)
    raise AssertionError('Timed out waiting for QuickLAN virtual network convergence')

def stop(child):
    if child.poll() is None:
        os.killpg(child.pid, signal.SIGTERM)
        try:
            child.wait(5)
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGKILL)
            child.wait(5)

class Engine:
    def __init__(self, index, network, credential):
        self.child = subprocess.Popen(['ip', 'netns', 'exec', names[index], engine, '--stdio-lab'],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
            start_new_session=True, bufsize=0)
        children.append(self.child)
        engines.append(self)
        self.replies = queue.Queue(maxsize=4)
        self.state = None
        self.failure = None
        self.closed = threading.Event()
        self.lock = threading.Lock()
        self.session = secrets.token_hex(16)
        self.reader = threading.Thread(target=self.read, daemon=True)
        self.reader.start()
        ready = self.replies.get(timeout=10)
        assert ready['event'] == 'ready', 'Engine handshake missing'
        self.send({'operation': 'start', 'protocol': 1, 'session_id': self.session,
                   'network': network, 'credential': credential, 'nickname': f'CI peer {index}'})
        state = self.replies.get(timeout=60)
        assert state['event'] == 'state' and state['virtual_ip'], f'Engine start failed: {state}'
        self.state = state
        self.heartbeat = threading.Thread(target=self.poll, daemon=True)
        self.heartbeat.start()

    def read(self):
        def exact(n):
            result = b''
            while len(result) < n:
                data = self.child.stdout.read(n-len(result))
                if not data:
                    raise EOFError()
                result += data
            return result
        try:
            while not self.closed.is_set():
                n = struct.unpack('>I', exact(4))[0]
                assert 0 < n <= 1048576
                self.replies.put(json.loads(exact(n)), timeout=5)
        except (EOFError, OSError, queue.Full):
            pass

    def send(self, message):
        data = json.dumps(message).encode()
        with self.lock:
            self.child.stdin.write(struct.pack('>I', len(data)) + data)
            self.child.stdin.flush()

    def poll(self):
        try:
            while not self.closed.wait(.5):
                self.send({'operation': 'status', 'protocol': 1})
                reply = self.replies.get(timeout=10)
                if reply['event'] != 'state':
                    self.failure = (reply['event'], reply.get('code'))
                    return
                self.state = reply
        except (OSError, queue.Empty, ValueError):
            if not self.closed.is_set():
                self.failure = 'IPC heartbeat failed'

    def close_controller(self):
        self.closed.set()
        self.heartbeat.join(timeout=12)
        self.child.stdin.close()
        assert self.child.wait(timeout=12) == 0, 'Controller loss did not stop engine cleanly'
        self.child.stdout.close()
        self.reader.join(timeout=2)

    @property
    def ip(self):
        assert self.failure is None, self.failure
        return self.state['virtual_ip']

before_routes, before_dns = routes(), pathlib.Path('/etc/resolv.conf').read_bytes()
try:
    for name in names:
        command(['ip', 'netns', 'add', name])
        created.append(name)
        # Namespace defaults can inherit forwarding=1 from hosted runners.
        # Change only the freshly created lab namespace, never the host.
        command(['ip', 'netns', 'exec', name, 'sysctl', '-w', 'net.ipv4.ip_forward=0'])
    if a.relay_lab:
        relay_exe = str(pathlib.Path(a.relay_lab).resolve(strict=True))
        for i in range(2):
            command(['ip', 'link', 'add', f'edge{i}', 'netns', names[i], 'type', 'veth',
                     'peer', 'name', f'relay{i}', 'netns', names[2]])
            ns(i, ['ip', 'link', 'set', 'lo', 'up'])
            ns(i, ['ip', 'addr', 'add', f'192.0.2.{1+4*i}/30', 'dev', f'edge{i}'])
            ns(i, ['ip', 'link', 'set', f'edge{i}', 'up'])
            ns(2, ['ip', 'addr', 'add', f'192.0.2.{2+4*i}/30', 'dev', f'relay{i}'])
            ns(2, ['ip', 'link', 'set', f'relay{i}', 'up'])
        ns(2, ['ip', 'link', 'set', 'lo', 'up'])
        assert ns(2, ['sysctl', '-n', 'net.ipv4.ip_forward']).stdout.strip() == '0'
        relay = subprocess.Popen(['ip', 'netns', 'exec', names[2], relay_exe], stdin=subprocess.PIPE,
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
        children.append(relay)
        wait(lambda: ':11010' in ns(2, ['ss', '-lnt']).stdout)
        assert ns(0, ['ip', 'route', 'get', '192.0.2.5'], check=False).returncode != 0
    else:
        command(['ip', 'link', 'add', 'underlay-a', 'netns', names[0], 'type', 'veth',
                 'peer', 'name', 'underlay-b', 'netns', names[1]])
        for i in range(2):
            link = ['underlay-a', 'underlay-b'][i]
            ns(i, ['ip', 'link', 'set', 'lo', 'up'])
            ns(i, ['ip', 'addr', 'add', f'192.0.2.{i+1}/30', 'dev', link])
            ns(i, ['ip', 'link', 'set', link, 'up'])
    for i in range(len(names)):
        assert ns(i, ['ip', 'route', 'get', '1.1.1.1'], check=False).returncode != 0
    network = {'id': secrets.token_hex(16), 'label': 'CI private network', 'subnet': '10.73.42.0/24',
               'policy': 'manual', 'bootstrap': []}
    credential = secrets.token_hex(32)
    if a.relay_lab:
        network.update(policy='assisted', bootstrap=[{'endpoint': f'{a.transport}://192.0.2.2:11010', 'operator': 'CI relay'}])
    host = Engine(0, network, credential)
    remote_network = dict(network, bootstrap=[{'endpoint': f'{a.transport}://192.0.2.1:11010', 'operator': 'CI host'}])
    if a.relay_lab:
        remote_network['bootstrap'] = [{'endpoint': f'{a.transport}://192.0.2.6:11010', 'operator': 'CI relay'}]
    remote = Engine(1, remote_network, credential)
    expected_path = 'relayed' if a.relay_lab else 'direct'
    def converged():
        return host.ip != remote.ip and all(any(peer['virtual_ip'] == other.ip and peer['path'] == expected_path
            for peer in item.state['peers']) for item, other in [(host, remote), (remote, host)])
    wait(converged)
    checks.append(f'Authenticated QuickLAN engines converge to distinct DHCP addresses and report {expected_path} paths')
    addresses = [host.ip, remote.ip]
    probe_servers = []
    for i, address in enumerate(addresses):
        assert any(any(addr.get('local') == address for addr in item['addr_info'])
            for item in json.loads(ns(i, ['ip', '-j', '-4', 'addr']).stdout))
        listeners = ns(i, ['ss', '-lnt']).stdout
        assert ':15888' not in listeners and ':15889' not in listeners, 'Unexpected management listener'
        child = subprocess.Popen(['ip', 'netns', 'exec', names[i], '/usr/bin/python3',
            str(ROOT/'scripts/service-probe.py'), 'server', '--address', address, '--seconds', '300'],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
        children.append(child)
        probe_servers.append(child)
        wait(lambda: child.poll() is not None or f'{address}:39191' in ns(i, ['ss', '-lnt']).stdout, seconds=3)
        assert child.poll() is None, 'Owned probe listener failed to bind'
    def exchange(i):
        assert host.failure is None and remote.failure is None, (host.failure, remote.failure)
        return ns(i, ['/usr/bin/python3', str(ROOT/'scripts/service-probe.py'), 'client', '--address', addresses[1-i]],
                  check=False, timeout=12).returncode == 0
    for i in range(2):
        wait(lambda: exchange(i))
    checks.append('Bidirectional TCP and UDP exact payload echo through OS virtual IPs')
    remote.close_controller()
    wait(lambda: '10.73.42.0/24' not in ns(1, ['ip', '-4', 'route']).stdout)
    assert not exchange(0), 'Payload survived remote adapter removal'
    checks.append('Controller disappearance closes engine and removes its TUN and route')
    # The same bootstrap and network ID with a different credential must never join.
    wrong = Engine(1, remote_network, secrets.token_hex(32))
    time.sleep(5)
    assert not wrong.state['peers'], 'Wrong credential authenticated a peer'
    assert not exchange(0), 'Wrong credential admitted application traffic'
    wrong.close_controller()
    checks.append('Wrong network credential cannot authenticate or exchange application data')
    remote = Engine(1, remote_network, credential)
    wait(converged)
    # DHCP may assign a different host address after a reconnect. Follow the
    # engine's actual state and rebind application listeners, as an app must.
    for child in probe_servers:
        stop(child)
    addresses = [host.ip, remote.ip]
    for i, address in enumerate(addresses):
        child = subprocess.Popen(['ip', 'netns', 'exec', names[i], '/usr/bin/python3',
            str(ROOT/'scripts/service-probe.py'), 'server', '--address', address, '--seconds', '300'],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
        children.append(child)
        wait(lambda: child.poll() is not None or f'{address}:39191' in ns(i, ['ss', '-lnt']).stdout, seconds=3)
        assert child.poll() is None, 'Owned probe listener failed to rebind after reconnect'
    for i in range(2):
        wait(lambda: exchange(i))
    checks.append(f'Restart restores {expected_path} peer state and bidirectional TCP/UDP')
    remote.close_controller()
    host.close_controller()
    if a.relay_lab:
        for child in children:
            if child is not relay:
                stop(child)
        # A separate direct link permits direct-only application traffic first.
        # Removing it leaves the already-proven relay alive for control traffic.
        command(['ip', 'link', 'add', 'direct-a', 'netns', names[0], 'type', 'veth',
                 'peer', 'name', 'direct-b', 'netns', names[1]])
        for i in range(2):
            link = ['direct-a', 'direct-b'][i]
            ns(i, ['ip', 'addr', 'add', f'192.0.3.{i+1}/30', 'dev', link])
            ns(i, ['ip', 'link', 'set', link, 'up'])
        network['policy'] = remote_network['policy'] = 'direct_only'
        network['bootstrap'].append({'endpoint': f'{a.transport}://192.0.3.2:11010', 'operator': 'CI peer'})
        remote_network['bootstrap'].append({'endpoint': f'{a.transport}://192.0.3.1:11010', 'operator': 'CI peer'})
        host, remote = Engine(0, network, credential), Engine(1, remote_network, credential)
        expected_path = 'direct'
        wait(converged)
        addresses = [host.ip, remote.ip]
        for i, address in enumerate(addresses):
            child = subprocess.Popen(['ip', 'netns', 'exec', names[i], '/usr/bin/python3',
                str(ROOT/'scripts/service-probe.py'), 'server', '--address', address, '--seconds', '300'],
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
            children.append(child)
        for i in range(2):
            wait(lambda: exchange(i))
        ns(0, ['ip', 'link', 'delete', 'direct-a'])
        # The relay has no OS route between edge subnets and forwarding is off.
        assert relay.poll() is None
        expected_path = 'unreachable'
        wait(converged, seconds=90)
        for i in range(2):
            assert not exchange(i), 'Direct-only application traffic fell back to a relay'
        checks.append('Direct-only passes direct TCP/UDP, then blocks both directions after direct link removal while relay control remains alive')
        remote.close_controller()
        host.close_controller()
finally:
    for child in reversed(children):
        stop(child)
    for name in reversed(created):
        assert not command(['ip', 'netns', 'pids', name]).stdout.strip(), 'Owned process survived cleanup'
        command(['ip', 'netns', 'delete', name])
assert routes() == before_routes
assert pathlib.Path('/etc/resolv.conf').read_bytes() == before_dns
checks.append('No default/public underlay route; all owned resources removed; host routes and DNS unchanged')
report = {'engine_sha256': hashlib.sha256(pathlib.Path(engine).read_bytes()).hexdigest(),
          'transport': a.transport, 'relay_policy_lab': bool(a.relay_lab), 'checks_passed': checks,
          'limitations': ['Linux isolated stacks, not physical Windows-to-Mac or internet NAT evidence',
                          'Lab-only framed stdio; production authenticated IPC is tested separately']}
(ROOT/f'docs/evidence/engine-integration-{a.transport}{"-relay" if a.relay_lab else ""}.json').write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps(report, indent=2))
