import { afterEach, describe, expect, it } from "vitest";

import { amelia, lhagva } from "../test/backend";
import { catalog, emptyDocument, evaluate } from "../wasm/palimpseste_wasm";
import { clearDraft, loadDraft, saveDraft } from "./drafts";
import {
  abilityLabel,
  bonusDieLabel,
  currentVoies,
  differences,
  indexOf,
  nextLabel,
  nextNumber,
  progressionCount,
  scopeLabel,
  truncated,
  withNext,
  without,
} from "./progression";

afterEach(() => localStorage.clear());

describe("the scope of a bonus", () => {
  it("names the speciality first when the bonus applies to it", () => {
    const bonus = { scope: "discipline", onSpeciality: true };
    expect(scopeLabel(bonus, "Peinture")).toBe("Peinture — discipline");
    expect(scopeLabel(bonus, undefined)).toBe("discipline");
    expect(scopeLabel({ scope: "esquiver", onSpeciality: false }, "Peinture")).toBe("esquiver");
  });
});

describe("the tests a dé bonus goes to", () => {
  const abilities = catalog("contemporain");
  const die = { scope: "véhicule choisi", onSpeciality: false, characteristics: [] };

  it("names the capacité chosen, its tests first", () => {
    const line = { ...die, onChosenAbility: true, target: "coc.vehicules.2", targetScope: undefined };
    expect(bonusDieLabel(abilities, line, undefined)).toBe(abilityLabel(abilities, "coc.vehicules.2"));
    const scoped = { ...line, targetScope: "moto — conduite du véhicule" };
    expect(bonusDieLabel(abilities, scoped, undefined)).toBe("moto — conduite du véhicule");
    const none = { ...line, target: undefined };
    expect(bonusDieLabel(abilities, none, undefined)).toBe("test à choisir");
  });

  it("reads as a scope for a dé bonus asking no choice", () => {
    const line = { ...die, onChosenAbility: false, target: undefined, targetScope: undefined };
    expect(bonusDieLabel(abilities, line, "moto")).toBe("véhicule choisi");
  });
});

describe("the progression of a document", () => {
  it("numbers COF levels from 2 and COC spendings from 1", () => {
    const cof = lhagva();
    expect(progressionCount(cof)).toBe(0);
    expect(nextNumber(cof)).toBe(2);
    expect(nextLabel(cof)).toBe("Monter au niveau 2");
    const levelled = withNext(cof);
    expect(progressionCount(levelled)).toBe(1);
    expect(nextNumber(levelled)).toBe(3);
    expect(indexOf(levelled, 2)).toBe(0);
    // The copy leaves the document as it was.
    expect(progressionCount(cof)).toBe(0);

    const coc = amelia();
    expect(nextNumber(coc)).toBe(1);
    expect(nextLabel(coc)).toBe("Dépenser de l’XP");
    expect(indexOf(withNext(coc), 1)).toBe(0);
  });

  it("keeps or drops entries", () => {
    const two = withNext(withNext(lhagva()));
    expect(progressionCount(truncated(two, 1))).toBe(1);
    expect(progressionCount(without(two, 0))).toBe(1);
    expect(progressionCount(truncated(emptyDocument("contemporain"), 0))).toBe(0);
  });

  it("names a capacité by its identifier", () => {
    const content = catalog("fantasy");
    expect(abilityLabel(content, "cof.rage.2")).toBe("Défier la mort (Voie de la rage, rang 2)");
    expect(abilityLabel(content, "cof.unknown.1")).toBe("cof.unknown.1");
  });

  it("gives the voies held now and what a change brings", () => {
    const document = lhagva();
    if (document.engine !== "fantasy") throw new Error("fantasy expected");
    const before = evaluate(document);
    document.levelUps = [{ abilities: ["cof.pourfendeur.2", "cof.rage.2"], forgotten: [] }];
    const after = evaluate(document);
    if (after.details.engine !== "fantasy") throw new Error("fantasy expected");
    const voies = currentVoies(catalog("fantasy"), after.details.abilitiesHeld);
    expect(voies.find((v) => v.id === "cof.rage")?.ranks).toEqual([1, 2]);
    // PV 12 to 19 at level 2 (COF.PRG.PV).
    expect(differences(before, after)).toContainEqual({
      label: "Points de vigueur (PV)",
      before: 12,
      after: 19,
    });
  });
});

describe("drafts", () => {
  it("are kept on the device until cleared", () => {
    expect(loadDraft(7)).toBeNull();
    const draft = { number: 2, fresh: true, base: 3, document: withNext(lhagva()) };
    saveDraft(7, draft);
    expect(loadDraft(7)).toEqual(draft);
    clearDraft(7);
    expect(loadDraft(7)).toBeNull();
  });
});
