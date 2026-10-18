import { afterEach, describe, expect, it, vi } from "vitest";

import { PlayState } from "./play.svelte";

const PV = "cof.vigour-points";

afterEach(() => {
  localStorage.clear();
  vi.restoreAllMocks();
});

describe("the play state", () => {
  it("keeps what each character used on the device", () => {
    new PlayState(1).set(PV, 3, 10);
    expect(new PlayState(1).used(PV, 10)).toBe(3);
    expect(new PlayState(2).used(PV, 10)).toBe(0);
  });

  it("stays within nothing used and the maximum", () => {
    const play = new PlayState(1);
    play.set(PV, 15, 10);
    expect(play.used(PV, 10)).toBe(10);
    play.set(PV, -2, 10);
    expect(play.used(PV, 10)).toBe(0);
    play.set(PV, 8, 10);
    // A maximum lowered in the editor since.
    expect(play.used(PV, 6)).toBe(6);
  });

  it("puts everything back", () => {
    const play = new PlayState(1);
    play.set(PV, 3, 10);
    play.reset();
    expect(play.used(PV, 10)).toBe(0);
    expect(localStorage.getItem("palimpseste.play.1")).toBeNull();
  });

  it("drops what it cannot read", () => {
    localStorage.setItem("palimpseste.play.1", "not json");
    expect(new PlayState(1).used(PV, 10)).toBe(0);
    localStorage.setItem("palimpseste.play.1", JSON.stringify({ [PV]: "x", "cof.luck-points": 2 }));
    const play = new PlayState(1);
    expect(play.used(PV, 10)).toBe(0);
    expect(play.used("cof.luck-points", 3)).toBe(2);
  });

  it("counts for the visit when storage is refused", () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("denied");
    });
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("denied");
    });
    const play = new PlayState(1);
    play.set(PV, 2, 10);
    expect(play.used(PV, 10)).toBe(2);
  });
});
