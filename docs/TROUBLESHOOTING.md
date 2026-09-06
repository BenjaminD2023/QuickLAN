# Troubleshooting

**Getting friends onto the same network.** On the same router, choose Create →
Host on the same Wi-Fi or LAN, select the real Wi-Fi/Ethernet address, then connect
and share the invitation. Remote friends need a reachable IP endpoint or a
compatible, operator-approved shared node in connection settings. A private LAN
address is normally unreachable from the internet. There is no automatic public
node selection, so an invitation without an endpoint cannot discover friends.
Both ends need compatible QuickLAN protocol/core settings and the same invitation.
Shared EasyTier nodes must enable Secure Mode. Do not disable firewalls globally.

**Permission refused.** The window stays unprivileged. Connect requests OS
permission for the separate adapter-owning process. Denial leaves the network
disconnected; retry Connect when ready. Canceling while an OS prompt is displayed
may require dismissing that prompt. Do not launch the entire GUI as root/admin.

**Helper unavailable or incompatible.** The packaged helper or its exact digest
is missing/mismatched. Disconnect and reinstall the matching complete package.
Do not copy a stock easytier-core executable into its place. For source builds,
prepare, build and stage the engine before building the desktop. There is no
persistent service to register or repair manually.

**Route conflict.** QuickLAN rejects overlapping LAN, VPN and container routes,
including a VPN's split-default routes. It will not silently change one member's
address range. Create a new network with a nonoverlapping private /24 in connection
settings and have every member migrate to the new invitation. Disconnect before
changing another VPN's route configuration. Old members can still use the old group.

**Connected but no friends.** Connected means the local adapter is ready, not
that a friend or game is reachable. Check that the friend is connected, the shared
endpoint is online and its selected TCP/UDP port is allowed. Host endpoints use
11010. A machine's physical IP can change; edit the endpoint and redistribute an
invitation if it does. Blank latency is unavailable evidence, not zero latency.

**A game/app cannot connect.** Start the host application after QuickLAN connects.
Use its current virtual IP and the documented application port. Peer details →
Check application port opens one TCP connection with no payload. Refused means
a listener/firewall rejected it; timeout can mean filtering or a broken path.
UDP-only games need an actual in-game test. Allow only the intended application
or port and overlay peers. A DHCP virtual IP may change after reconnect; restart
or rebind applications using the old address. Broadcast/mDNS discovery is absent.

**No public assistance.** Uses explicit IP endpoints without implicit public STUN,
external-IP lookups or DNS fallback. Arbitrary NATed remote peers may fail.
Assisted mode may use configured relay fallback after consent. Direct-only never
permits relayed application traffic; if the direct path fails it must stop traffic.
No mode guarantees connectivity, bandwidth or latency on every internet topology.

**Unexpected stop.** Close/Quit disconnects. Controller loss, heartbeat failure,
malformed IPC, missing adapter or invalid peer addressing stops the engine.
Wait for cleanup before reconnecting. No invisible tray session is intended.

**Credential store unavailable.** Unlock the normal user session and allow its
vault operation. Do not put secrets into JSON as a workaround. Metadata lives in
Tauri's user app-data directory for `io.quicklan.desktop`; secret entries use
`io.quicklan.desktop.network`. A crash between stores can require forgetting and
re-importing a known-good invitation; never silently regenerate a credential.

**Invitation rejected.** Use an intact `quicklan1:` token of at most 8192 bytes.
Unknown schemas, noncanonical private /24 ranges and unsupported endpoint schemes
are rejected. Raw EasyTier configs are not invitations. Preview makes no connection.

**Unsigned application warnings.** Mac builds have ad-hoc integrity signatures,
not Developer ID/notarization. Windows application/installer are unsigned. OS
warnings or blocking can occur. Do not disable Gatekeeper or SmartScreen globally.
Signing credentials and ordinary-user installation tests remain outstanding.

**Diagnostics.** Preview and explicitly copy the fixed-field report. No automatic
upload occurs. Credentials, labels, addresses and raw upstream logs are excluded.
Browser-only `npm run dev` has no native engine; use the installed app or desktop:dev.
