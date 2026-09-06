# Design implementation and visual verification

Concept: `docs/design-concept.png`, generated before frontend implementation. Browser evidence: `docs/evidence/ui-empty.png`, `ui-create.png`, `ui-network.png`, `ui-join.png`, `ui-dark.png` and `ui-narrow.png`. These browser captures have an explicit **TEST SIMULATION — no networking** strip. The production bundle test confirms both the mock hook and strip are absent.

The concept and latest captures were opened and compared directly on 2026-09-06. The in-app browser was also used to inspect the real production UI entry, create dialog and advanced settings without a mock fallback. Native rendering is recorded separately in `evidence/native-desktop.json`.

| Anchor | Observed result |
|---|---|
| Shell proportions | 220px pale-gray rail, fine border and white content match the concept's desktop utility composition. |
| Accent and hierarchy | Teal brand and primary actions, dark readable headings, muted support text and restrained amber notices match the selected direction. |
| Empty state | Centered headline, two-line explanatory copy and vertically stacked create/join actions preserve the hierarchy. Trust copy was added below the actions to satisfy onboarding. |
| Network view | Disconnected state, unavailable virtual address, empty peer region and contextual helper warning remain honest; no dummy peers or metrics appear in production. |
| Create dialog | Centered native HTML dialog, real label/nickname inputs, advanced disclosure and separate cancel/submit actions. Advanced transport details stay collapsed initially. |
| Invitation preview | Readable label/subnet/policy/operator rows, amber bearer warning, explicit trust checkbox and disabled submit before consent match the concept. |
| Bottom rail actions | Diagnostics, game/app guide and Quit and disconnect are intentional additions needed by the product brief. The concept only showed Preferences/About here. |
| Native controls and typography | System font and lucide SVG glyphs replace image-generated approximations. All product text is selectable HTML; the concept is not used as UI. |
| Error behavior | Missing helper and malformed invitation produce visible, sanitized errors; no success animation masks backend absence. |
| Accessibility and adaptability | Tests cover input focus, Escape dismissal, native dialog semantics, dark mode and no horizontal overflow at 390px. Explicit labels fix dynamic textarea/select label ambiguity found during testing. Reduced-motion CSS is included. |

No major unresolved visual mismatch was observed in the reviewed browser views. This does not imply all keyboard/screen-reader combinations, translated strings, Windows WebView2 or Intel Mac rendering have been manually verified. Native networking and helper permission dialogs cannot be visually approved because those integrations are not implemented.
