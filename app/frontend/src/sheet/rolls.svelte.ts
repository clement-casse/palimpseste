// The rolls of a session (ADR FE-ROLLS), kept on this device only, never
// saved with the character: each outcome as the engine returned it, the
// latest first. The journal is read again before each roll is added, so
// two sheets of one character open at once keep each other's rolls.

import type { RollOutcome } from "../wasm/palimpseste_wasm";
import { store } from "./play.svelte";

/** A roll made, and when. */
export interface RollEntry {
  /** When it was rolled, in milliseconds since the epoch. */
  at: number;
  outcome: RollOutcome;
}

/** How many rolls the journal keeps. */
export const KEPT = 50;

export class RollLog {
  #key: string;
  // Replaced whole: an outcome is never changed.
  #entries = $state.raw<RollEntry[]>([]);

  constructor(characterId: number) {
    this.#key = `palimpseste.rolls.${characterId}`;
    this.#entries = read(this.#key);
  }

  /** The rolls kept, the latest first. */
  get entries(): RollEntry[] {
    return this.#entries;
  }

  /** Keeps `outcome`, rolled now, dropping the oldest beyond `KEPT`. */
  add(outcome: RollOutcome) {
    // What another sheet of this character kept since; this visit's own
    // when storage is refused.
    const kept = read(this.#key);
    const before = kept.length > 0 ? kept : this.#entries;
    this.#entries = [{ at: Date.now(), outcome }, ...before].slice(0, KEPT);
    store(this.#key, JSON.stringify(this.#entries));
  }

  /** Forgets every roll. */
  clear() {
    this.#entries = [];
    store(this.#key, null);
  }
}

function isEntry(kept: unknown): kept is RollEntry {
  if (typeof kept !== "object" || kept === null) return false;
  const { at, outcome } = kept as { at?: unknown; outcome?: Partial<Record<keyof RollOutcome, unknown>> | null };
  return (
    typeof at === "number" &&
    typeof outcome === "object" &&
    outcome !== null &&
    typeof outcome.name === "string" &&
    typeof outcome.total === "number" &&
    [outcome.dice, outcome.lines, outcome.unpicked, outcome.setAside].every(Array.isArray)
  );
}

/** What was kept for `key`, anything unreadable dropped. */
function read(key: string): RollEntry[] {
  try {
    const kept: unknown = JSON.parse(localStorage.getItem(key) ?? "[]");
    return Array.isArray(kept) ? kept.filter(isEntry).slice(0, KEPT) : [];
  } catch {
    return [];
  }
}
