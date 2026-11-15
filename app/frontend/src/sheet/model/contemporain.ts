// A COC2 character at the table, as plain data (ADR FE-SHEET): what the
// sheet shows, shaped from what the engine computed. Every number comes
// from the engine; nothing here applies a rule.

import {
  type ContemporainCatalog,
  type ContemporainDetails,
  type ContemporainDocument,
  type ContemporainEquipment,
  type ContemporainWeaponEntry,
  type Sheet,
  type ValueLine,
  type VoieEntry,
} from "../../lib/engine";
import {
  COC,
  COC_ATTACK_MODES,
  cocAttack,
  cocCriticalBonus,
  cocCriticalRange,
  cocDamage,
  cocWeapon,
  cocWeaponSlugs,
  isCocWeaponValue,
  valueLine,
} from "../../lib/engine/ids";
import { currentVoies } from "../../progression/progression";
import { type Field, type Tile, carried, tile } from "./common";

/** A weapon, rolled against the attack value of its category. */
export type Weapon = {
  weapon: ContemporainWeaponEntry;
  quantity: number;
  attack: ValueLine | undefined;
  damage: ValueLine | undefined;
  /** The bonus critique. */
  bonus: ValueLine | undefined;
  /** The critical range. */
  range: ValueLine | undefined;
};

/** A trait taken, its points signed: negative for a disadvantage. */
export type TakenTrait = { id: string; name: string; points: number };

export type ContemporainSheetModel = {
  identity: Field[];
  /** The length of the échelle de santé, while it can be computed. */
  health: number | undefined;
  /** The combat values, the AGI test modifier when it is not nil, then each attack value. */
  combat: Tile[];
  /** The values whose situational modifiers are listed under the combat tiles: weapons show theirs in their row. */
  situational: ValueLine[];
  /** Bare hands, then the weapons carried. */
  weapons: Weapon[];
  /** The voies held now, with their ranks. */
  voies: VoieEntry[];
  /** The XP left to spend. */
  experience: number | undefined;
  advantages: { total: number; traits: TakenTrait[] };
  disadvantages: { total: number; traits: TakenTrait[] };
  /** The names of the sequels acquired in play. */
  sequels: { id: string; name: string }[];
  /** The names of the protections worn. */
  protections: string[];
};

const COMBAT = [
  { id: COC.initiative, label: "Init", title: "Initiative", signed: false },
  { id: COC.defence, label: "DEF", title: "Défense", signed: false },
  { id: COC.damageReduction, label: "RD", title: "Réduction des dommages", signed: false },
  {
    id: COC.healingCapacity,
    label: "CG / sem.",
    title: "Capacité de guérison : DM soignés par semaine",
    signed: false,
  },
  { id: COC.contactDamageBonus, label: "DM contact", title: "Bonus aux DM au contact", signed: true },
];

/** What the sheet of a COC2 character shows, with `equipment` in play. */
export function contemporainSheetModel({
  choices,
  catalog,
  sheet,
  details,
  equipment,
}: {
  choices: ContemporainDocument;
  catalog: ContemporainCatalog;
  sheet: Sheet;
  details: ContemporainDetails;
  equipment: ContemporainEquipment;
}): ContemporainSheetModel {
  const values = sheet.values;
  const domaine = catalog.domaines.find((d) => d.id === choices.domaine);
  const bracket = details.ageBracket ? ` (${details.ageBracket})` : "";
  const agi = valueLine(values, COC.agiTestModifier);
  const traits = (choices.traits ?? []).flatMap((taken) => {
    const option = catalog.traits.find((t) => t.id === taken.id);
    return option ? [{ ...option, points: option.advantage ? taken.points : -taken.points }] : [];
  });
  const taken = (advantage: boolean): TakenTrait[] =>
    traits
      .filter((t) => t.advantage === advantage)
      .map(({ id, name, points }) => ({ id, name, points }));

  return {
    identity: [
      { label: "Âge", value: choices.age === undefined ? "À choisir" : `${choices.age} ans${bracket}` },
      { label: "Domaine", value: domaine?.name ?? "À choisir" },
      { label: "Classe sociale", value: details.socialClass ?? "contradictoire" },
    ],
    health: valueLine(values, COC.healthScale)?.total,
    combat: [
      // The values nobody rolls: no die beside them.
      ...COMBAT.map(({ id, ...text }) => ({ ...tile(id, valueLine(values, id), text), roll: undefined })),
      ...(agi?.total
        ? [
            {
              ...tile(agi.id, agi, { label: "Tests d'AGI", signed: true }),
              situational: undefined,
              roll: undefined,
            },
          ]
        : []),
      ...COC_ATTACK_MODES.flatMap((mode) => {
        const attack = valueLine(values, cocAttack(mode));
        return attack ? [tile(attack.id, attack, { label: attack.label, signed: true })] : [];
      }),
    ],
    situational: values.filter((line) => !isCocWeaponValue(line.id)),
    // From the engine's damage lines, each rolled against its attack value.
    weapons: cocWeaponSlugs(values).flatMap((slug) => {
      const weapon = catalog.weapons.find((w) => w.id === cocWeapon(slug));
      if (!weapon) return [];
      return [
        {
          weapon,
          quantity: carried(equipment.items, weapon.id),
          attack: valueLine(values, cocAttack(weapon.attackMode.id)),
          damage: valueLine(values, cocDamage(slug)),
          bonus: valueLine(values, cocCriticalBonus(slug)),
          range: valueLine(values, cocCriticalRange(slug)),
        },
      ];
    }),
    voies: currentVoies(catalog, details.abilitiesHeld),
    experience: valueLine(values, COC.experience)?.total,
    advantages: { total: details.advantages, traits: taken(true) },
    disadvantages: { total: details.disadvantages, traits: taken(false) },
    sequels: (choices.acquiredSequels ?? []).map((id) => ({
      id,
      name: catalog.sequels.find((s) => s.id === id)?.name ?? id,
    })),
    protections: (equipment.protections ?? []).map(
      (id) => catalog.protections.find((p) => p.id === id)?.name ?? id,
    ),
  };
}
