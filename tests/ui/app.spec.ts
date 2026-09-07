import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";
async function installTestAdapter(page: Page) {
  await page.addInitScript(() => {
    const networks: Record<string, unknown>[] = [];
    let preferences = {
      nickname: "My computer",
      theme: "light",
      language: "en",
      onboarding_complete: false,
    };
    let connection = {
      phase: "disconnected",
      network_id: null as string | null,
      virtual_ip: null,
      peers: [],
      error: null as string | null,
    };
    let firewallProfiles = [
      { name: "Domain", enabled: true },
      { name: "Private", enabled: true },
      { name: "Public", enabled: false },
    ];
    let pending: Record<string, unknown> | null = null;
    const helper = {
      installed: false,
      connection_enabled: false,
      code: "helper_unavailable",
      release_gaps: [
        "Authenticated core management and signed OS helper installation are not implemented.",
        "Stock core retains implicit TCP STUN servers.",
      ],
    };
    const w = window as unknown as {
      __QUICKLAN_TEST_INVOKE: (
        c: string,
        a?: Record<string, unknown>,
      ) => Promise<unknown>;
    };
    w.__QUICKLAN_TEST_INVOKE = async (command, args = {}) => {
      if (command === "get_firewall_status")
        return structuredClone({
          platform: "windows",
          profiles: firewallProfiles,
        });
      if (command === "set_firewall_enabled") {
        if (!args.enabled && !args.confirmed)
          throw "firewall_confirmation_required";
        firewallProfiles = firewallProfiles.map((p) => ({
          ...p,
          enabled: Boolean(args.enabled),
        }));
        return structuredClone({
          platform: "windows",
          profiles: firewallProfiles,
        });
      }
      if (command === "get_local_endpoints")
        return [{ interface: "en0", endpoint: "tcp://192.168.1.12:11010" }];
      if (command === "get_state")
        return structuredClone({
          saved: { networks, preferences },
          connection,
          helper,
        });
      if (command === "save_preferences") {
        preferences = args.preferences as typeof preferences;
        return;
      }
      if (command === "create_network") {
        const n = {
          id: crypto.randomUUID().replaceAll("-", ""),
          label: args.label,
          subnet: args.subnet || "10.73.42.0/24",
          policy: args.policy,
          bootstrap: args.bootstrap,
        };
        networks.push(n);
        return n;
      }
      if (command === "connect_network") {
        connection = {
          ...connection,
          phase: "failed",
          network_id: args.id as string,
          error: "permission_denied",
        };
        return connection;
      }
      if (command === "disconnect_network") {
        connection = {
          phase: "disconnected",
          network_id: null,
          virtual_ip: null,
          peers: [],
          error: null,
        };
        return connection;
      }
      if (command === "preview_invitation") {
        if (args.token !== "test-only-invitation") throw "invalid_invitation";
        pending = {
          id: crypto.randomUUID().replaceAll("-", ""),
          label: "Game night",
          subnet: "10.92.16.0/24",
          policy: "manual",
          bootstrap: [
            { endpoint: "tcp://192.168.1.12:11010", operator: "Friend" },
          ],
        };
        return { ticket: "test-only-ticket", network: pending };
      }
      if (command === "accept_invitation") {
        if (!args.trusted || !pending) throw "confirmation_required";
        networks.push(pending);
        const n = pending;
        pending = null;
        return n;
      }
      if (command === "cancel_invitation") {
        pending = null;
        return;
      }
      if (command === "forget_network") {
        const i = networks.findIndex((n) => n.id === args.id);
        networks.splice(i, 1);
        return;
      }
      if (command === "rename_network") {
        networks.find((n) => n.id === args.id)!.label = args.label;
        return;
      }
      if (command === "update_settings") {
        Object.assign(networks.find((n) => n.id === args.id)!, {
          policy: args.policy,
          bootstrap: args.bootstrap,
        });
        return;
      }
      if (command === "get_diagnostics")
        return {
          product: "QuickLAN",
          app_version: "0.1.0",
          core_pin: "2.6.4-8428a89d",
          platform: "test",
          architecture: "test",
          phase: connection.phase,
          peer_count: 0,
          events: [],
          exclusion_notice: "TEST SIMULATION. No credentials or addresses.",
        };
      if (
        [
          "copy_invitation",
          "copy_diagnostics",
          "copy_virtual_ip",
          "quit_app",
        ].includes(command)
      )
        return;
      throw "unsupported_test_command";
    };
  });
}
test.beforeEach(async ({ page }) => {
  await installTestAdapter(page);
  await page.goto("/");
  await expect(page.getByText("TEST SIMULATION — no networking")).toBeVisible();
});
test("choose a local host endpoint and preserve it in saved connection settings", async ({
  page,
}) => {
  await page
    .getByRole("button", { name: "Create a network", exact: true })
    .click();
  await page.getByLabel("Network label", { exact: true }).fill("Local friends");
  await page
    .getByRole("button", { name: "Host on the same Wi-Fi or LAN" })
    .click();
  await page
    .getByLabel("This computer’s local endpoint")
    .selectOption("tcp://192.168.1.12:11010");
  await page
    .getByRole("button", { name: "Create and connect", exact: true })
    .last()
    .click();
  await page
    .getByRole("button", { name: /Connection policy Manual endpoints/ })
    .click();
  await expect(
    page.getByLabel("Reachable endpoint", { exact: true }),
  ).toHaveValue("tcp://192.168.1.12:11010");
  await expect(page.getByLabel("Node operator", { exact: true })).toHaveValue(
    "My computer",
  );
  await page
    .getByRole("combobox", { name: "Connection policy", exact: true })
    .selectOption("direct_only");
  await expect(
    page.getByRole("button", { name: "Save", exact: true }),
  ).toBeDisabled();
  await page
    .getByRole("checkbox", { name: /I accept the listed nodes for discovery/ })
    .check();
  await page.getByRole("button", { name: "Save", exact: true }).click();
  await expect(
    page.getByRole("button", {
      name: /Connection policy Discovery assistance; no relayed application data Direct-only application traffic/,
    }),
  ).toBeVisible();
});
test("create, real error presentation, invite, settings and local forget", async ({
  page,
}) => {
  await page
    .getByRole("button", { name: "Create a network", exact: true })
    .click();
  await page.getByLabel("Network label", { exact: true }).fill("Friday night");
  await expect(
    page.getByRole("button", { name: "Create and connect", exact: true }),
  ).toBeDisabled();
  await page
    .getByRole("button", { name: "Host on the same Wi-Fi or LAN" })
    .click();
  await page
    .getByLabel("This computer’s local endpoint")
    .selectOption("tcp://192.168.1.12:11010");
  await page
    .getByRole("button", { name: "Create and connect", exact: true })
    .last()
    .click();
  await expect(
    page.getByRole("heading", { name: "Friday night", exact: true }),
  ).toBeVisible();
  await expect(
    page.getByText("Connection unavailable", { exact: true }),
  ).toBeVisible();
  await page.screenshot({ path: "docs/evidence/ui-network.png" });
  await expect(
    page.getByText("No connected devices", { exact: true }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Invite", exact: true }).click();
  await expect(
    page.getByText("Anyone with this invitation may join.", { exact: true }),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "Copy invitation", exact: true })
    .click();
  await expect(page.getByRole("status")).toHaveText("Copied to clipboard");
  await page.getByRole("button", { name: "Close", exact: true }).click();
  await page
    .getByRole("button", { name: /Connection policy Manual endpoints/ })
    .click();
  await page
    .getByLabel("Reachable endpoint", { exact: true })
    .fill("tcp://192.168.1.12:11010");
  await page.getByLabel("Node operator", { exact: true }).fill("Friend");
  await page.getByRole("button", { name: "Save", exact: true }).click();
  await page
    .getByRole("button", { name: "Network actions", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Forget network", exact: true })
    .click();
  await expect(
    page.getByText(/Other members can still communicate/),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "Forget locally", exact: true })
    .click();
  await expect(
    page.getByRole("heading", { name: "Your friends. Your network." }),
  ).toBeVisible();
});
test("join is previewed and requires trust; malformed invitations remain errors", async ({
  page,
}) => {
  await page
    .getByRole("button", { name: "Join with invitation", exact: true })
    .first()
    .click();
  await page.getByLabel("Invitation token", { exact: true }).fill("malformed");
  await page
    .getByRole("button", { name: "Review invitation", exact: true })
    .click();
  await expect(page.getByRole("alert")).toContainText("malformed");
  await page
    .getByLabel("Invitation token", { exact: true })
    .fill("test-only-invitation");
  await page
    .getByRole("button", { name: "Review invitation", exact: true })
    .click();
  await expect(
    page.getByRole("button", { name: "Join and connect", exact: true }),
  ).toBeDisabled();
  await page.screenshot({ path: "docs/evidence/ui-join.png" });
  await page.getByRole("checkbox").check();
  await page
    .getByRole("button", { name: "Join and connect", exact: true })
    .click();
  await expect(
    page.getByRole("heading", { name: "Game night", exact: true }),
  ).toBeVisible();
  await expect(page.getByText("Connection unavailable", { exact: true })).toBeVisible();
});
test("keyboard, dark mode, diagnostics and narrow layout", async ({ page }) => {
  await page.screenshot({ path: "docs/evidence/ui-empty.png" });
  await page
    .getByRole("button", { name: "Create a network", exact: true })
    .click();
  await expect(page.getByLabel("Network label", { exact: true })).toBeFocused();
  await page.screenshot({ path: "docs/evidence/ui-create.png" });
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).not.toBeVisible();
  await page.getByRole("button", { name: "Preferences", exact: true }).click();
  await page.getByLabel("Theme", { exact: true }).selectOption("dark");
  await page
    .getByRole("button", { name: "Save preferences", exact: true })
    .click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
  await page.screenshot({ path: "docs/evidence/ui-dark.png" });
  await page.getByRole("button", { name: "Diagnostics", exact: true }).click();
  await expect(
    page.getByRole("button", { name: "Copy sanitized report", exact: true }),
  ).toBeEnabled();
  await page.setViewportSize({ width: 390, height: 844 });
  await page.screenshot({ path: "docs/evidence/ui-narrow.png" });
  const overflows = await page.evaluate(
    () => document.documentElement.scrollWidth > innerWidth,
  );
  expect(overflows).toBe(false);
});

test("live peer details, explicit TCP probe and peer disappearance", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.evaluate(() => {
    const peer = {
      id: "42",
      nickname: "A friend's device",
      virtual_ip: "10.73.42.2",
      path: "direct",
      latency_ms: 4.2,
      identity_verified: false,
    };
    const view = {
      saved: {
        networks: [
          {
            id: "a".repeat(32),
            label: "Live network",
            subnet: "10.73.42.0/24",
            policy: "manual",
            bootstrap: [],
          },
        ],
        preferences: {
          nickname: "This device",
          theme: "light",
          language: "en",
          onboarding_complete: true,
        },
      },
      connection: {
        phase: "connected",
        network_id: "a".repeat(32),
        virtual_ip: "10.73.42.1",
        peers: [peer],
        error: null,
      },
      helper: {
        installed: true,
        connection_enabled: true,
        code: null,
        release_gaps: [],
      },
    };
    let probes = 0;
    const win = window as unknown as {
      __QUICKLAN_TEST_INVOKE: (
        command: string,
        args?: Record<string, unknown>,
      ) => Promise<unknown>;
      removePeer: () => void;
    };
    win.removePeer = () => {
      view.connection.peers = [];
    };
    win.__QUICKLAN_TEST_INVOKE = async (command, args) => {
      if (command === "get_state") return structuredClone(view);
      if (command === "probe_service") {
        if (args?.peerId !== "42" || args?.port !== 25565)
          throw "invalid_service";
        probes++;
        return probes === 1 ? "refused" : "reachable";
      }
      throw "unsupported_test_command";
    };
  });
  await expect(page).toHaveTitle("QuickLAN");
  await expect(
    page.getByRole("heading", { name: "Live network" }),
  ).toBeVisible();
  await page.getByRole("button", { name: /A friend's device/ }).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog.getByText("4.2 ms")).toBeVisible();
  await dialog.getByLabel("TCP port").fill("25565");
  await dialog
    .getByRole("button", { name: "Check application port", exact: true })
    .click();
  await expect(dialog.getByRole("status")).toContainText("Connection refused");
  await dialog
    .getByRole("button", { name: "Check application port", exact: true })
    .click();
  await expect(dialog.getByRole("status")).toContainText(
    "TCP connection accepted",
  );
  await page.screenshot({ path: "/tmp/quicklan-peer-probe.png" });
  await page.evaluate(() =>
    (window as unknown as { removePeer: () => void }).removePeer(),
  );
  await expect(
    page
      .getByRole("dialog")
      .getByText("This peer is no longer in the current network state."),
  ).toBeVisible();
  expect(errors).toEqual([]);
});

test("firewall confirmation, cancel, disable and enable reflect OS results", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Preferences", exact: true }).click();
  const panel = page.getByRole("region", { name: "System firewall" });
  await expect(panel.getByText("Enabled", { exact: true })).toHaveCount(2);
  await expect(panel.getByText("Disabled", { exact: true })).toHaveCount(1);
  await panel
    .getByRole("button", { name: "Disable firewall", exact: true })
    .click();
  let dialog = page.getByRole("dialog");
  await expect(
    dialog.getByRole("button", { name: "Disable firewall", exact: true }),
  ).toBeDisabled();
  await dialog.getByRole("button", { name: "Cancel", exact: true }).click();
  await expect(panel.getByText("Enabled", { exact: true })).toHaveCount(2);
  await panel
    .getByRole("button", { name: "Disable firewall", exact: true })
    .click();
  dialog = page.getByRole("dialog");
  await dialog.getByRole("checkbox").check();
  await dialog
    .getByRole("button", { name: "Disable firewall", exact: true })
    .click();
  await expect(dialog).not.toBeVisible();
  await expect(panel.getByText("Disabled", { exact: true })).toHaveCount(3);
  await panel
    .getByRole("button", { name: "Enable firewall", exact: true })
    .click();
  await expect(panel.getByText("Enabled", { exact: true })).toHaveCount(3);
  await expect(
    panel.getByText(
      "The operating system reports the requested firewall state.",
    ),
  ).toBeVisible();
});
test("firewall denial and unreadable state never report false success", async ({
  page,
}) => {
  await page.evaluate(() => {
    const original = window.__QUICKLAN_TEST_INVOKE!;
    window.__QUICKLAN_TEST_INVOKE = async (c, a) => {
      if (c === "set_firewall_enabled") throw "firewall_change_failed";
      return original(c, a);
    };
  });
  await page.getByRole("button", { name: "Preferences", exact: true }).click();
  const panel = page.getByRole("region", { name: "System firewall" });
  await panel
    .getByRole("button", { name: "Enable firewall", exact: true })
    .click();
  await expect(panel.getByRole("alert")).toContainText("cancelled");
  await expect(panel.getByText("Disabled", { exact: true })).toHaveCount(1);
  await page.evaluate(() => {
    const original = window.__QUICKLAN_TEST_INVOKE!;
    window.__QUICKLAN_TEST_INVOKE = async (c, a) => {
      if (c === "get_firewall_status") throw "firewall_unavailable";
      return original(c, a);
    };
  });
  await panel.getByRole("button", { name: "Refresh", exact: true }).click();
  await expect(
    panel.getByRole("button", { name: "Disable firewall", exact: true }),
  ).toBeDisabled();
  await expect(
    panel.getByRole("button", { name: "Enable firewall", exact: true }),
  ).toBeDisabled();
});
