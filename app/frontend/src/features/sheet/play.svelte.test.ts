import { afterEach, describe, expect, it, vi } from "vitest";

import { PlayState } from "./play.svelte";

const PV = "cof.vigour-points";

afterEach(() => {
  localStorage.clear();
  vi.restoreAllMocks();
});

describe("the play state", () => {
  it("keeps what each character used on the device", () => {
    new PlayState(1, "fantasy").set(PV, 3, 10);
    expect(new PlayState(1, "fantasy").used(PV, 10)).toBe(3);
    expect(new PlayState(2, "fantasy").used(PV, 10)).toBe(0);
  });

  it("stays within nothing used and the maximum", () => {
    const play = new PlayState(1, "fantasy");
    play.set(PV, 15, 10);
    expect(play.used(PV, 10)).toBe(10);
    play.set(PV, -2, 10);
    expect(play.used(PV, 10)).toBe(0);
    play.set(PV, 8, 10);
    // A maximum lowered in the editor since.
    expect(play.used(PV, 6)).toBe(6);
  });

  it("puts everything back", () => {
    const play = new PlayState(1, "fantasy");
    play.set(PV, 3, 10);
    play.reset();
    expect(play.used(PV, 10)).toBe(0);
    expect(localStorage.getItem("palimpseste.play.1")).toBeNull();
  });

  it("drops what it cannot read", () => {
    localStorage.setItem("palimpseste.play.1", "not json");
    expect(new PlayState(1, "fantasy").used(PV, 10)).toBe(0);
    localStorage.setItem("palimpseste.play.1", JSON.stringify({ [PV]: "x", "cof.luck-points": 2 }));
    const play = new PlayState(1, "fantasy");
    expect(play.used(PV, 10)).toBe(0);
    expect(play.used("cof.luck-points", 3)).toBe(2);
  });

  it("keeps the equipment in play of each character, until forgotten", () => {
    const equipment = {
      engine: "fantasy" as const,
      armour: "cof.armour.cuir-renforce",
      items: [{ kind: "other" as const, name: "Torche", quantity: 3 }],
    };
    new PlayState(1, "fantasy").equip(equipment);
    expect(new PlayState(1, "fantasy").equipment).toEqual(equipment);
    expect(new PlayState(2, "fantasy").equipment).toBeNull();

    const play = new PlayState(1, "fantasy");
    play.set(PV, 2, 10);
    play.reset();
    expect(play.equipment).toEqual(equipment);
    play.forgetEquipment();
    expect(new PlayState(1, "fantasy").equipment).toBeNull();
  });

  it("drops an equipment of another engine or of another shape", () => {
    localStorage.setItem(
      "palimpseste.play.1.equipment",
      JSON.stringify({ engine: "contemporain", items: [] }),
    );
    expect(new PlayState(1, "fantasy").equipment).toBeNull();
    localStorage.setItem("palimpseste.play.1.equipment", JSON.stringify({ engine: "fantasy" }));
    expect(new PlayState(1, "fantasy").equipment).toBeNull();
    localStorage.setItem("palimpseste.play.1.equipment", "{");
    expect(new PlayState(1, "fantasy").equipment).toBeNull();
  });

  it("counts for the visit when storage is refused", () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("denied");
    });
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("denied");
    });
    const play = new PlayState(1, "fantasy");
    play.set(PV, 2, 10);
    expect(play.used(PV, 10)).toBe(2);
  });
});
