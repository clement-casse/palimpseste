import { fireEvent, render, screen } from "@testing-library/svelte";
import { expect, test } from "vitest";

import CharacteristicSheet from "../character/CharacteristicSheet.svelte";
import type { CharacteristicLine } from "../wasm/palimpseste_wasm";
import Characteristics from "./Characteristics.svelte";

// The popover opens on a debounced timer, after the page is laid out: give it
// time on a busy CI runner.
const OPENING = { timeout: 5000 };

const constitution: CharacteristicLine = {
  characteristic: "CON",
  total: 2,
  contributions: [
    { source: "value", sourceId: "cof.series-value", label: "Valeur de série", amount: 1 },
    {
      source: "ability",
      sourceId: "cof.pagne.4",
      label: "Constitution héroïque (Voie du pagne, rang 4)",
      amount: 1,
    },
  ],
  situational: [],
};

test("hovering a characteristic on the sheet shows every source, capacités included", async () => {
  render(Characteristics, { characteristics: [constitution], notes: [] });
  await fireEvent.mouseEnter(screen.getByRole("button", { name: "CON +2, détail" }));
  expect(
    await screen.findByText("Constitution héroïque (Voie du pagne, rang 4)", {}, OPENING),
  ).toBeTruthy();
  expect(screen.getByText("Valeur de série")).toBeTruthy();
});

test("hovering a characteristic in the editor shows its sources while the item is closed", async () => {
  render(CharacteristicSheet, { characteristics: [constitution], onadjust: () => {} });
  const total = document.getElementById("editor-characteristic-CON");
  if (!total) throw new Error("no total for CON");
  await fireEvent.mouseEnter(total);
  expect(await screen.findByText("CON : +2", {}, OPENING)).toBeTruthy();
  expect(screen.getByText("Constitution héroïque (Voie du pagne, rang 4)")).toBeTruthy();
});
