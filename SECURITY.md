# Security policy

QuickLAN 0.1.0 is an engineering preview. No release is currently supported for production networking. Critical release gates are listed in docs/TEST_MATRIX.md and docs/UPSTREAM_CAPABILITIES.md.

Do not post exploitable details, bearer tokens, logs, memory dumps or packet payloads in a public issue. Public hosting and a private security reporting channel are not configured. The repository owner must enable GitHub private vulnerability reporting and publish the contact before public launch. Until then, report privately to the person who supplied this local checkout.

There is no automatic update mechanism. Checksums detect mismatch with a pinned artifact; they do not establish publisher identity or safety. Signing, protected releases and rollback procedures are separate gates. Maintain the vulnerability evidence and do not interpret a clean scan as a security audit.

The stock core has an IP-whitelisted management TCP endpoint and implicit TCP STUN behavior. QuickLAN therefore does not launch it with privilege or expose a Connect bypass. Do not weaken these gates to make a demo look operational.
