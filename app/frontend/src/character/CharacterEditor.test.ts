import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { navigate } from "../router.svelte";
import { signedIn } from "../session.svelte";
import { type Sent, character, lhagva, stubBackend } from "../test/backend";
import CharacterEditor from "./CharacterEditor.svelte";

/** A backend that stores every document, after `delay` milliseconds. */
function storing(delay = 0) {
  let version = 1;
  return async (request: Sent) => {
    if (request.method === "PUT") {
      if (delay) await new Promise((resolve) => setTimeout(resolve, delay));
      version += 1;
      return new Response(null, { status: 204, headers: { ETag: `"${version}"` } });
    }
    if (request.method === "PATCH") {
      const { name } = request.body as { name: string };
      if (name === "Échec") return new Response("boom", { status: 500 });
      return Response.json({ ...character(lhagva()), name });
    }
    return new Response(null, { status: 404 });
  };
}

const saves = (sent: Sent[]) => sent.filter((request) => request.method === "PUT");

function open(step: string) {
  navigate(`/characters/1#${step}`);
  signedIn({ nickname: "Ana", profile: "" });
  return render(CharacterEditor, { character: character(lhagva()) });
}

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe("the COF equipment step", () => {
  it("edits armour, starting equipment, items and purse, then saves them", async () => {
    const sent = stubBackend(storing());
    open("equipment");
    expect(screen.getByRole("heading", { level: 2 }).textContent).toBe("Équipement");

    await fireEvent.change(screen.getByRole("combobox", { name: /^Armure/ }), {
      target: { value: "cof.armour.cotte-de-mailles" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Javelot" }));
    await fireEvent.change(screen.getByRole("combobox", { name: "Ajouter une arme" }), {
      target: { value: "cof.weapon.arc-court" },
    });
    await fireEvent.input(screen.getByRole("textbox", { name: "Autre objet" }), {
      target: { value: "Corde" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Ajouter" }));
    await fireEvent.change(screen.getAllByRole("spinbutton", { name: "Quantité" })[0], {
      target: { value: "2" },
    });
    await fireEvent.change(screen.getByRole("spinbutton", { name: /^Bourse/ }), {
      target: { value: "9" },
    });
    expect(saves(sent)).toHaveLength(0);

    await vi.advanceTimersByTimeAsync(600);
    expect(saves(sent)).toHaveLength(1);
    expect(saves(sent)[0].headers["If-Match"]).toBe('"1"');
    expect(saves(sent)[0].body).toMatchObject({
      armour: "cof.armour.cotte-de-mailles",
      purse: 90,
      items: [
        { kind: "weapon", id: "cof.weapon.epee-longue", quantity: 2 },
        { kind: "weapon", id: "cof.weapon.javelot", quantity: 3 },
        { kind: "weapon", id: "cof.weapon.dague", quantity: 1 },
        { kind: "other", name: "adventurer's bag", quantity: 1 },
        { kind: "weapon", id: "cof.weapon.arc-court", quantity: 1 },
        { kind: "other", name: "Corde", quantity: 1 },
      ],
    });
  });

  it("refuses a quantity the document cannot hold, and keeps the editor", async () => {
    stubBackend(storing());
    open("equipment");
    for (const typed of ["-1", "5000000000", "", "1.5"]) {
      const quantity = screen.getAllByRole("spinbutton", { name: "Quantité" })[0];
      await fireEvent.change(quantity, { target: { value: typed } });
      expect((quantity as HTMLInputElement).value, typed).toBe("1");
      expect(screen.getByRole("navigation", { name: "Étapes de création" })).toBeTruthy();
    }
  });
});

describe("the COF voies step", () => {
  it("writes the speciality a capacité asks for, and gives it up when cleared", async () => {
    const sent = stubBackend(storing());
    open("voies");
    const origin = screen.getByRole("textbox", { name: /^Origine : Diversité/ });
    expect((origin as HTMLInputElement).value).toBe("invented");
    await fireEvent.change(origin, { target: { value: " Montagnard " } });
    await vi.advanceTimersByTimeAsync(600);
    expect(saves(sent)[0].body).toMatchObject({
      specialities: [{ ability: "cof.humain.1", speciality: "Montagnard" }],
    });
    await fireEvent.change(origin, { target: { value: "" } });
    await vi.advanceTimersByTimeAsync(600);
    expect(saves(sent)[1].body).toMatchObject({ specialities: [] });
    expect(screen.getAllByText(/écrivez sa spécialité \(origine\)/).length).toBeGreaterThan(0);
  });
});

describe("saving", () => {
  it("saves the edits still waiting when the editor is left", async () => {
    const sent = stubBackend(storing());
    const editor = open("equipment");
    await fireEvent.change(screen.getByRole("combobox", { name: /^Bouclier/ }), {
      target: { value: "cof.shield.petit-bouclier" },
    });
    editor.unmount();
    await vi.runAllTimersAsync();
    expect(saves(sent)).toHaveLength(1);
    expect(saves(sent)[0].body).toMatchObject({ shield: "cof.shield.petit-bouclier" });
  });

  it("saves after the save in flight, at its version, when the editor is left", async () => {
    const sent = stubBackend(storing(300));
    const editor = open("equipment");
    await fireEvent.change(screen.getByRole("combobox", { name: /^Armure/ }), {
      target: { value: "cof.armour.tissus-matelasses" },
    });
    await vi.advanceTimersByTimeAsync(700);
    await fireEvent.change(screen.getByRole("combobox", { name: /^Bouclier/ }), {
      target: { value: "cof.shield.petit-bouclier" },
    });
    editor.unmount();
    await vi.runAllTimersAsync();
    expect(saves(sent).map((save) => save.headers["If-Match"])).toEqual(['"1"', '"2"']);
    expect(saves(sent)[1].body).toMatchObject({
      armour: "cof.armour.tissus-matelasses",
      shield: "cof.shield.petit-bouclier",
    });
  });
});

describe("renaming", () => {
  // Typing fires `input`, leaving the field `change`.
  async function rename(name: string) {
    const field = screen.getByRole("textbox", { name: "Nom du personnage" });
    await fireEvent.input(field, { target: { value: name } });
    await fireEvent.change(field);
    await vi.runAllTimersAsync();
  }

  it("sends a rename back to the first name", async () => {
    const sent = stubBackend(storing());
    open("profil");
    await rename("Lhag");
    await rename("Lhagva");
    const renames = sent.filter((request) => request.method === "PATCH");
    expect(renames.map((request) => request.body)).toEqual([{ name: "Lhag" }, { name: "Lhagva" }]);
  });

  it("says when renaming fails", async () => {
    stubBackend(storing());
    open("profil");
    await rename("Échec");
    expect(screen.getByRole("alert").textContent?.trim()).toMatch(/^Le serveur a rencontré une erreur/);
  });
});

describe("adjusting", () => {
  it("asks for a reason, then lists the adjustment signed by the user", async () => {
    stubBackend(storing());
    open("characteristics");
    await fireEvent.click(screen.getByRole("button", { name: /^AGI/ }));
    await fireEvent.click(screen.getAllByRole("button", { name: "Ajuster…" })[0]);

    const dialog = screen.getByRole("dialog");
    expect(dialog.textContent).toMatch("Ajuster AGI");
    const confirm = screen.getByRole("button", { name: "Confirmer" }) as HTMLButtonElement;
    expect(confirm.disabled).toBe(true);
    await fireEvent.input(screen.getByRole("textbox", { name: "Raison" }), {
      target: { value: "Entraînement" },
    });
    await fireEvent.click(confirm);
    await vi.runAllTimersAsync();

    expect(screen.queryByRole("dialog")).toBeNull();
    expect(screen.getByText(/Ajustement AGI\s+\+1 — Ana : Entraînement/)).toBeTruthy();
  });
});

describe("the steps", () => {
  it("open the step chosen in the list", async () => {
    stubBackend(storing());
    open("profil");
    const steps = screen.getByRole("navigation", { name: "Étapes de création" });
    await fireEvent.click(within(steps).getByRole("button", { name: /Équipement/ }));
    expect(screen.getByRole("heading", { level: 2 }).textContent).toBe("Équipement");
    expect(location.hash).toBe("#equipment");
  });
});
