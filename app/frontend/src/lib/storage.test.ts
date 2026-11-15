import { afterEach, describe, expect, it, vi } from "vitest";

import { KEYS, readJson, readText, remove, writeJson, writeText } from "./storage";

afterEach(() => {
  localStorage.clear();
  sessionStorage.clear();
  vi.restoreAllMocks();
});

describe("the device storage", () => {
  it("keeps text and JSON, and forgets them", () => {
    writeText(KEYS.theme, "dark");
    writeJson(KEYS.play(1), { pv: 3 });
    expect(readText(KEYS.theme)).toBe("dark");
    expect(readJson(KEYS.play(1))).toEqual({ pv: 3 });
    remove(KEYS.theme);
    expect(readText(KEYS.theme)).toBeNull();
  });

  it("finds nothing where nothing or no JSON is kept", () => {
    expect(readJson(KEYS.rolls(1))).toBeUndefined();
    localStorage.setItem(KEYS.rolls(1), "not json");
    expect(readJson(KEYS.rolls(1))).toBeUndefined();
  });

  it("keeps the tab's own apart from the device's", () => {
    writeText(KEYS.return, "/friends", "session");
    expect(readText(KEYS.return)).toBeNull();
    expect(readText(KEYS.return, "session")).toBe("/friends");
  });

  it("goes on when storage is refused", () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("refused");
    });
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("refused");
    });
    vi.spyOn(Storage.prototype, "removeItem").mockImplementation(() => {
      throw new Error("refused");
    });
    expect(() => writeJson(KEYS.draft(1), {})).not.toThrow();
    expect(() => remove(KEYS.draft(1))).not.toThrow();
    expect(readText(KEYS.draft(1))).toBeNull();
    expect(readJson(KEYS.draft(1))).toBeUndefined();
  });
});
