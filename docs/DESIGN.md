# QuickLAN desktop design

Reference: `design-concept.png`, generated with the built-in Image Gen tool on 2026-09-06. It is a four-state design board, not a shipped UI or proof of networking. The frontend-app-builder and imagegen skills guide implementation. No raster artwork is needed in the app: the network glyph is a functional lucide icon, all text/controls are native React.

Direction: a small desktop utility with a 220px light-gray network rail, true-white content, charcoal text, teal primary actions (#126d63), subtle borders (#dfe3e5), 8px control radii and system sans typography. Main text 14px/1.5, headings 26px semibold, labels 12px medium. Spacing 4/8/12/16/24/32/48. Icons 18px outline, 1.7px stroke. No gradients, KPI cards, art assets, marketing page or fabricated peers.

Primary component families: sidebar rows, buttons (primary/secondary/danger/quiet), input fields, modal dialogs, segmented themes, inline notices, peer rows, definition lists. Focus rings must be visible. HTML dialog supplies focus containment; every input is labeled. Disable only controls that cannot truthfully run; error notices explain the reason.

Screens prepared before implementation:

1. Empty/onboarding: “Your friends. Your network.”, create or paste invitation. Explain trusted peers/listening services and engineering gates. No connection on load.
2. Saved network: heading, invite and connect controls; disconnected/starting/joining/connected/reconnecting/stopping/failed states; virtual IP only from backend; real peer list or empty. Helper missing/error occupies inline notice.
3. Create: network label, nickname (global preferences), advanced subnet and shared endpoints with operator field; manual/assisted modes and required assistance consent. Direct-only visibly unavailable.
4. Join token then preview: label, subnet, endpoints/operators, trusted-group warning, explicit confirmation. Saving does not connect.
5. Peer detail: self-reported nickname, virtual address, path and available measured latency. No identity verification implied.
6. Diagnostics: sanitized preview, copy/export by user action; actionable helper/traversal/port guide.
7. Preferences: local nickname, system/light/dark and English/Chinese. No startup connection or invisible background networking in this engineering build.
8. About/licenses and help: pin, attribution, privacy and beta gaps. Explicit Quit and disconnect. Window close also disconnects; no tray behavior yet.
9. Network actions: rename, replacement credentials (old network remains), forget locally. Confirmation copy accurately explains membership semantics.

Dark theme uses the same structure with neutral charcoal surfaces and readable teal. Small viewports compact the rail and preserve readable dialogs. This is a desktop product; responsive browser checks are layout tests only.

Intentional extensions from the concept: a real saved-network list in the sidebar, diagnostics/help actions required by the brief, network actions, accessible dialogs and pending-operation states. Desktop title chrome uses actual OS decorations instead of drawing fake minimize/close buttons. The footer states the tested build limitation. No connect simulation is ever substituted.
