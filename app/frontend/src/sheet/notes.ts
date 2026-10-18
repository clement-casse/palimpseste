// The waivers and adjustments of a character as numbered notes, each placed
// where the sheet shows what it is about (ADR FE-SHEET). Waivers come first,
// in the document's order, then adjustments.

import { sameSubject, signed } from "../character/edits";
import { message } from "../messages";
import type {
  Characteristic,
  Document,
  ReportedViolation,
  Sheet,
} from "../wasm/palimpseste_wasm";

/** Where the sheet shows the mark of a note. */
export type Place =
  | { kind: "characteristic"; id: Characteristic }
  | { kind: "voie"; id: string }
  | { kind: "value"; id: string }
  | { kind: "step"; id: string };

export type Note = {
  number: number;
  kind: "waiver" | "adjustment";
  /** `null` for a waiver that accepts nothing, shown in the list only. */
  place: Place | null;
  text: string;
  author: string;
  reason: string;
};

/** The place of the thing a waived violation is about. */
function placeOf(violation: ReportedViolation): Place {
  const subject = violation.subject;
  switch (subject.kind) {
    case "characteristic":
      return { kind: "characteristic", id: subject.id };
    case "voie":
      return { kind: "voie", id: subject.id };
    case "ability":
      // A capacité, `<voie>.<rank>`, shows on its voie's row.
      return { kind: "voie", id: subject.id.slice(0, subject.id.lastIndexOf(".")) };
    default:
      return { kind: "step", id: violation.step };
  }
}

export function notesOf(sheet: Sheet, document: Document): Note[] {
  const notes: Note[] = [];
  for (const waiver of document.waivers ?? []) {
    const same = (w: { rule: string; subject: ReportedViolation["subject"] }) =>
      w.rule === waiver.rule && sameSubject(w.subject, waiver.subject);
    const violation = sheet.unusedWaivers.some(same)
      ? undefined
      : sheet.violations.find((v) => v.waived && same(v));
    notes.push({
      number: notes.length + 1,
      kind: "waiver",
      place: violation ? placeOf(violation) : null,
      text: violation ? message(violation) : `Sans objet (${waiver.rule}).`,
      author: waiver.author,
      reason: waiver.reason,
    });
  }
  for (const adjustment of document.adjustments ?? []) {
    const target = adjustment.target;
    const label =
      target.kind === "characteristic"
        ? target.id
        : (sheet.values.find((line) => line.id === target.id)?.label ?? target.id);
    notes.push({
      number: notes.length + 1,
      kind: "adjustment",
      place: target.kind === "characteristic" ? { ...target } : { kind: "value", id: target.id },
      text: `${label} ${signed(adjustment.amount)}.`,
      author: adjustment.author,
      reason: adjustment.reason,
    });
  }
  return notes;
}

/** The notes shown at any of `places`. */
export function marksAt(notes: Note[], ...places: Place[]): Note[] {
  return notes.filter(
    (note) =>
      note.place !== null &&
      places.some((place) => place.kind === note.place?.kind && place.id === note.place.id),
  );
}

/** The places of the sections showing creation steps `ids`. */
export function steps(...ids: string[]): Place[] {
  return ids.map((id) => ({ kind: "step", id }));
}
