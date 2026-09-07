# Explicit firewall controls — 0.2.1

Preferences → System firewall reads the OS state and offers Disable firewall and
Enable firewall. Disabling requires an explicit acknowledgement, followed by OS
administrator authorization. It affects the whole computer and remains in effect
after disconnect, quit, restart or uninstall. Enable firewall turns protection on;
it enables every Windows profile rather than restoring a previous mixed state.
Existing per-application rules are retained. No automatic firewall changes occur
when connecting, disconnecting, starting or quitting QuickLAN.

Windows uses the OS system-directory `netsh.exe` with `runas` to set all three
Defender Firewall profiles (Domain, Private, Public). It reads effective policy
with `Get-NetFirewallProfile -PolicyStore ActiveStore` from the OS NetSecurity
module, and verifies each profile after mutation. macOS uses the built-in
`socketfilterfw` to control Application Firewall, with AppleScript administrator
authorization, then verifies `--getglobalstate`. Only fixed on/off commands are
accepted: renderer input cannot provide a shell command, executable path or rule.
Concurrent mutations are serialized. Cancellation, unreadable state and a policy
that prevents the requested state produce errors, never a false success.

This does not control third-party security software or macOS PF rules, open a
router port, solve NAT traversal, bypass device management, or guarantee a game
can connect. Windows Group Policy and macOS management may enforce protection.
A local change can partially apply before an error; the UI reads actual state
again and offers Refresh. It does not alter firewall rules or stop the Windows
firewall service. macOS block-all behavior follows the OS global switch.

Automated acceptance uses disposable Windows and macOS runners, invokes the same
runtime functions as the desktop, checks confirmation refusal, disables/enables
real firewall state and restores the original settings in a `finally` block.
Original per-profile Windows settings and macOS block-all mode are restored by
the test harness. The user's Mac is not used for live firewall mutation tests.
Browser tests cover confirmation/cancel, mixed state, disable/enable, denial and
unreadable state. Interactive ordinary-user permission dialogs and organization
management policies require additional physical-device testing.

OS references: [Microsoft firewall profiles](https://learn.microsoft.com/en-us/powershell/module/netsecurity/get-netfirewallprofile)
and [Apple application firewall](https://support.apple.com/guide/mac-help/mh11783/mac).
