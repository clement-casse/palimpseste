import { describe, expect, it } from "vitest";

import { lhagva } from "../test/backend";
import { type Document, type ReportedViolation, evaluate } from "../wasm/palimpseste_wasm";
import { marksAt, notesOf } from "./notes";

/** Lhagva with the deviation `change` makes, found by `find`, and its waiver. */
function waived(change: (document: Document) => void, find: (v: ReportedViolation) => boolean) {
  const document = lhagva();
  change(document);
  const violation = evaluate(document).violations.find(find);
  if (!violation) throw new Error("no such violation");
  const { rule, subject } = violation;
  document.waivers = [{ rule, subject, author: "Ana", reason: "Vu en jeu" }];
  return { document, violation };
}

/** Lhagva's purse out of the 2d6 pa rolled. */
const richer = () =>
  waived(
    (d) => {
      if (d.engine === "fantasy") d.purse = 200;
    },
    (v) => v.subject.kind === "value" && v.subject.id === "cof.purse",
  );

function notes(document: Document) {
  return notesOf(evaluate(document), document);
}

describe("notes", () => {
  it("place a waiver on the voie it is about", () => {
    const { document } = waived(
      (d) => {
        d.voies = (d.voies ?? []).map((v) => (v.id === "cof.rage" ? { ...v, ranks: [1, 2] } : v));
      },
      (v) => v.rule === "COF.PRG.LEVEL-REQ",
    );
    const [note] = notes(document);
    expect(note).toMatchObject({
      number: 1,
      kind: "waiver",
      place: { kind: "voie", id: "cof.rage" },
      author: "Ana",
      reason: "Vu en jeu",
    });
    expect(note.text).toContain("rang 2");
  });

  it("place a waiver on another subject in the section of its step", () => {
    const { document, violation } = richer();
    const [note] = notes(document);
    expect(violation.step).toBe("equipment");
    expect(note.place).toEqual({ kind: "step", id: "equipment" });
    expect(marksAt([note], { kind: "step", id: "equipment" })).toEqual([note]);
  });

  it("list a waiver accepting nothing without a place", () => {
    const document = lhagva();
    document.waivers = [
      { rule: "COF.CRE.VOIES", subject: { kind: "character" }, author: "Ana", reason: "Au cas où" },
    ];
    const [note] = notes(document);
    expect(note.place).toBeNull();
    expect(note.text).toContain("Sans objet");
  });

  it("number adjustments after waivers and mark the value they change", () => {
    const { document } = richer();
    document.adjustments = [
      { target: { kind: "value", id: "cof.defence" }, amount: 1, author: "Ana", reason: "Bague" },
      { target: { kind: "characteristic", id: "FOR" }, amount: -1, author: "Ana", reason: "Blessure" },
    ];
    const all = notes(document);
    expect(all.map((note) => [note.number, note.kind])).toEqual([
      [1, "waiver"],
      [2, "adjustment"],
      [3, "adjustment"],
    ]);
    expect(all[1].text).toBe("Défense (DEF) +1.");
    expect(marksAt(all, { kind: "value", id: "cof.defence" })).toEqual([all[1]]);
    expect(marksAt(all, { kind: "characteristic", id: "FOR" })).toEqual([all[2]]);
  });
});
