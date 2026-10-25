// How the sheets write the engine's computed values.

import { signed } from "../character/edits";
import type { SituationalLine, ValueLine } from "../wasm/palimpseste_wasm";

/**
 * A modifier holding only in a situation (ADR ENG-SITUATIONAL), such as
 * `+2 contre plus grand que lui`, `+1d6 contre un animal`, or the situation
 * alone for a note.
 */
export function situationalText(line: SituationalLine): string {
  if (line.amount != null) return `${signed(line.amount)} ${line.situation}`;
  if (line.dice) return `+${line.dice} ${line.situation}`;
  return line.situation;
}

/** The value, signed when it is a modifier; a dash while it cannot be computed. */
export function shown(line: ValueLine | undefined, sign = false): string {
  if (line?.total === undefined) return "—";
  return sign ? signed(line.total) : String(line.total);
}

/** Weapon damage: its dice and the DM added, such as `1d8+3`. */
export function damage(line: ValueLine | undefined): string {
  if (line?.total === undefined) return "—";
  const dice = line.dice ?? "";
  return line.total === 0 ? dice : `${dice}${signed(line.total)}`;
}

/** A critical range from its lowest roll, such as `19-20`. */
export function range(line: ValueLine | undefined): string {
  if (line?.total === undefined) return "—";
  return line.total >= 20 ? String(line.total) : `${line.total}-20`;
}

/** The identifiers of the weapons that values `prefix.<weapon>…` are about, in order. */
export function weaponSlugs(values: ValueLine[], prefix: string): string[] {
  const slugs = values
    .filter((line) => line.id.startsWith(`${prefix}.`))
    .map((line) => line.id.slice(prefix.length + 1).split(".")[0]);
  return [...new Set(slugs)];
}
