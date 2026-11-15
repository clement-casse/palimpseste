// A COF2 character at the table, as plain data (ADR FE-SHEET): what the
// sheet shows, shaped from what the engine computed. Every number comes
// from the engine; nothing here applies a rule.

import {
  type FantasyCatalog,
  type FantasyDetails,
  type FantasyDocument,
  type FantasyEquipment,
  type FantasyWeaponEntry,
  type ProtectionEntry,
  type Sheet,
  type ValueLine,
  type VoieEntry,
} from "../../lib/engine";
import {
  COF,
  cofAttackOf,
  cofCriticalRange,
  cofDamagePrefix,
  cofWeapon,
  cofWeaponSlugs,
  isCofWeaponValue,
  valueLine,
} from "../../lib/engine/ids";
import { currentVoies } from "../../progression/progression";
import { type Field, type Tile, carried, tile } from "./common";

/** A resource counted against its computed maximum. */
export type Resource = {
  /** The identifier of the value: what is used is kept by it. */
  id: string;
  /** The short name, such as PV. */
  short: string;
  /** The full name. */
  label: string;
  max: number;
  /** The die of the resource (DR). */
  dice: string | undefined;
};

/** A weapon used one way: its attack and its DM. */
export type WeaponUse = {
  name: string;
  /** The identifier of the attack value. */
  attackId: string;
  attack: ValueLine | undefined;
  damage: ValueLine;
};

/** A weapon carried, with each of its uses. */
export type Weapon = {
  weapon: FantasyWeaponEntry;
  quantity: number;
  uses: WeaponUse[];
  critical: ValueLine | undefined;
};

export type FantasySheetModel = {
  identity: Field[];
  /** PV, PM, PC and DR; PM only for a character who knows a spell (COF.DER.PM). */
  resources: Resource[];
  combat: Tile[];
  /** The values whose situational modifiers are listed under the combat tiles: weapons show theirs in their row. */
  situational: ValueLine[];
  /** The weapons carried, in the document's order. */
  weapons: Weapon[];
  /** The voies held now, with their ranks. */
  voies: VoieEntry[];
  armour: ProtectionEntry | undefined;
  shield: ProtectionEntry | undefined;
  /** The purse, as the engine writes it. */
  money: string | undefined;
};

const RESOURCES = [
  { id: COF.vigourPoints, short: "PV" },
  { id: COF.manaPoints, short: "PM" },
  { id: COF.luckPoints, short: "PC" },
  { id: COF.recoveryDice, short: "DR" },
];

const COMBAT = [
  { id: COF.initiative, label: "Init", title: "Initiative", signed: false },
  { id: COF.defence, label: "DEF", title: "Défense", signed: false },
  { id: COF.attackContact, label: "Contact", title: "Attaque au contact", signed: true },
  { id: COF.attackRanged, label: "Distance", title: "Attaque à distance", signed: true },
  { id: COF.attackMagic, label: "Magique", title: "Attaque magique", signed: true },
];

/** What the sheet of a COF2 character shows, with `equipment` in play. */
export function fantasySheetModel({
  choices,
  catalog,
  sheet,
  details,
  equipment,
}: {
  choices: FantasyDocument;
  catalog: FantasyCatalog;
  sheet: Sheet;
  details: FantasyDetails;
  equipment: FantasyEquipment;
}): FantasySheetModel {
  const values = sheet.values;
  const profil = catalog.profils.find((p) => p.id === choices.profil);
  const peuple = catalog.peuples.find((p) => p.id === choices.peuple);
  const family = details.family ? ` (${details.family.name})` : "";

  return {
    identity: [
      { label: "Profil", value: profil ? `${profil.name}${family}` : "À choisir" },
      { label: "Peuple", value: peuple?.name ?? "À choisir" },
      { label: "Niveau", value: `${details.level} (d4° : ${details.evolutiveDie})` },
    ],
    resources: RESOURCES.flatMap(({ id, short }) => {
      const line = valueLine(values, id);
      return line?.total === undefined
        ? []
        : [{ id, short, label: line.label, max: line.total, dice: line.dice }];
    }),
    combat: COMBAT.map(({ id, ...text }) => tile(id, valueLine(values, id), text)),
    situational: values.filter((line) => !isCofWeaponValue(line.id)),
    // From the engine's damage lines: one per use, in the order of the
    // catalogue's uses, then the critical range.
    weapons: cofWeaponSlugs(values).flatMap((slug) => {
      const weapon = catalog.weapons.find((w) => w.id === cofWeapon(slug));
      if (!weapon) return [];
      const uses = values
        .filter((line) => line.id.startsWith(cofDamagePrefix(slug)))
        .map((damage, i) => {
          const attackId = cofAttackOf(damage.id);
          return {
            name: weapon.uses[i]?.name ?? "",
            attackId,
            attack: valueLine(values, attackId),
            damage,
          };
        });
      return [
        {
          weapon,
          quantity: carried(equipment.items, weapon.id),
          uses,
          critical: valueLine(values, cofCriticalRange(slug)),
        },
      ];
    }),
    voies: currentVoies(catalog, details.abilitiesHeld),
    armour: catalog.armours.find((a) => a.id === equipment.armour),
    shield: catalog.shields.find((s) => s.id === equipment.shield),
    money: details.money ?? undefined,
  };
}
