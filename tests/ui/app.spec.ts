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
          subnet: args.subnet,
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
          error: "helper_unavailable",
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
test("create, real error presentation, invite, settings and local forget", async ({
  page,
}) => {
  await page
    .getByRole("button", { name: "Create a network", exact: true })
    .click();
  await page.getByLabel("Network label", { exact: true }).fill("Friday night");
  await page
    .getByRole("button", { name: "Create network", exact: true })
    .last()
    .click();
  await expect(
    page.getByRole("heading", { name: "Friday night", exact: true }),
  ).toBeVisible();
  await expect(page.getByText("Disconnected", { exact: true })).toBeVisible();
  await page.screenshot({ path: "docs/evidence/ui-network.png" });
  await page.getByRole("button", { name: "Connect", exact: true }).click();
  await expect(
    page.getByText("Connection unavailable", { exact: true }),
  ).toBeVisible();
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
  await page.getByRole("button", { name: "Add node", exact: true }).click();
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
    page.getByRole("button", { name: "Save network", exact: true }),
  ).toBeDisabled();
  await page.screenshot({ path: "docs/evidence/ui-join.png" });
  await page.getByRole("checkbox").check();
  await page.getByRole("button", { name: "Save network", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Game night", exact: true }),
  ).toBeVisible();
  await expect(page.getByText("Disconnected", { exact: true })).toBeVisible();
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
