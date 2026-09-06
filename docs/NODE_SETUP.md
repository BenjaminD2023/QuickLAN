# Optional nodes on hardware you already own

QuickLAN needs a shared reachable endpoint; it does not require a rented server
or a project-operated service. You can use an existing friend's computer as the
first contact point for a private group. This is a bootstrap peer, not a public
relay: QuickLAN does not forward application data between other devices.

1. On the host, create a network and select its actual Wi-Fi/Ethernet address.
   Connect and keep the window open while friends join. Same-router friends can
   normally use that local endpoint if the router and firewall permit it.
2. For remote friends, use an address they can reach. On a router you administer,
   an explicitly approved, narrowly scoped TCP or UDP port mapping to that host's
   port 11010 may provide reachability. Router configuration is external to
   QuickLAN; it makes no mappings automatically. CGNAT or restrictive networks
   may prevent this. Do not expose management services or disable a firewall.
3. Edit the group's connection settings to use that reachable endpoint and a
   truthful operator label, then share a fresh invitation with every member.
   Everyone must use the same network and addressing proposal. A changed address
   or offline host can prevent new joins; do not assume every established path
   survives a bootstrap outage.
4. Keep the host and its applications updated. Joining the group authorizes
   trusted credential holders to reach services permitted by the host firewall.
   The invitation is a bearer credential, not a public directory listing.

For optional **relay fallback**, an operator can instead provide a compatible
EasyTier shared node on existing reachable hardware. Follow the upstream
[shared-node operator guide](https://easytier.cn/en/guide/network/host-public-server.html)
and the pinned source findings in UPSTREAM_CAPABILITIES.md. QuickLAN's patched
2.6.4 engine requires **Secure Mode enabled on that node**. A plain legacy shared
node is not compatible. Do not give the operator your QuickLAN group credential;
the controlled relay tests use a separate nonmember network. Node labels do not
pin or authenticate the operator's key.

Use a dedicated unprivileged account and the upstream no-TUN relay configuration;
do not elevate a stock EasyTier process with its unauthenticated TCP management
surface. Expose only the intended peer transport listener, restrict management
access, and review the upstream service's own discovery, logging and update
behavior. QuickLAN's no-public-assistance guarantees do not configure an external
operator's independent software. Operators are responsible for permission to
use their connection, bandwidth costs/limits, patching and availability. No
unlimited free capacity or success through every NAT is promised.

Members enter the operator-approved endpoint and choose **P2P preferred, relay
permitted**, accepting the metadata and relay behavior. The operator can observe
public IPs, timing and volume, and can disrupt service. Direct-only permits
connection assistance but refuses application data when no direct path exists.
No shared node is selected or contacted until the member explicitly connects.
