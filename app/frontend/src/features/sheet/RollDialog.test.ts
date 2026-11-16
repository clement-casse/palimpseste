import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

import { amelia, character, lhagva } from "../../test/backend";
import { type Document, emptyDocument, rollOffer } from "$lib/engine";
import CharacterSheet from "./CharacterSheet.svelte";
import { equipmentOf } from "./equipment";

/** The next dice show `faces`, in order. */
function dice(...faces: number[]) {
  const bytes = faces.map((face) => face - 1);
  vi.spyOn(crypto, "getRandomValues").mockImplementation((array) => {
    const next = bytes.shift();
    if (next === undefined) throw new Error("no die left to roll");
    if (array instanceof Uint8Array) array[0] = next;
    return array;
  });
}

/** A COF2 character with a competence bonus to some CHA tests (intimidation). */
function bully(): Document {
  const document = emptyDocument("fantasy");
  if (document.engine !== "fantasy") throw new Error("not a COF2 document");
  document.voies = [{ id: "cof.spadassin", ranks: [1] }];
  return document;
}

/** What the roll adds to its dice whatever the circumstances, from the engine. */
function base(document: Document, roll: string): number {
  return rollOffer(document, { equipment: equipmentOf(document), damage: 0 }, roll).base;
}

const dialog = () => within(screen.getByRole("dialog"));
const total = () => dialog().getByRole("status", { name: "Total" }).textContent;
const open = (name: string | RegExp) => fireEvent.click(screen.getAllByRole("button", { name })[0]);
const mode = (name: string) => dialog().getByRole("radio", { name }).getAttribute("aria-checked");
const unfold = () =>
  fireEvent.click(dialog().getByRole("button", { name: /^Autres bonus et modificateurs/ }));

afterEach(() => {
  localStorage.clear();
  vi.restoreAllMocks();
});

describe("rolling from the sheet", () => {
  it("rolls a characteristic test and shows where the total comes from", async () => {
    const document = bully();
    render(CharacterSheet, { character: character(document) });
    await open("Lancer : Test de CHA");
    expect(dialog().getByRole("heading", { name: "Test de CHA" })).toBeTruthy();
    // The circumstances the engine cannot know are offered, unpicked.
    const bonus = dialog().getByRole("checkbox", { name: /\+3 intimidation/ });
    expect(bonus).toHaveProperty("checked", false);

    dice(10);
    await fireEvent.click(dialog().getByRole("button", { name: "Lancer le dé" }));
    expect(total()).toBe(String(10 + base(document, "cof.roll.test.cha")));
    expect(dialog().getByText("d20 : 10")).toBeTruthy();
    // What was left unpicked is recalled, for the Game Master to hear.
    const recalled = dialog().getByText("Peut aussi s’appliquer, à annoncer au MJ").parentElement;
    expect(recalled?.textContent).toContain("+3 intimidation");
  });

  it("folds away the bonuses of other characteristics, still offered", async () => {
    // Intimidation is usually a CHA test (D-60): on an AGI test, it waits
    // in the folded section, with the help and the modifier.
    const document = bully();
    render(CharacterSheet, { character: character(document) });
    await open("Lancer : Test de AGI");
    expect(dialog().getByRole("radiogroup", { name: "Dé" })).toBeTruthy();
    expect(dialog().getByRole("radio", { name: "Dé normal" }).getAttribute("aria-checked")).toBe(
      "true",
    );
    expect(dialog().queryByRole("checkbox", { name: /\+3 intimidation/ })).toBeNull();
    expect(dialog().queryByRole("spinbutton", { name: "Modificateur du MJ" })).toBeNull();

    await unfold();
    await fireEvent.click(dialog().getByRole("checkbox", { name: /\+3 intimidation/ }));
    dice(10);
    await fireEvent.click(dialog().getByRole("button", { name: "Lancer le dé" }));
    expect(total()).toBe(String(10 + base(document, "cof.roll.test.agi") + 3));
  });

  it("keeps a mode chosen when it is tapped again", async () => {
    render(CharacterSheet, { character: character(bully()) });
    await open("Lancer : Test de CHA");
    await fireEvent.click(dialog().getByRole("radio", { name: "Dé bonus" }));
    await fireEvent.click(dialog().getByRole("radio", { name: "Dé bonus" }));
    expect(dialog().getByRole("radio", { name: "Dé bonus" }).getAttribute("aria-checked")).toBe(
      "true",
    );
  });

  it("starts with the dé bonus the character has on every test", async () => {
    // Force héroïque: a dé bonus to every FOR test.
    const document = emptyDocument("fantasy");
    if (document.engine !== "fantasy") throw new Error("not a COF2 document");
    document.voies = [{ id: "cof.brute", ranks: [1, 2, 3, 4] }];
    render(CharacterSheet, { character: character(document) });
    await open("Lancer : Test de FOR");
    expect(mode("Dé bonus")).toBe("true");
    const granted = dialog().getByRole("list", { name: "Dés bonus acquis" });
    expect(granted.textContent).toContain("Force héroïque");
    expect(dialog().queryByRole("checkbox", { name: /^Dé bonus/ })).toBeNull();
  });

  it("offers the dé bonus of a circumstance among the circumstances", async () => {
    // Expert en criminalité: a dé bonus to investigation, usually INT or
    // PER (D-60); only the player knows whether the test is one.
    const document = emptyDocument("fantasy");
    if (document.engine !== "fantasy") throw new Error("not a COF2 document");
    document.voies = [{ id: "cof.roublard", ranks: [1, 2, 3, 4] }];
    render(CharacterSheet, { character: character(document) });
    await open("Lancer : Test de INT");
    expect(mode("Dé normal")).toBe("true");
    const die = () => dialog().getByRole("checkbox", { name: /Dé bonus : investigation/ });
    await fireEvent.click(die());
    expect(mode("Dé bonus")).toBe("true");
    await fireEvent.click(die());
    expect(mode("Dé normal")).toBe("true");
    // The die the Game Master imposed comes back once no dé bonus applies.
    await fireEvent.click(dialog().getByRole("radio", { name: "Dé malus" }));
    await fireEvent.click(die());
    expect(mode("Dé bonus")).toBe("true");
    await fireEvent.click(die());
    expect(mode("Dé malus")).toBe("true");
    // Another die chosen by hand sets the dé bonus picked aside.
    await fireEvent.click(die());
    await fireEvent.click(dialog().getByRole("radio", { name: "Dé normal" }));
    expect(die()).toHaveProperty("checked", false);
    await fireEvent.click(screen.getByRole("button", { name: /Close|Fermer/ }));

    // On another characteristic, it waits folded away.
    await open("Lancer : Test de FOR");
    expect(dialog().queryByRole("checkbox", { name: /Dé bonus : investigation/ })).toBeNull();
    await unfold();
    await fireEvent.click(die());
    expect(mode("Dé bonus")).toBe("true");
    expect(
      dialog().getByRole("button", { name: /^Autres bonus et modificateurs/ }).textContent,
    ).toContain("(1 en cours)");
  });

  it("adds what the player picks: a bonus, the allies helping, the modifier, a dé bonus", async () => {
    const document = bully();
    render(CharacterSheet, { character: character(document) });
    await open("Lancer : Test de CHA");
    await fireEvent.click(dialog().getByRole("checkbox", { name: /\+3 intimidation/ }));
    await unfold();
    await fireEvent.click(dialog().getByRole("button", { name: "Un allié de plus" }));
    await fireEvent.click(dialog().getByRole("button", { name: "Un allié de plus" }));
    await fireEvent.click(dialog().getByRole("radio", { name: "Dé bonus" }));
    const modifier = dialog().getByRole("spinbutton", { name: "Modificateur du MJ" });
    await fireEvent.change(modifier, { target: { value: "-1" } });
    expect(
      dialog().getByRole("button", { name: /^Autres bonus et modificateurs/ }).textContent,
    ).toContain("(2 en cours)");

    dice(4, 15);
    await fireEvent.click(dialog().getByRole("button", { name: "Lancer le dé" }));
    expect(total()).toBe(String(15 + base(document, "cof.roll.test.cha") + 3 + 4 - 1));
    expect(dialog().getByText(/d20 : 4/).textContent).toContain("(écarté)");
    const lines = dialog().getByRole("list", { name: "Calcul" }).textContent;
    expect(lines).toContain("Coopération : 2 alliés");
    expect(lines).toContain("Modificateur du MJ");
    expect(lines).toContain("intimidation");
    expect(dialog().queryByText("Peut aussi s’appliquer, à annoncer au MJ")).toBeNull();
  });

  it("follows a critical attack with its DM, doubled", async () => {
    render(CharacterSheet, { character: character(lhagva()) });
    await open(/^Lancer : attaque, Épée longue/);
    expect(dialog().queryByRole("group", { name: /Alliés/ })).toBeNull();
    dice(20);
    await fireEvent.click(dialog().getByRole("button", { name: "Lancer le dé" }));
    expect(dialog().getByText("20 naturel")).toBeTruthy();
    expect(dialog().getByText("Dans la zone de critique (20)")).toBeTruthy();

    await fireEvent.click(dialog().getByRole("button", { name: "Lancer les DM (critique)" }));
    expect(dialog().getByRole("heading", { name: /^DM : Épée longue/ })).toBeTruthy();
    expect(dialog().getByRole("checkbox", { name: "Coup critique : DM doublés" })).toHaveProperty(
      "checked",
      true,
    );
    dice(5);
    await fireEvent.click(dialog().getByRole("button", { name: "Lancer les DM" }));
    expect(dialog().getByText("DM critiques")).toBeTruthy();
    expect(dialog().getByRole("list", { name: "Calcul" }).textContent).toContain(
      "Critique : DM doublés",
    );
    expect(Number(total()) % 2).toBe(0);
  });

  it("offers the malus of the state of health, picked for a physical test", async () => {
    localStorage.setItem("palimpseste.play.1", JSON.stringify({ "coc.health-scale": 10 }));
    render(CharacterSheet, { character: character(amelia(), "Amelia") });
    await open("Lancer : Test de CON");
    expect(dialog().getByRole("checkbox", { name: /-3 Affaibli/ })).toHaveProperty("checked", true);
    await unfold();
    expect(dialog().getByRole("group", { name: "Alliés aidant" })).toBeTruthy();
  });

  it("keeps the rolls in the journal, on the device", async () => {
    const document = bully();
    const { unmount } = render(CharacterSheet, { character: character(document) });
    expect(screen.getByText(/Aucun jet pour l’instant/)).toBeTruthy();
    await open("Lancer : Test de CHA");
    dice(10);
    await fireEvent.click(dialog().getByRole("button", { name: "Lancer le dé" }));
    unmount();

    render(CharacterSheet, { character: character(document) });
    const journal = screen.getByRole("heading", { name: "Journal des jets" }).closest("section");
    if (!journal) throw new Error("no journal");
    expect(within(journal).getByText("Test de CHA")).toBeTruthy();
    await fireEvent.click(within(journal).getByRole("button", { name: "Vider le journal" }));
    expect(screen.getByText(/Aucun jet pour l’instant/)).toBeTruthy();
  });

  it("rolls nothing on a friend's sheet", () => {
    const friend = { id: 3, nickname: "Ben", profile: "" };
    render(CharacterSheet, { character: character(lhagva()), friend });
    expect(screen.queryAllByRole("button", { name: /^Lancer :/ })).toEqual([]);
    expect(screen.queryByRole("heading", { name: "Journal des jets" })).toBeNull();
  });
});
