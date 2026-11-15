// Lhagva, the book's worked COF2 character, created through the API and
// read in the browser: the capacités by name with what they give, and the
// breakdown of a characteristic on hover.
import { readFileSync } from "node:fs";

import { type APIRequestContext, expect, test } from "@playwright/test";

const lhagva: unknown = JSON.parse(
  readFileSync(new URL("../../document/tests/fixtures/lhagva.json", import.meta.url), "utf8"),
);

/** Creates Lhagva for the signed-in user; returns the character's id. */
async function createLhagva(request: APIRequestContext): Promise<number> {
  const created = await request.post("/api/me/characters", {
    data: { name: "Lhagva", engine: "fantasy" },
  });
  expect(created.status()).toBe(201);
  const { id } = (await created.json()) as { id: number };
  const read = await request.get(`/api/characters/${id}`);
  const saved = await request.put(`/api/characters/${id}/document`, {
    data: lhagva,
    headers: { "If-Match": read.headers()["etag"] ?? "" },
  });
  expect(saved.ok()).toBeTruthy();
  return id;
}

let id: number;

test.beforeAll(async ({ request }) => {
  id = await createLhagva(request);
});

test("the sheet names each capacité with what it gives", async ({ page }) => {
  await page.goto(`/characters/${id}/sheet`);
  await expect(page.getByText("1. Réflexes éclair")).toBeVisible();
  // Réflexes éclair: rank + 2 to AGI dodge tests.
  await expect(page.getByText(/\+3\s*esquiver\s*\(AGI\)/).first()).toBeVisible();
  await expect(page.getByRole("columnheader", { name: "Attaque" })).toBeVisible();
});

test("hovering a characteristic on the sheet shows its sources", async ({ page }) => {
  await page.goto(`/characters/${id}/sheet`);
  await page.getByRole("button", { name: /^PER .*, détail$/ }).hover();
  const popover = page.getByRole("tooltip");
  await expect(popover).toBeVisible();
  // The humain bonus Lhagva put in PER.
  await expect(popover).toContainText("Humain +1");
});

test("hovering a characteristic in the editor shows its sources", async ({ page }) => {
  await page.goto(`/characters/${id}#characteristics`);
  await page.locator("#editor-characteristic-PER").hover();
  await expect(page.getByRole("tooltip")).toContainText("Humain +1");
});

test("a test rolled from the sheet shows its calculation and stays in the journal", async ({
  page,
}) => {
  await page.goto(`/characters/${id}/sheet`);
  await page.getByRole("button", { name: "Lancer : Test de AGI" }).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog.getByRole("heading", { name: "Test de AGI" })).toBeVisible();
  // Réflexes éclair's bonus holds for dodging only: offered, not picked.
  const dodge = dialog.getByRole("checkbox", { name: /\+3 esquiver/ });
  await expect(dodge).not.toBeChecked();
  // The help waits in the folded section.
  await dialog.getByRole("button", { name: /^Autres bonus et modificateurs/ }).click();
  await dialog.getByRole("button", { name: "Un allié de plus" }).click();
  await dialog.getByRole("button", { name: "Lancer le dé" }).click();

  // The die is drawn in the browser: any face, then the lines add up.
  await expect(dialog.getByText(/d20 : \d+/).first()).toBeVisible();
  const lines = dialog.getByRole("list", { name: "Calcul" });
  await expect(lines).toContainText("Coopération : 1 allié");
  const amounts = await lines.locator("li > span:last-child").allTextContents();
  const sum = amounts.reduce((total, amount) => total + Number(amount), 0);
  await expect(dialog.getByRole("status", { name: "Total" })).toHaveText(String(sum));
  await expect(dialog.getByText("Peut aussi s’appliquer, à annoncer au MJ")).toBeVisible();

  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();
  const journal = page.locator("section", {
    has: page.getByRole("heading", { name: "Journal des jets" }),
  });
  await expect(journal.getByText("Test de AGI")).toBeVisible();
});

test("an attack rolled from the sheet leads to its DM", async ({ page }) => {
  await page.goto(`/characters/${id}/sheet`);
  await page
    .getByRole("button", { name: /^Lancer : attaque, Épée longue/ })
    .first()
    .click();
  const dialog = page.getByRole("dialog");
  await dialog.getByRole("button", { name: "Lancer le dé" }).click();
  await dialog
    .getByRole("button", { name: /^Lancer les DM/ })
    .last()
    .click();
  await expect(dialog.getByRole("heading", { name: /^DM : Épée longue/ })).toBeVisible();
  // The attack's dialog fades out behind the DM's: the last is the DM's own.
  await dialog.last().getByRole("button", { name: "Lancer les DM", exact: true }).click();
  await expect(dialog.last().getByRole("status", { name: "Total" })).toBeVisible();
});
