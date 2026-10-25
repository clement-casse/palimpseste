// Signs in once through the mock provider, creates the account at its
// first sign-in, and keeps the session for every test.
import { expect, test as setup } from "@playwright/test";

setup("signs in through the mock provider and creates the account", async ({ page }) => {
  await page.goto("/login");
  await page.getByRole("link", { name: "Se connecter avec Test" }).click();
  await expect(page.getByRole("heading", { name: "Créer un compte" })).toBeVisible();
  await page.getByLabel("Pseudonyme").fill("Testeur");
  await page.getByRole("button", { name: "Créer le compte" }).click();
  await page.waitForURL((url) => url.pathname === "/");
  await page.context().storageState({ path: "e2e/.auth/user.json" });
});
