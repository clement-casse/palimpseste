import { describe, expect, it } from "vitest";

import type { VoieEntry } from "../wasm/palimpseste_wasm";
import { setRanks } from "./edits";

describe("ranks set on a document without voies", () => {
  it("keeps the first voie", () => {
    const document = $state<{ voies?: VoieEntry[] }>({});
    setRanks(document, "cof.voie.du-guerrier", [1]);
    expect(document.voies).toEqual([{ id: "cof.voie.du-guerrier", ranks: [1] }]);
  });
});
