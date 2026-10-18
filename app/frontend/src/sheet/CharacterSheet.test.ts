import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

import { unsettled } from "../engines";
import { amelia, character, lhagva, stubBackend } from "../test/backend";
import { setTheme } from "../theme.svelte";
import { type Document, catalog, emptyDocument, evaluate } from "../wasm/palimpseste_wasm";
import CharacterSheet from "./CharacterSheet.svelte";
import HealthTrack from "./HealthTrack.svelte";
import SheetPage from "./SheetPage.svelte";

function open(document: Document, name = "Lhagva") {
  return render(CharacterSheet, { character: character(document, name) });
}

/** What is left of the resource named `name`. */
const left = (name: string) =>
  within(screen.getByRole("group", { name })).getByRole("status").textContent;

function maximum(document: Document, id: string): number {
  const total = evaluate(document).values.find((line) => line.id === id)?.total;
  if (total === undefined) throw new Error(`no ${id}`);
  return total;
}

afterEach(() => {
  localStorage.clear();
  setTheme("system");
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe("the COF sheet", () => {
  it("shows the character at a glance", () => {
    open(lhagva());
    expect(screen.getByRole("heading", { level: 2 }).textContent).toBe("Lhagva");
    expect(screen.getByText(/Barbare \(Combattants\) · Humain · Niveau 1/)).toBeTruthy();
    expect(screen.getByRole("rowheader", { name: "Épée longue" })).toBeTruthy();
    expect(screen.getByRole("rowheader", { name: "Javelot ×2" })).toBeTruthy();
    expect(screen.getByText("adventurer's bag")).toBeTruthy();
    expect(screen.getByText(/Bourse : 7 pa/)).toBeTruthy();
    expect(screen.getByText("d10")).toBeTruthy();
    // A barbarian knows no spell, so has no PM.
    expect(screen.queryByRole("group", { name: "Points de mana (PM)" })).toBeNull();
  });

  it("counts PV down and up on the device, and restores them all", async () => {
    const pv = maximum(lhagva(), "cof.vigour-points");
    const sheet = open(lhagva());
    expect(left("Points de vigueur (PV)")).toBe(`${pv} / ${pv}`);

    await fireEvent.click(screen.getByRole("button", { name: "Retirer 1 PV" }));
    await fireEvent.click(screen.getByRole("button", { name: "Retirer 1 PV" }));
    await fireEvent.click(screen.getByRole("button", { name: "Ajouter 1 PV" }));
    expect(left("Points de vigueur (PV)")).toBe(`${pv - 1} / ${pv}`);

    sheet.unmount();
    open(lhagva());
    expect(left("Points de vigueur (PV)")).toBe(`${pv - 1} / ${pv}`);

    vi.stubGlobal("confirm", () => true);
    await fireEvent.click(screen.getByRole("button", { name: "Tout restaurer" }));
    expect(left("Points de vigueur (PV)")).toBe(`${pv} / ${pv}`);
  });

  it("marks a waiver where it applies and lists it", () => {
    const document = lhagva();
    document.voies = (document.voies ?? []).map((v) =>
      v.id === "cof.rage" ? { ...v, ranks: [1, 2] } : v,
    );
    const violation = evaluate(document).violations.find((v) => v.rule === "COF.PRG.LEVEL-REQ");
    if (!violation) throw new Error("no deviation");
    document.waivers = [
      { rule: violation.rule, subject: violation.subject, author: "Ana", reason: "Vétéran" },
    ];
    open(document);

    const voies = screen.getByRole("heading", { name: "Voies" }).closest("section");
    if (!voies) throw new Error("no voies section");
    expect(within(voies).getByRole("button", { name: "Note 1" })).toBeTruthy();
    const list = screen.getByRole("heading", { name: "Dérogations et ajustements" }).closest("section");
    if (!list) throw new Error("no notes section");
    expect(within(list).getByText(/Ana : Vétéran/)).toBeTruthy();
  });

  it("says when the character is unfinished", () => {
    const finished = unsettled(evaluate(lhagva()).steps) === 0;
    const sheet = open(lhagva());
    expect(screen.queryByRole("note") === null).toBe(finished);
    sheet.unmount();

    open(emptyDocument("fantasy"), "Nouveau");
    expect(screen.getByRole("note").textContent).toMatch(/Création inachevée : \d+ points à régler/);
    expect(screen.getByRole("link", { name: "Reprendre la création" }).getAttribute("href")).toBe(
      "/characters/1",
    );
  });

  it("keeps the screen on while shown", async () => {
    const lock = { release: vi.fn(async () => {}), addEventListener: vi.fn() };
    const request = vi.fn(async () => lock);
    vi.stubGlobal("navigator", { ...navigator, wakeLock: { request } });

    const sheet = open(lhagva());
    await vi.waitFor(() => expect(request).toHaveBeenCalledWith("screen"));
    const toggle = screen.getByRole("button", { name: "Écran toujours allumé" });
    expect(toggle.getAttribute("aria-pressed")).toBe("true");

    sheet.unmount();
    await vi.waitFor(() => expect(lock.release).toHaveBeenCalledOnce());
  });

  it("switches the theme of the device", async () => {
    open(lhagva());
    await fireEvent.change(screen.getByRole("combobox", { name: "Thème" }), {
      target: { value: "oled" },
    });
    expect(document.documentElement.dataset.theme).toBe("oled");
  });
});

describe("the COC sheet", () => {
  it("shows the character at a glance", () => {
    open(amelia(), "Amelia");
    expect(screen.getByText(/32 ans/)).toBeTruthy();
    const knife = screen
      .getByRole("rowheader", { name: "Couteau, poignard, baïonnette" })
      .closest("tr");
    if (!knife) throw new Error("no knife row");
    expect(within(knife).getByText("+2")).toBeTruthy();
    expect(within(knife).getByText("1d6+2")).toBeTruthy();
    expect(screen.getByRole("rowheader", { name: "Mains nues" })).toBeTruthy();
    expect(screen.getByText("Force de la nature (+2)")).toBeTruthy();
    expect(screen.getByText("motorbike")).toBeTruthy();
  });

  it("fills the échelle de santé, kept on the device", async () => {
    const sheet = open(amelia(), "Amelia");
    const health = screen.getByRole("group", { name: "Échelle de santé" });
    expect(within(health).getAllByRole("button", { name: /^Case/ })).toHaveLength(20);
    expect(within(health).getByRole("status").textContent).toBe("0 / 20 DM");
    expect(within(health).getByText("Aucun état")).toBeTruthy();

    for (let i = 0; i < 5; i++) {
      await fireEvent.click(within(health).getByRole("button", { name: "Ajouter 1 DM" }));
    }
    expect(within(health).getByRole("status").textContent).toBe("5 / 20 DM");
    expect(within(health).queryByText("Aucun état")).toBeNull();

    await fireEvent.click(within(health).getByRole("button", { name: "Case 10" }));
    expect(within(health).getByRole("status").textContent).toBe("10 / 20 DM");
    // The last step filled empties again.
    await fireEvent.click(within(health).getByRole("button", { name: "Case 10" }));
    expect(within(health).getByRole("status").textContent).toBe("9 / 20 DM");

    sheet.unmount();
    open(amelia(), "Amelia");
    expect(screen.getByRole("group", { name: "Échelle de santé" }).textContent).toContain(
      "9 / 20 DM",
    );
  });
});

describe("a shorter échelle de santé", () => {
  const scale = () => {
    const content = catalog("contemporain");
    if (content.engine !== "contemporain") throw new Error("contemporain catalog expected");
    return content.healthScale;
  };

  function track(length: number, damage: number) {
    return render(HealthTrack, { scale: scale(), length, damage, notes: [], onchange: () => {} });
  }

  it("blackens its first steps, and damage fills from the next one", () => {
    track(16, 1);
    expect(screen.getAllByTitle(/noircie$/)).toHaveLength(4);
    expect(screen.getAllByRole("button", { name: /^Case/ })).toHaveLength(16);
    expect(screen.getByRole("button", { name: "Case 5" }).getAttribute("aria-pressed")).toBe("true");
    expect(screen.queryByText("Aucun état")).toBeNull();
  });

  it("never applies a state on a blackened step", () => {
    track(12, 1);
    expect(screen.getByText("Aucun état")).toBeTruthy();
  });
});

describe("the sheet page", () => {
  it("loads the character", async () => {
    stubBackend((request) =>
      request.url === "/api/characters/1"
        ? Response.json(character(lhagva()))
        : new Response(null, { status: 404 }),
    );
    render(SheetPage, { id: 1 });
    expect((await screen.findByRole("heading", { level: 2 })).textContent).toBe("Lhagva");
  });
});
