import { describe, it, expect } from "vitest";
import { errorText } from "../src/lib/i18n";
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
});
