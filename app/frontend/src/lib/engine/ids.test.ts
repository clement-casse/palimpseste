import { describe, expect, it } from "vitest";

import { amelia, ionas, lhagva } from "../../test/backend";
import { catalog, evaluate } from ".";
import { COC, COC_ATTACK_MODES, COC_TOXICOMANE, COF, cocAttack } from "./ids";

const ids = (...documents: Parameters<typeof evaluate>[0][]) =>
  new Set(documents.flatMap((document) => evaluate(document).values.map((line) => line.id)));

describe("the identifiers the interface names", () => {
  it("are values the COF2 engine emits", () => {
    // Ionas knows spells, so has PM.
    const emitted = ids(lhagva(), ionas());
    for (const id of Object.values(COF)) expect(emitted, id).toContain(id);
  });

  it("are values the COC2 engine emits", () => {
    const emitted = ids(amelia());
    // The age is the subject of a violation, not a value.
    const values = Object.values(COC).filter((id) => id !== COC.age);
    for (const id of values) expect(emitted, id).toContain(id);
    for (const mode of COC_ATTACK_MODES) expect(emitted, mode).toContain(cocAttack(mode));
  });

  it("are content the COC2 catalogue lists", () => {
    const content = catalog("contemporain");
    if (content.engine !== "contemporain") throw new Error("not the COC2 catalogue");
    expect(content.traits.map((option) => option.id)).toContain(COC_TOXICOMANE);
    expect(content.weapons.map((weapon) => weapon.attackMode.id)).toEqual(
      expect.arrayContaining(COC_ATTACK_MODES),
    );
  });
});
