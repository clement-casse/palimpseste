// The level-up or spending of XP being prepared, kept aside on this device
// until it is validated (ADR FE-PROGRESSION): a copy of the whole document
// with the entry being prepared, and the version of the character it
// started from. One draft per character; storage may be blocked, then the
// draft lasts as long as the page.

import { type Document, attempt, evaluate } from "$lib/engine";
import { KEYS, readJson, remove, writeJson } from "$lib/storage";

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

/**
 * Whether `kept` is a draft the engines read: what storage holds may be
 * anything, and a draft kept from an earlier document version no longer
 * evaluates.
 */
function isDraft(kept: unknown): kept is Draft {
  if (typeof kept !== "object" || kept === null) return false;
  const { number, fresh, base, document } = kept as Record<string, unknown>;
  if (!Number.isInteger(number) || typeof fresh !== "boolean" || !Number.isInteger(base)) {
    return false;
  }
  return attempt(() => evaluate(document as Document)).ok;
}

/** The draft kept for character `id`; one that cannot be read is forgotten. */
export function loadDraft(id: number): Draft | null {
  const kept = readJson(KEYS.draft(id));
  if (isDraft(kept)) return kept;
  clearDraft(id);
  return null;
}

export function saveDraft(id: number, draft: Draft) {
  writeJson(KEYS.draft(id), draft);
}

export function clearDraft(id: number) {
  remove(KEYS.draft(id));
}
