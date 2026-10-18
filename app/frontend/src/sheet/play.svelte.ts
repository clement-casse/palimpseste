// What a player changes during a session (ADR FE-SHEET), kept on this
// device only, never saved with the character: the resources counted (PV
// spent, steps of the échelle de santé filled…) and the equipment in play.
// Each resource is stored as the amount used, by value identifier, so a
// maximum changed in the editor keeps the count sensible.

import type { Engine, Equipment } from "../wasm/palimpseste_wasm";
import { isEquipment } from "./equipment";

export class PlayState {
  #key: string;
  #equipmentKey: string;
  #used = $state<Record<string, number>>({});
  // Replaced whole, so the engine reads a plain object.
  #equipment = $state.raw<Equipment | null>(null);

  constructor(characterId: number, engine: Engine) {
    this.#key = `palimpseste.play.${characterId}`;
    this.#equipmentKey = `palimpseste.play.${characterId}.equipment`;
    this.#used = read(this.#key);
    this.#equipment = readEquipment(this.#equipmentKey, engine);
  }

  /** The equipment in play, `null` while it is the one chosen at creation. */
  get equipment(): Equipment | null {
    return this.#equipment;
  }

  /** Wears and carries `equipment` in play. */
  equip(equipment: Equipment) {
    this.#equipment = equipment;
    store(this.#equipmentKey, JSON.stringify(equipment));
  }

  /** Goes back to the equipment chosen at creation. */
  forgetEquipment() {
    this.#equipment = null;
    store(this.#equipmentKey, null);
  }

  /** How much of value `id` is used, within 0 and its maximum `max`. */
  used(id: string, max: number): number {
    return Math.max(0, Math.min(this.#used[id] ?? 0, max));
  }

  /** Sets how much of value `id` is used, kept within 0 and `max`. */
  set(id: string, used: number, max: number) {
    this.#used[id] = Math.max(0, Math.min(used, max));
    this.#write();
  }

  /** Puts every resource back to its maximum; the equipment stays. */
  reset() {
    this.#used = {};
    store(this.#key, null);
  }

  #write() {
    store(this.#key, JSON.stringify(this.#used));
  }
}

/** Keeps `value` under `key`, or forgets it for `null`. */
function store(key: string, value: string | null) {
  try {
    if (value === null) localStorage.removeItem(key);
    else localStorage.setItem(key, value);
  } catch {
    // Storage refused (private browsing, quota): it holds for this visit.
  }
}

/** The equipment kept for `key`, `null` when none or unreadable. */
function readEquipment(key: string, engine: Engine): Equipment | null {
  try {
    const kept: unknown = JSON.parse(localStorage.getItem(key) ?? "null");
    return isEquipment(kept, engine) ? kept : null;
  } catch {
    return null;
  }
}

/** What was kept for `key`, anything unreadable dropped. */
function read(key: string): Record<string, number> {
  try {
    const kept: unknown = JSON.parse(localStorage.getItem(key) ?? "{}");
    if (typeof kept !== "object" || kept === null) return {};
    return Object.fromEntries(
      Object.entries(kept).filter(
        (entry): entry is [string, number] => Number.isInteger(entry[1]) && entry[1] > 0,
      ),
    );
  } catch {
    return {};
  }
}
