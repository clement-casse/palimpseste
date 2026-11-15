// How the sheets write the engine's computed values.

import { signed } from "../character/edits";
import type { RollOptionLine, SituationalLine, ValueLine } from "../lib/engine";

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

/** A critical range from its lowest roll `from`, such as `19-20`. */
export function rangeFrom(from: number): string {
  return from >= 20 ? String(from) : `${from}-20`;
}

/** The critical range of a value, a dash while it cannot be computed. */
export function range(line: ValueLine | undefined): string {
  return line?.total === undefined ? "—" : rangeFrom(line.total);
}

/** What a roll's option adds and to what, such as `+3 intimidation` or `+1d6 contre un animal`. */
export function optionText(option: RollOptionLine): string {
  const amount = option.dice ? `+${option.dice}` : signed(option.amount ?? 0);
  return `${amount} ${option.label}`;
}
