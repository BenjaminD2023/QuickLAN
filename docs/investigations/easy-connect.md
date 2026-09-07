# Automatic connection investigation — 2026-09-07

The released 0.2.1 UI defaulted to manual policy with an empty endpoint list and did not render asynchronous connection errors. The current Mac uses Internet VPN capture routes 8.0.0.0/5 and 128.0.0.0/1; the old route-overlap algorithm rejected every RFC1918 range under that VPN.

Primary service sources reviewed:
- https://github.com/EasyTier/EasyTier/issues/2448 — maintainer KKRainbow explicitly says the project does not provide a service (July 2026). Do not seed the retired public.easytier.top endpoint.
- https://raw.githubusercontent.com/EasyTier/easytier.github.io/main/en/guide/network/secure-mode.md — secure clients require secure-mode servers. Keep secure mode enabled.
- https://easytier.dreamlife.indevs.in/ — operator offers public EasyTier peer tcp://dreamlife.indevs.in:11010.
- https://docs.one-kvm.cn/feature_usage/easytier_mesh/ — operator publishes two free public peers, no availability or latency promise; prohibits abuse, traffic flooding, attacks and bulk resource use. Peers published in the page's public config script: tcp://154.94.237.219:11010 and tcp://79.127.129.149:11010.

Compatibility probes use two synthetic random network identities and strong credentials, no TUN device, no system routes, no STUN/default discovery, no forwarding for other networks, no listeners, and a bounded 45-second discovery attempt. No personal network/invitation data is used.

Observed results on this Mac:
- All three advertised public endpoints accepted TCP connections.
- All three failed secure discovery after 45 seconds (zero secure relay peers); no payload was sent without successful authenticated discovery.
- The same probe against a fresh secure nonmember relay on loopback passed bidirectional encrypted IPv4/UDP payload transfer with direct links disabled.
- These observations establish that the probe works with a compatible relay. They do not establish the specific cause of each public service failure or prove that no compatible community service exists.

Automatic internet setup remains blocked by the absence of a verified compatible service. Do not describe the corrected UI or LAN workflow as Radmin-equivalent internet networking, and do not ship the tested incompatible public addresses as defaults.
