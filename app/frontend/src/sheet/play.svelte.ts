// The resources a player counts during a session (ADR FE-SHEET): PV spent,
// steps of the échelle de santé filled… kept on this device only, never
// saved with the character. Each is stored as the amount used, by value
// identifier, so a maximum changed in the editor keeps the count sensible.

export class PlayState {
  #key: string;
  #used = $state<Record<string, number>>({});

  constructor(characterId: number) {
    this.#key = `palimpseste.play.${characterId}`;
    this.#used = read(this.#key);
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

  /** Puts every resource back to its maximum. */
  reset() {
    this.#used = {};
    try {
      localStorage.removeItem(this.#key);
    } catch {
      // Storage refused: nothing was kept there.
    }
  }

  #write() {
    try {
      localStorage.setItem(this.#key, JSON.stringify(this.#used));
    } catch {
      // Storage refused (private browsing, quota): counts hold for this visit.
    }
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
