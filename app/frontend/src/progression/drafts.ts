// The level-up or spending of XP being prepared, kept aside on this device
// until it is validated (ADR FE-PROGRESSION): a copy of the whole document
// with the entry being prepared, and the version of the character it
// started from. One draft per character; storage may be blocked, then the
// draft lasts as long as the page.

import type { Document } from "../wasm/palimpseste_wasm";

export type Draft = {
  /** The level (COF2) or rank (COC2) of the entry prepared. */
  number: number;
  /** Whether it is the next one, not yet in the saved document. */
  fresh: boolean;
  /** The version of the character the draft started from. */
  base: number;
  /** The whole document, the entry prepared included. */
  document: Document;
};

const key = (id: number) => `palimpseste.draft.${id}`;

export function loadDraft(id: number): Draft | null {
  try {
    const stored = localStorage.getItem(key(id));
    return stored ? (JSON.parse(stored) as Draft) : null;
  } catch {
    return null;
  }
}

export function saveDraft(id: number, draft: Draft) {
  try {
    localStorage.setItem(key(id), JSON.stringify(draft));
  } catch {
    // Storage blocked or full: the draft lives as long as the page.
  }
}

export function clearDraft(id: number) {
  try {
    localStorage.removeItem(key(id));
  } catch {
    // Nothing kept.
  }
}
