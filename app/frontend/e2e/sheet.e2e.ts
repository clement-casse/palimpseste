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
