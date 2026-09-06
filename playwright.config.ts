import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "tests/ui",
  fullyParallel: false,
  workers: 1,
  timeout: 30000,
  use: {
    baseURL: "http://127.0.0.1:1421",
    viewport: { width: 1060, height: 740 },
    trace: "retain-on-failure",
  },
  webServer: {
    command: "npx vite --mode test --port 1421 --host 127.0.0.1",
    url: "http://127.0.0.1:1421",
    reuseExistingServer: !process.env.CI,
  },
  reporter: "list",
});
