import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { errorText, en } from "../src/lib/i18n";
import { emptyView } from "../src/lib/types";
describe("frontend trust boundary", () => {
  it("does not display raw backend errors or bearer tokens", () => {
    for (const value of [
      "quicklan1:private-bearer",
      "network_secret=not-for-ui",
      { message: "private value" },
      new Error("private value"),
    ])
      expect(errorText(value)).not.toContain("private");
  });
  it("starts without fabricated network state or unavailable-to-zero metrics", () => {
    expect(emptyView.connection.peers).toEqual([]);
    expect(emptyView.connection.virtual_ip).toBeNull();
    expect(emptyView.connection.phase).toBe("disconnected");
    expect(emptyView.helper.connection_enabled).toBe(false);
  });
  it("keeps the test adapter and simulation hook out of production output", () => {
    const js = readdirSync("dist/assets")
      .filter((x) => x.endsWith(".js"))
      .map((x) => readFileSync(`dist/assets/${x}`, "utf8"))
      .join("");
    expect(js).not.toContain("__QUICKLAN_TEST_INVOKE");
    expect(js).not.toContain("TEST SIMULATION");
  });
  it("does not grant remote or arbitrary-command capabilities", () => {
    const c = JSON.parse(
      readFileSync("src-tauri/capabilities/main.json", "utf8"),
    );
    expect(c.remote).toBeUndefined();
    expect(c.windows).toEqual(["main"]);
    expect(
      c.permissions.some((p: string) => /shell:|fs:|http:|read-text/.test(p)),
    ).toBe(false);
  });
  it("does not use storage APIs to retain browser secrets", () => {
    const code = [
      "src/App.tsx",
      "src/lib/bridge.ts",
      "src/features/NetworkDialogs.tsx",
    ]
      .map((p) => readFileSync(p, "utf8"))
      .join("");
    expect(code).not.toMatch(/localStorage|sessionStorage|indexedDB/);
    expect(en.bearer).toContain("Anyone");
  });
});
