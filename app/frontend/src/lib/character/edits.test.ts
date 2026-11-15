import { describe, expect, it } from "vitest";

import { U32_MAX, integerOf } from "./edits";

/** A change event of a number input holding `typed`. */
function typed(typed: string): Event {
  const input = document.createElement("input");
  input.type = "text";
  input.value = typed;
  const event = new Event("change");
  input.addEventListener("change", () => {});
  Object.defineProperty(event, "currentTarget", { value: input });
  return event;
}

describe("integerOf", () => {
  it("reads whole numbers within bounds", () => {
    expect(integerOf(typed(" 3 "), 2)).toBe(3);
    expect(integerOf(typed("-7"), 2)).toBe(-7);
    expect(integerOf(typed(String(U32_MAX)), 2, { max: U32_MAX })).toBe(U32_MAX);
  });

  it("reads a blank as undefined, unless it is refused", () => {
    expect(integerOf(typed(""), 2)).toBeUndefined();
    expect(integerOf(typed(""), 2, { blank: false })).toBeNull();
  });

  it("refuses anything else and puts the input back", () => {
    for (const text of ["-1", "5000000000", "1.5", "abc"]) {
      const event = typed(text);
      expect(integerOf(event, 2, { min: 0, max: U32_MAX }), text).toBeNull();
      expect((event.currentTarget as HTMLInputElement).value, text).toBe("2");
    }
  });
});
