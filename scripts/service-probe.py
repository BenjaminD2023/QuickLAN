#!/usr/bin/env python3
"""Explicit TCP/UDP echo probe for consented real overlay addresses; no firewall edits."""
import argparse
import contextlib
import ipaddress
import secrets
import socket
import threading
import time

p = argparse.ArgumentParser()
p.add_argument('mode', choices=['server', 'client'])
p.add_argument('--address', required=True, help='Actual assigned IPv4 address of the server')
p.add_argument('--port', type=int, default=39191)
p.add_argument('--seconds', type=int, default=60, help='Bounded server lifetime, at most 300 seconds')
p.add_argument('--loopback-lab', action='store_true', help='Explicitly permit localhost for testing this probe, not overlay evidence')
a = p.parse_args()
ip = ipaddress.ip_address(a.address)
allowed = any(ip in ipaddress.ip_network(c) for c in ['10.0.0.0/8', '172.16.0.0/12', '192.168.0.0/16'])
allowed = allowed or (a.loopback_lab and ip.is_loopback)
if ip.version != 4 or not allowed or not 1024 <= a.port <= 65535 or not 1 <= a.seconds <= 300:
    p.error('Use an explicit RFC1918 IPv4 address, unprivileged port and bounded duration')
endpoint = (str(ip), a.port)
if a.mode == 'client':
    payload = secrets.token_bytes(64)
    for kind, name in [(socket.SOCK_STREAM, 'TCP'), (socket.SOCK_DGRAM, 'UDP')]:
        with socket.socket(socket.AF_INET, kind) as s:
            s.settimeout(5)
            s.connect(endpoint)
            s.sendall(payload)
            data = b''
            while len(data) < len(payload):
                part = s.recv(1024)
                if not part:
                    break
                data += part
            if data != payload:
                raise SystemExit(f'{name}: payload mismatch')
            print(f'{name}: exact 64-byte echo passed')
else:
    stop = threading.Event()
    def serve(s, kind):
        while not stop.is_set():
            try:
                if kind == socket.SOCK_DGRAM:
                    data, remote = s.recvfrom(1024)
                    s.sendto(data, remote)
                else:
                    conn, _ = s.accept()
                    with conn:
                        conn.settimeout(1)
                        remaining = 64
                        while remaining > 0:
                            chunk = conn.recv(remaining)
                            if not chunk:
                                break
                            conn.sendall(chunk)
                            remaining -= len(chunk)
            except (socket.timeout, ConnectionError):
                continue
    with contextlib.ExitStack() as stack:
        sockets = []
        for kind in [socket.SOCK_STREAM, socket.SOCK_DGRAM]:
            s = stack.enter_context(socket.socket(socket.AF_INET, kind))
            s.settimeout(0.5)
            # The acceptance test restarts its own listener on reconnect. Allow
            # reuse after a previous TCP connection enters TIME_WAIT.
            s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            s.bind(endpoint)  # Binding failures terminate before reporting readiness.
            if kind == socket.SOCK_STREAM:
                s.listen(4)
            sockets.append((s, kind))
        workers = [threading.Thread(target=serve, args=pair) for pair in sockets]
        for worker in workers:
            worker.start()
        print('TCP and UDP probe listeners ready; firewall configuration unchanged.', flush=True)
        try:
            time.sleep(a.seconds)
        except KeyboardInterrupt:
            pass
        finally:
            stop.set()
            for worker in workers:
                worker.join()
