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

/** A whole number typed by the user, `undefined` when blank or not a number. */
export function integerOf(event: Event): number | undefined {
  const text = valueOf(event).trim();
  const value = Number(text);
  return text === "" || !Number.isInteger(value) ? undefined : value;
}

export function signed(value: number): string {
  return value > 0 ? `+${value}` : String(value);
}
