// End-to-end tests: the built PWA served by the real backend, on a
// throwaway database, signing in through a mock OIDC provider (ADR
// FE-TESTS). `mise run frontend:e2e` builds the PWA first.
import { defineConfig, devices } from "@playwright/test";

const BACKEND = "http://127.0.0.1:3100";
const PROVIDER = "http://127.0.0.1:9400";
const DATABASE = "target/e2e.sqlite3";

export default defineConfig({
  testDir: "e2e",
  fullyParallel: false,
  workers: 1,
  use: { baseURL: BACKEND, trace: "retain-on-failure" },
  projects: [
    { name: "sign-in", testMatch: /auth\.setup\.ts/ },
    {
      name: "chromium",
      testMatch: /.*\.e2e\.ts/,
      use: { ...devices["Desktop Chrome"], storageState: "e2e/.auth/user.json" },
      dependencies: ["sign-in"],
    },
  ],
  webServer: [
    {
      command: "node e2e/oidc-provider.mjs",
      url: `${PROVIDER}/.well-known/openid-configuration`,
    },
    {
      // From the repository root, on a database created for this run. Flags,
      // not variables: they win over a developer's `mise.local.toml`, which
      // mise applies to `cargo` again.
      command: [
        `rm -f ${DATABASE} && cargo run -p palimpseste-backend --`,
        "--addr 127.0.0.1:3100",
        `--public-url ${BACKEND}`,
        `--database ${DATABASE}`,
        "--frontend-dir app/frontend/dist",
        "--oidc-providers mock",
      ].join(" "),
      cwd: "../..",
      url: `${BACKEND}/healthz`,
      timeout: 300_000,
      env: {
        PALIMPSESTE_OIDC_MOCK_NAME: "Test",
        PALIMPSESTE_OIDC_MOCK_ISSUER: PROVIDER,
        PALIMPSESTE_OIDC_MOCK_CLIENT_ID: "palimpseste",
        PALIMPSESTE_OIDC_MOCK_CLIENT_SECRET: "e2e",
      },
    },
  ],
});
