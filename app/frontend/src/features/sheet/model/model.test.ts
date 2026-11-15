import { describe, expect, it } from "vitest";

import { type Document, catalog, evaluate, narrow, withEquipment } from "$lib/engine";
import { COC, COF } from "$lib/engine/ids";
import { amelia, ionas, lhagva } from "../../../test/backend";
import { equipmentOf } from "../equipment";
import { contemporainSheetModel } from "./contemporain";
import { fantasySheetModel } from "./fantasy";

/** The character of `document` in play with its creation equipment. */
function inPlay(document: Document) {
  const view = narrow(document, catalog(document.engine), evaluate(document));
  const played = view && withEquipment(view, equipmentOf(document));
  if (!played) throw new Error("the fixture cannot be read");
  return played;
}

function fantasy(document: Document) {
  const view = inPlay(document);
  if (view.engine !== "fantasy") throw new Error("not a COF2 character");
  return fantasySheetModel(view);
}

function contemporain(document: Document) {
  const view = inPlay(document);
  if (view.engine !== "contemporain") throw new Error("not a COC2 character");
  return contemporainSheetModel(view);
}

describe("the COF2 sheet model", () => {
  it("names the character", () => {
    expect(fantasy(lhagva()).identity).toEqual([
      { label: "Profil", value: "Barbare (Combattants)" },
      { label: "Peuple", value: "Humain" },
      { label: "Niveau", value: "1 (d4° : d4)" },
    ]);
  });

  it("counts PM only for a character who knows a spell", () => {
    const short = (document: Document) => fantasy(document).resources.map((r) => r.short);
    expect(short(lhagva())).toEqual(["PV", "PC", "DR"]);
    expect(short(ionas())).toEqual(["PV", "PM", "PC", "DR"]);
  });

  it("gives each resource its maximum, its name and its die", () => {
    expect(fantasy(lhagva()).resources.find((r) => r.id === COF.recoveryDice)).toEqual({
      id: COF.recoveryDice,
      short: "DR",
      label: "Dés de récupération (DR)",
      max: 4,
      dice: "d10",
    });
  });

  it("writes the combat values, a die beside the attacks only", () => {
    const combat = fantasy(lhagva()).combat;
    expect(combat.map((tile) => [tile.label, tile.text])).toEqual([
      ["Init", "14"],
      ["DEF", "16"],
      ["Contact", "+4"],
      ["Distance", "+2"],
      ["Magique", "+2"],
    ]);
    expect(combat.filter((tile) => tile.roll).map((tile) => tile.id)).toEqual([
      COF.attackContact,
      COF.attackRanged,
      COF.attackMagic,
    ]);
  });

  it("lists the weapons carried, each use with its attack and its DM", () => {
    const weapons = fantasy(lhagva()).weapons;
    expect(weapons.map((w) => [w.weapon.name, w.quantity])).toEqual([
      ["Épée longue", 1],
      ["Javelot", 2],
      ["Dague", 1],
    ]);
    const dague = weapons[2];
    expect(dague.uses.map((use) => use.name)).toEqual(["À une main", "À distance"]);
    for (const use of dague.uses) {
      expect(use.attack?.id).toBe(use.attackId);
      expect(use.damage.dice).toBe("1d4");
    }
    expect(dague.critical?.total).toBe(20);
  });

  it("keeps the weapons' own values out of the situational list", () => {
    const listed = fantasy(lhagva()).situational.map((line) => line.id);
    expect(listed).toContain(COF.defence);
    expect(listed.some((id) => id.startsWith("cof.damage."))).toBe(false);
  });

  it("names what is worn and held", () => {
    const model = fantasy(lhagva());
    expect(model.armour?.name).toBe("Cuir simple");
    expect(model.shield?.name).toBe("Grand bouclier");
    expect(model.money).toBe("7 pa");
    expect(fantasy(ionas()).armour).toBeUndefined();
  });

  it("lists the voies held with their ranks", () => {
    expect(fantasy(ionas()).voies).toContainEqual({ id: "cof.invocation", ranks: [1, 2] });
  });
});

describe("the COC2 sheet model", () => {
  it("names the character", () => {
    expect(contemporain(amelia()).identity).toEqual([
      { label: "Âge", value: "32 ans (31-50 ans)" },
      { label: "Domaine", value: "Aventure" },
      { label: "Classe sociale", value: "Classe moyenne" },
    ]);
  });

  it("gives the length of the échelle de santé and the XP left", () => {
    const model = contemporain(amelia());
    expect(model.health).toBe(20);
    expect(model.experience).toBe(0);
  });

  it("writes the combat values, then each attack value with its die", () => {
    const combat = contemporain(amelia()).combat;
    expect(combat.slice(0, 5).map((tile) => [tile.id, tile.label, tile.roll])).toEqual([
      [COC.initiative, "Init", undefined],
      [COC.defence, "DEF", undefined],
      [COC.damageReduction, "RD", undefined],
      [COC.healingCapacity, "CG / sem.", undefined],
      [COC.contactDamageBonus, "DM contact", undefined],
    ]);
    const attacks = combat.slice(5);
    expect(attacks).toHaveLength(4);
    expect(attacks.every((tile) => tile.roll)).toBe(true);
    expect(attacks[0]).toMatchObject({ label: "ATC mains nues", text: "+1" });
  });

  it("lists bare hands, then the weapons carried", () => {
    const weapons = contemporain(amelia()).weapons;
    expect(weapons.map((w) => w.weapon.name)).toEqual([
      "Mains nues",
      "Couteau, poignard, baïonnette",
    ]);
    expect(weapons[1]).toMatchObject({
      quantity: 1,
      attack: { id: "coc.attack.contact-weapon" },
      damage: { dice: "1d6" },
      bonus: { total: 6 },
      range: { total: 20 },
    });
  });

  it("signs the traits: a disadvantage counts against", () => {
    const { advantages, disadvantages } = contemporain(amelia());
    expect(advantages.total).toBe(5);
    expect(advantages.traits).toContainEqual({
      id: "coc.trait.force-de-la-nature",
      name: "Force de la nature",
      points: 2,
    });
    expect(disadvantages.total).toBe(-5);
    expect(disadvantages.traits.every((taken) => taken.points < 0)).toBe(true);
  });

  it("names the sequels acquired and the protections worn", () => {
    const document = amelia();
    if (document.engine !== "contemporain") throw new Error("not a COC2 character");
    const content = catalog("contemporain");
    if (content.engine !== "contemporain") throw new Error("not the COC2 catalogue");
    document.acquiredSequels = [content.sequels[0].id];
    document.protections = [content.protections[0].id];
    const model = contemporain(document);
    expect(model.sequels).toEqual([content.sequels[0]]);
    expect(model.protections).toEqual([content.protections[0].name]);
  });
});
