// The identifiers of the engines the interface names itself: the values it
// places on a page (a resource in the bar, a combat tile, a budget beside a
// form). Everything else is read from what the engines list, never named
// here. `ids.test.ts` checks that the engines still emit each of them.

import type { ValueLine } from ".";

/** The COF2 values named by the interface. */
export const COF = {
  vigourPoints: "cof.vigour-points",
  manaPoints: "cof.mana-points",
  luckPoints: "cof.luck-points",
  recoveryDice: "cof.recovery-dice",
  initiative: "cof.initiative",
  defence: "cof.defence",
  attackContact: "cof.attack.contact",
  attackRanged: "cof.attack.ranged",
  attackMagic: "cof.attack.magic",
  money: "cof.money",
} as const;

/** The COC2 values named by the interface. */
export const COC = {
  age: "coc.age",
  healthScale: "coc.health-scale",
  experience: "coc.experience",
  initiative: "coc.initiative",
  defence: "coc.defence",
  damageReduction: "coc.damage-reduction",
  healingCapacity: "coc.healing-capacity",
  contactDamageBonus: "coc.contact-damage-bonus",
  agiTestModifier: "coc.agi-test-modifier",
  capacityPoints: "coc.capacity-points",
  maxVoies: "coc.max-voies",
  maxRank: "coc.max-rank",
  caracPoints: "coc.carac-points",
} as const;

/** The COC2 attack modes, in the order the sheet shows their attack values. */
export const COC_ATTACK_MODES = ["unarmed", "contact-weapon", "ranged-weapon", "heavy-weapon"];

/** The COC2 trait cured with XP (COC.PRG.XP-USE). */
export const COC_TOXICOMANE = "coc.trait.toxicomane";

// The values about one weapon are named after it: `<engine>.weapon.<slug>`
// gives `<engine>.damage.<slug>…`, and so on.

/** The catalogue identifier of the COF2 weapon `slug`. */
export const cofWeapon = (slug: string) => `cof.weapon.${slug}`;
/** What starts the damage lines of the COF2 weapon `slug`, one per use. */
export const cofDamagePrefix = (slug: string) => `cof.damage.${slug}.`;
/** The attack with a COF2 weapon in a use, from the identifier of its damage. */
export const cofAttackOf = (damage: string) => damage.replace("cof.damage.", "cof.attack.");
export const cofCriticalRange = (slug: string) => `cof.critical-range.${slug}`;
/** Whether `id` is the attack or the damage of a COF2 weapon in a use. */
export const isCofWeaponValue = (id: string) => /^cof\.(attack|damage)\.[^.]+\.[^.]+$/.test(id);

/** The catalogue identifier of the COC2 weapon `slug`. */
export const cocWeapon = (slug: string) => `coc.weapon.${slug}`;
export const cocDamage = (slug: string) => `coc.damage.${slug}`;
/** The attack value of the COC2 attack mode `mode`. */
export const cocAttack = (mode: string) => `coc.attack.${mode}`;
export const cocCriticalBonus = (slug: string) => `coc.critical-bonus.${slug}`;
export const cocCriticalRange = (slug: string) => `coc.critical-range.${slug}`;
/** Whether `id` is the damage of a COC2 weapon. */
export const isCocWeaponValue = (id: string) => id.startsWith("coc.damage.");

/** The line of value `id`, when the engine lists it. */
export function valueLine(values: ValueLine[], id: string): ValueLine | undefined {
  return values.find((line) => line.id === id);
}

/** The COF2 weapons the engine gives damage for, in order. */
export const cofWeaponSlugs = (values: ValueLine[]) => weaponSlugs(values, "cof.damage");
/** The COC2 weapons the engine gives damage for, in order. */
export const cocWeaponSlugs = (values: ValueLine[]) => weaponSlugs(values, "coc.damage");

/** The identifiers of the weapons that values `prefix.<weapon>…` are about, in order. */
function weaponSlugs(values: ValueLine[], prefix: string): string[] {
  const slugs = values
    .filter((line) => line.id.startsWith(`${prefix}.`))
    .map((line) => line.id.slice(prefix.length + 1).split(".")[0]);
  return [...new Set(slugs)];
}
