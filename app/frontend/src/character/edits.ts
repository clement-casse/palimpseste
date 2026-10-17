// Small helpers shared by the engine editors to read and change a choices
// document. Changes keep every other choice: a choice that no longer fits
// the rules turns into a violation, it is never removed.

import type { Characteristic, Subject, VoieEntry } from "../wasm/palimpseste_wasm";

export const CHARACTERISTICS: Characteristic[] = ["AGI", "CON", "FOR", "PER", "CHA", "INT", "VOL"];

/** The ranks acquired in voie `id`. */
export function ranksOf(voies: VoieEntry[] | undefined, id: string): number[] {
  return voies?.find((voie) => voie.id === id)?.ranks ?? [];
}

/** Sets the ranks acquired in voie `id`; no rank drops the voie. */
export function setRanks(holder: { voies?: VoieEntry[] }, id: string, ranks: number[]) {
  const voies = (holder.voies ??= []);
  const index = voies.findIndex((voie) => voie.id === id);
  if (ranks.length === 0) {
    if (index >= 0) voies.splice(index, 1);
  } else if (index >= 0) {
    voies[index].ranks = ranks;
  } else {
    voies.push({ id, ranks });
  }
}

export function sameSubject(a: Subject, b: Subject): boolean {
  return a.kind === b.kind && ("id" in a ? a.id : null) === ("id" in b ? b.id : null);
}

/** The name of each entry, by identifier. */
export function namesOf(entries: { id: string; name: string }[]): Map<string, string> {
  return new Map(entries.map((entry) => [entry.id, entry.name]));
}

/** The value of the input or select that fired `event`. */
export function valueOf(event: Event): string {
  const target = event.currentTarget;
  return target instanceof HTMLInputElement ||
    target instanceof HTMLSelectElement ||
    target instanceof HTMLTextAreaElement
    ? target.value
    : "";
}

// The bounds of the integer types of the choices document: a number beyond
// them makes the whole document unreadable by its engine.
export const U16_MAX = 65_535;
export const U32_MAX = 4_294_967_295;
export const I32_MIN = -2_147_483_648;
export const I32_MAX = 2_147_483_647;

/**
 * The whole number typed in the input that fired `event`, within `min` and
 * `max`, or `undefined` when blank and `blank` allows it. Anything else puts
 * the input back to `current` and gives `null`, so the document never holds
 * a number it cannot store.
 */
export function integerOf(
  event: Event,
  current: number | undefined,
  { min = I32_MIN, max = I32_MAX, blank = true } = {},
): number | undefined | null {
  const text = valueOf(event).trim();
  const value = Number(text);
  if (text === "" && blank) return undefined;
  if (text !== "" && Number.isInteger(value) && value >= min && value <= max) return value;
  if (event.currentTarget instanceof HTMLInputElement) {
    event.currentTarget.value = current === undefined ? "" : String(current);
  }
  return null;
}

export function signed(value: number): string {
  return value > 0 ? `+${value}` : String(value);
}
