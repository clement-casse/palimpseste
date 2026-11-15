// What the app keeps on this device, in the browser's storage: none of it
// reaches the backend. Storage may be refused (private browsing, a quota):
// a read then finds nothing and a write is dropped, so what would have been
// kept holds for this visit only. This module is the only one to touch
// `localStorage` and `sessionStorage`, and names every key.

/** Where each thing is kept. */
export const KEYS = {
  /** The theme chosen (ADR FE-THEMES). */
  theme: "palimpseste.theme",
  /** The page to come back to once signed in, for this tab (ADR FE-SIGN-IN). */
  return: "palimpseste.return",
  /** The level-up or spending being prepared for a character (ADR FE-PROGRESSION). */
  draft: (character: number) => `palimpseste.draft.${character}`,
  /** The resources a character used in play (ADR FE-SHEET). */
  play: (character: number) => `palimpseste.play.${character}`,
  /** The equipment of a character in play (ADR FE-SHEET). */
  equipment: (character: number) => `palimpseste.play.${character}.equipment`,
  /** The rolls of a character (ADR FE-ROLLS). */
  rolls: (character: number) => `palimpseste.rolls.${character}`,
};

/** `local` lasts on the device, `session` as long as the tab. */
export type Area = "local" | "session";

// Reading the property itself throws where storage is blocked.
const storage = (area: Area) => (area === "local" ? localStorage : sessionStorage);

/** The text kept under `key`, `null` when none. */
export function readText(key: string, area: Area = "local"): string | null {
  try {
    return storage(area).getItem(key);
  } catch {
    return null;
  }
}

/**
 * What is kept under `key`, read as JSON; `undefined` when none or not
 * JSON. It may be anything: the caller checks its shape.
 */
export function readJson(key: string, area: Area = "local"): unknown {
  const text = readText(key, area);
  if (text === null) return undefined;
  try {
    return JSON.parse(text) as unknown;
  } catch {
    return undefined;
  }
}

/** Keeps `text` under `key`. */
export function writeText(key: string, text: string, area: Area = "local") {
  try {
    storage(area).setItem(key, text);
  } catch {
    // Storage refused or full: it holds for this visit.
  }
}

/** Keeps `value` under `key`, as JSON. */
export function writeJson(key: string, value: unknown, area: Area = "local") {
  writeText(key, JSON.stringify(value), area);
}

/** Forgets what is kept under `key`. */
export function remove(key: string, area: Area = "local") {
  try {
    storage(area).removeItem(key);
  } catch {
    // Nothing kept.
  }
}
