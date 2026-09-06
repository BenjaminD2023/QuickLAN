#!/usr/bin/env python3
"""Real EasyTier test: userspace virtual-address TCP/UDP, NOT host TUN evidence.

macOS only: sandbox-exec confines all core outbound traffic to loopback/Unix IPC.
No elevated processes, firewall changes, public test nodes or persistent secrets.
Stock core RPC is NOT authenticated. Run only in a trusted developer session.
"""
import argparse, hashlib, json, os, pathlib, platform, secrets, socket, subprocess, tempfile, threading, time
ROOT = pathlib.Path(__file__).resolve().parents[1]
LOCK = json.loads((ROOT/'upstream/easytier.lock.json').read_text())
SANDBOX = '(version 1)(allow default)(deny network-outbound)(allow network-outbound (remote ip "localhost:*") (remote unix-socket))'

def available_port():
    with socket.socket() as s:
        s.bind(('127.0.0.1', 0))
        return s.getsockname()[1]

def verify(binary, expected):
    if not binary.is_file() or binary.is_symlink():
        raise RuntimeError('Missing regular core binary. Run python3 scripts/fetch-core.py.')
    if hashlib.sha256(binary.read_bytes()).hexdigest() != expected:
        raise RuntimeError('Core digest mismatch; refusing execution.')

def wait_for(check, seconds=18):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        if check(): return
        time.sleep(.3)
    raise AssertionError('Timed out waiting for real peer state')

class Echo:
    def __init__(self, kind):
        self.sock = socket.socket(socket.AF_INET, kind)
        self.sock.bind(('127.0.0.1', 0))
        self.port = self.sock.getsockname()[1]
        self.sock.settimeout(.2)
        self.stop = threading.Event()
        if kind == socket.SOCK_STREAM: self.sock.listen(8)
        def run():
            while not self.stop.is_set():
                try:
                    if kind == socket.SOCK_STREAM:
                        conn, _ = self.sock.accept()
                        with conn:
                            conn.settimeout(2)
                            value = conn.recv(2048)
                            conn.sendall(value)
                    else:
                        value, addr = self.sock.recvfrom(2048)
                        self.sock.sendto(value, addr)
                except (TimeoutError, OSError): pass
        self.thread = threading.Thread(target=run, daemon=True)
        self.thread.start()
    def close(self):
        self.stop.set(); self.sock.close(); self.thread.join(3)

def run(transport):
    if platform.system() != 'Darwin' or platform.machine() != 'arm64':
        raise SystemExit('This executable test currently requires macOS ARM64 with sandbox-exec. See docs/TEST_MATRIX.md for native procedures.')
    core = ROOT/'.cache/easytier-macos-aarch64/easytier-core'
    cli = core.with_name('easytier-cli')
    for p in (core, cli): verify(p, LOCK['verified_binaries']['aarch64-apple-darwin/'+p.name])
    ver = subprocess.check_output([str(core), '--version'], text=True).strip()
    assert ver == LOCK['version_output'], 'Unsupported core version'
    start = time.monotonic()
    results=[]; processes={}; services=[]
    with tempfile.TemporaryDirectory(prefix='quicklan-integration-') as td:
        td = pathlib.Path(td); td.chmod(0o700)
        env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin', 'HOME':str(td), 'LANG':'en_US.UTF-8'}
        secret = secrets.token_hex(32); identity = secrets.token_hex(16)
        listen=[available_port() for _ in range(4)]; rpc=[available_port() for _ in range(4)]
        forwarded=[[available_port(),available_port()] for _ in range(4)]
        config=[]
        for _ in range(2): services.append([Echo(socket.SOCK_STREAM), Echo(socket.SOCK_DGRAM)])
        def spawn(i):
            processes[i] = subprocess.Popen(['/usr/bin/sandbox-exec','-p',SANDBOX,str(core),
                '--config-file',str(config[i]),'--rpc-portal',f'127.0.0.1:{rpc[i]}',
                '--secure-mode','true','--disable-env-parsing','--console-log-level','off'],
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=env, cwd=td)
        def stop(i):
            p=processes.pop(i)
            p.terminate()
            try: p.wait(timeout=5)
            except subprocess.TimeoutExpired: p.kill(); p.wait(timeout=5)
        def query(i, cmd):
            if processes[i].poll() is not None: raise AssertionError('Core terminated unexpectedly')
            p=subprocess.run([str(cli),'-p',f'127.0.0.1:{rpc[i]}','-o','json',cmd],
                capture_output=True,timeout=4,env=env)
            if p.returncode: return None
            if len(p.stdout)>1_048_576: raise AssertionError('Unexpectedly large core output')
            return json.loads(p.stdout)
        def peers(i):
            value=query(i,'peer')
            return value if isinstance(value,list) else []
        def exchange(i, proto):
            kind=socket.SOCK_STREAM if proto==0 else socket.SOCK_DGRAM
            payload=b'quicklan-test-'+secrets.token_bytes(32)
            with socket.socket(socket.AF_INET,kind) as s:
                s.settimeout(4)
                s.connect(('127.0.0.1',forwarded[i][proto]))
                s.sendall(payload)
                assert s.recv(2048)==payload, 'Payload did not survive overlay round trip'
        try:
            for i in range(4):
                group = secrets.token_hex(16) if i==3 else identity
                credential = secrets.token_hex(32) if i==2 else secret
                cfg=f'''instance_name = "quicklan-lab-{i}"
hostname = "lab-{i}"
ipv4 = "10.73.42.{i+1}/24"
dhcp = false
listeners = ["{transport}://127.0.0.1:{listen[i]}"]
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
no_tun = true
use_smoltcp = true
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
                if i: cfg+=f'\n[[peer]]\nuri = "{transport}://127.0.0.1:{listen[0]}"\n'
                if i<2:
                    for proto in range(2):
                        cfg+=f'\n[[port_forward]]\nbind_addr = "127.0.0.1:{forwarded[i][proto]}"\ndst_addr = "10.73.42.{2-i}:{services[1-i][proto].port}"\nproto = "{["tcp","udp"][proto]}"\n'
                p=td/f'{i}.toml'
                fd=os.open(p,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
                with os.fdopen(fd,'w') as f: f.write(cfg)
                config.append(p); spawn(i)
            wait_for(lambda: len(peers(0))==2 and len(peers(1))==2)
            results.append('correct-secret peer join: PASS')
            for i in range(2):
                node=query(i,'node')
                # Raw node JSON includes config/credentials. Only copy an explicit safe projection.
                assert node['ipv4_addr']==f'10.73.42.{i+1}/24'
                assert node['stun_info']['public_ip']==[], 'Unexpected public discovery despite confinement'
                remote=[x for x in peers(i) if x['cost']!='Local']
                assert len(remote)==1 and remote[0]['cost']=='p2p'
                for proto in range(2):
                    exchange(i,proto)
                    results.append(f'{["TCP","UDP"][proto]} payload {i} -> {1-i} -> {i}: PASS')
                # The peer JSON includes only ephemeral private overlay addresses/names/counters.
                (ROOT/f'tests/fixtures/peer-{i}.json').write_text(json.dumps(peers(i),indent=2)+'\n')
            time.sleep(3)
            assert len(peers(2))==1 and len(peers(3))==1
            assert all(x['hostname'] in ('lab-0','lab-1') for x in peers(0))
            results += ['wrong secret excluded: PASS','separate network excluded: PASS','OS-confined public discovery empty: PASS']
            stop(1)
            wait_for(lambda: len(peers(0))==1)
            spawn(1)
            wait_for(lambda: len(peers(0))==2 and len(peers(1))==2)
            for i in range(2):
                for proto in range(2): exchange(i,proto)
            results.append('disconnect, restart and bidirectional TCP/UDP: PASS')
        finally:
            for i in list(processes): stop(i)
            for group in services:
                for s in group: s.close()
        assert not processes
        results.append('owned child processes reaped and private temporary configs removed on exit: PASS')
    assert not td.exists()
    report={'date':time.strftime('%Y-%m-%d'),'platform':platform.platform(),'core':ver,
        'topology':f'two real no-TUN cores on one Mac, loopback {transport} underlay; two negative peers',
        'results':results,'elapsed_seconds':round(time.monotonic()-start,2),
        'limitations':['NOT host virtual-IP/TUN connectivity','NOT internet NAT traversal','NOT cross-platform verification',
            'NOT direct-only data-plane or forced-relay validation','stock management RPC unauthenticated; trusted developer session only']}
    path=ROOT/f'docs/evidence/core-integration-{transport}.json'
    path.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--transport',choices=['tcp','udp'],default='tcp')
    run(parser.parse_args().transport)
