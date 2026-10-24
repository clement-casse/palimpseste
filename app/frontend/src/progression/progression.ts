// What the level-ups (COF2) and spendings of XP (COC2) of a document are,
// whatever the engine (ADR FE-PROGRESSION). A COF2 level-up is numbered by
// the level it reaches, from 2; a COC2 spending from 1.

import type {
  AbilityEntry,
  Document,
  HeldAbility,
  Sheet,
  VoieEntry,
} from "../wasm/palimpseste_wasm";

/** What names the capacités: either engine's catalog. */
export type Abilities = { abilities: AbilityEntry[] };

/** How many level-ups or spendings of XP the document holds. */
export function progressionCount(document: Document): number {
  return document.engine === "fantasy"
    ? (document.levelUps?.length ?? 0)
    : (document.xpSpendings?.length ?? 0);
}

/** The number of the entry at `index`: its level (COF2) or rank (COC2). */
export function numberAt(document: Document, index: number): number {
  return document.engine === "fantasy" ? index + 2 : index + 1;
}

/** The index of the entry numbered `number`. */
export function indexOf(document: Document, number: number): number {
  return document.engine === "fantasy" ? number - 2 : number - 1;
}

/** The number the next level-up or spending would take. */
export function nextNumber(document: Document): number {
  return numberAt(document, progressionCount(document));
}

/** What starting the next one is called. */
export function nextLabel(document: Document): string {
  return document.engine === "fantasy"
    ? `Monter au niveau ${nextNumber(document)}`
    : "Dépenser de l’XP";
}

/** What validating one is called. */
export function validateLabel(document: Document): string {
  return document.engine === "fantasy" ? "Valider le niveau" : "Valider la dépense";
}

/** A deep copy of `document`. */
export function copy<T extends Document>(document: T): T {
  return structuredClone(document);
}

/** A copy of `document` with an empty level-up or spending at its end. */
export function withNext(document: Document): Document {
  const next = copy(document);
  if (next.engine === "fantasy") {
    next.levelUps = [...(next.levelUps ?? []), { abilities: [], forgotten: [] }];
  } else {
    next.xpSpendings = [...(next.xpSpendings ?? []), { abilities: [], traits: [] }];
  }
  return next;
}

/** A copy of `document` keeping only its first `count` entries. */
export function truncated(document: Document, count: number): Document {
  const kept = copy(document);
  if (kept.engine === "fantasy") kept.levelUps = (kept.levelUps ?? []).slice(0, count);
  else kept.xpSpendings = (kept.xpSpendings ?? []).slice(0, count);
  return kept;
}

/** A copy of `document` without its entry at `index`. */
export function without(document: Document, index: number): Document {
  const kept = copy(document);
  if (kept.engine === "fantasy") kept.levelUps = (kept.levelUps ?? []).filter((_, i) => i !== index);
  else kept.xpSpendings = (kept.xpSpendings ?? []).filter((_, i) => i !== index);
  return kept;
}

/**
 * The label of the capacité `id` (ADR ENG-VALUE-IDS): the one place a
 * capacité's identifier turns into what is shown, so its name, description
 * and effects can take over here later.
 */
export function abilityLabel(catalog: Abilities, id: string): string {
  return catalog.abilities.find((entry) => entry.id === id)?.label ?? id;
}

/** The status of the entry numbered `number`, if the sheet has it. */
export function statusOf(sheet: Sheet, number: number) {
  return sheet.progression.find((entry) => entry.number === number);
}

/** Whether the entry numbered `number` has nothing left to settle. */
export function settled(sheet: Sheet, number: number): boolean {
  const status = statusOf(sheet, number);
  return (
    status !== undefined &&
    status.steps.every((step) => step.incomplete === 0 && step.deviations === 0)
  );
}

/** A value or characteristic that changed between two evaluations. */
export type Difference = { label: string; before: number | undefined; after: number | undefined };

/**
 * The characteristics and values whose totals differ from `before` to
 * `after`: what a level-up, a spending or a sequel changes, as the engine
 * computes it.
 */
export function differences(before: Sheet, after: Sheet): Difference[] {
  const found: Difference[] = [];
  for (const line of after.characteristics) {
    const old = before.characteristics.find((c) => c.characteristic === line.characteristic);
    if (old?.total !== line.total) {
      found.push({ label: line.characteristic, before: old?.total, after: line.total });
    }
  }
  for (const line of after.values) {
    const old = before.values.find((v) => v.id === line.id);
    if (old?.total !== line.total) found.push({ label: line.label, before: old?.total, after: line.total });
  }
  return found;
}

/**
 * The voies held now and their ranks, from the capacités held: those chosen
 * at creation and those gained since, without the ones forgotten.
 */
export function currentVoies(catalog: Abilities, held: HeldAbility[]): VoieEntry[] {
  const voies = new Map<string, number[]>();
  for (const { ability } of held) {
    const entry = catalog.abilities.find((a) => a.id === ability);
    if (!entry) continue;
    voies.set(entry.voie, [...(voies.get(entry.voie) ?? []), entry.rank]);
  }
  return [...voies].map(([id, ranks]) => ({ id, ranks }));
}
