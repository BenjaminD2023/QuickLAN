# Architecture and decisions

QuickLAN is an engineering preview of a trusted-group virtual network utility. It uses Tauri 2, React, TypeScript, Vite and a Rust application domain. A pinned EasyTier process is the intended network engine; the stock release is currently confined to explicit developer tests because its management and networking-policy gaps block elevated production use.

```text
React UI (rendering, forms, typed requests; no credential persistence)
  -> allowlisted local-window Tauri commands
    -> Mutex<App<OsStore>>: validation, local persistence, invitations, lifecycle
      -> per-network OS credential entry + private metadata file
      -> sanitized event codes (100-entry memory ring)
      -> helper boundary [closed: platform implementation is a release gate]
          -> pinned EasyTier process [developer test harness only]
```

- One active network per application. A single-instance plugin prevents duplicate UI processes. The backend mutex serializes commands; lifecycle generations reject stale results after stop/restart. A failed core/helper operation clears peers and virtual IP. Unknown or unsupported JSON is an error, not invented state.
- Friendly labels and nicknames are distinct from a random 128-bit network identifier. Secrets contain 256 bits of OS randomness encoded as lowercase hex. Invitations are `quicklan1:` followed by bounded base64url JSON with exact schema/protocol validation. Their preview exposes no credential, and its one-use local ticket binds user confirmation to the parsed content. The ticket is a local UI confirmation mechanism, **not a one-use network invitation**.
- Persistent secrets use macOS Keychain / Windows Credential Manager, one entry per opaque network ID. JSON metadata contains names/settings but no credentials. Secrets are zeroized where owned; temporary parser/serializer/clipboard/OS copies can remain. No memory-protection guarantee is claimed.
- Create saves the credential before atomic metadata commit; failure attempts credential cleanup. Forget removes the credential and restores it on metadata failure. A crash or failed rollback can leave an orphan credential or a saved entry with unavailable credentials. Fail clearly; do not silently regenerate credentials. Full cross-store transactional recovery is still a release gate.
- No raw upstream output is persisted. Source inspection found that core info logs include TOML credentials. The lab redirects upstream logs to the null device, and only exports an explicit status projection. Diagnostics contain fixed error codes/state, never arbitrary exception strings.
- The UI refreshes native state every three seconds. There is no automatic connection, account, cloud control plane, exit node, DNS change, file sharing, chat or automatic update check. Window close and explicit quit disconnect; tray/background operation is intentionally absent.
- Public endpoint directory is empty. Saved assisted profiles require a user-specified operator, shared node and consent. Manual profile promises remain unavailable for system networking until the implicit TCP STUN issue is resolved. Direct-only profile is rejected at the backend, not merely disabled visually.
- `routes=[]` is a candidate route-installation safeguard verified in upstream source, not a substitute for rejecting hostile data-plane advertisements. Route range checks and ownership models are unit-tested; live physical route enumeration and protected application are unfinished.
- Original wrapper code: Apache-2.0. Separate upstream process: LGPL-3.0. Stock core not bundled in the engineering app. No affiliation or independent audit claim.

See platform/macos and platform/windows for concrete privilege-boundary requirements and docs/UPSTREAM_CAPABILITIES.md for evidence. Network startup stays closed until those requirements are implemented and tested; this is a deliberate security gate, not a silent mock adapter.
