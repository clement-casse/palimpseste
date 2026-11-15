import { afterEach, describe, expect, it } from "vitest";

import type { RollOutcome } from "../wasm/palimpseste_wasm";
import { KEPT, RollLog } from "./rolls.svelte";

const outcome = (total: number): RollOutcome => ({
  roll: "cof.roll.test.for",
  name: "Test de FOR",
  kind: "test",
  mode: "plain",
  dice: [{ faces: 20, face: total, kept: true }],
  natural: undefined,
  lines: [{ kind: "dice", label: "1d20", origin: undefined, amount: total, dice: [] }],
  total,
  critical: false,
  criticalFrom: undefined,
  unpicked: [],
  setAside: [],
  damageRoll: undefined,
});

afterEach(() => localStorage.clear());

describe("the journal of rolls", () => {
  it("keeps the rolls of each character on the device, the latest first", () => {
    const log = new RollLog(1);
    log.add(outcome(7));
    log.add(outcome(12));
    expect(new RollLog(1).entries.map((entry) => entry.outcome.total)).toEqual([12, 7]);
    expect(new RollLog(2).entries).toEqual([]);
  });

  it("keeps the rolls another sheet of the character added since", () => {
    const here = new RollLog(1);
    const there = new RollLog(1);
    here.add(outcome(7));
    there.add(outcome(12));
    expect(there.entries.map((entry) => entry.outcome.total)).toEqual([12, 7]);
    expect(new RollLog(1).entries.map((entry) => entry.outcome.total)).toEqual([12, 7]);
  });

  it("drops the oldest rolls beyond what it keeps", () => {
    const log = new RollLog(1);
    for (let total = 1; total <= KEPT + 2; total += 1) log.add(outcome(total));
    expect(log.entries).toHaveLength(KEPT);
    expect(log.entries[0].outcome.total).toBe(KEPT + 2);
    expect(log.entries.at(-1)?.outcome.total).toBe(3);
  });

  it("forgets every roll when cleared", () => {
    const log = new RollLog(1);
    log.add(outcome(7));
    log.clear();
    expect(log.entries).toEqual([]);
    expect(new RollLog(1).entries).toEqual([]);
  });

  it("drops what it cannot read", () => {
    localStorage.setItem("palimpseste.rolls.1", "not json");
    expect(new RollLog(1).entries).toEqual([]);
    localStorage.setItem(
      "palimpseste.rolls.1",
      JSON.stringify([{ at: 1, outcome: outcome(9) }, { at: "soon" }, 4, { at: 2, outcome: { name: "?" } }]),
    );
    expect(new RollLog(1).entries.map((entry) => entry.outcome.total)).toEqual([9]);
  });
});
