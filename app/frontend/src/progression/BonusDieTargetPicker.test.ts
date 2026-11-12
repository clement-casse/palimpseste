import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";

import {
  type ContemporainCatalog,
  type ContemporainDocument,
  catalog,
  emptyDocument,
  evaluate,
} from "../wasm/palimpseste_wasm";
import BonusDieTargetPicker from "./BonusDieTargetPicker.svelte";

describe("the capacité a dé bonus goes to", () => {
  it("offers the ranks of the voie, the others out of the rules, and records the choice", async () => {
    const choices = emptyDocument("contemporain") as ContemporainDocument & { engine: "contemporain" };
    choices.voies = [{ id: "coc.crime", ranks: [1, 2, 3, 4, 5] }];
    const { details } = evaluate(choices);
    if (details.engine !== "contemporain") throw new Error("contemporain details expected");
    const line = details.rankFive.find((l) => l.voie === "coc.crime");
    expect(line?.bonusDieChoice).toEqual(["coc.crime.1", "coc.crime.3", "coc.crime.4"]);
    render(BonusDieTargetPicker, {
      choices,
      catalog: catalog("contemporain") as ContemporainCatalog,
      line: line!,
    });

    const select = screen.getByRole("combobox");
    const outside = screen.getByRole("group", { name: "Hors règles" });
    expect(outside.querySelectorAll("option")).toHaveLength(2);
    await fireEvent.change(select, { target: { value: "coc.crime.3" } });
    expect(choices.bonusDieTargets).toEqual([{ voie: "coc.crime", target: "coc.crime.3" }]);
    await fireEvent.change(select, { target: { value: "" } });
    expect(choices.bonusDieTargets).toEqual([]);
  });
});
